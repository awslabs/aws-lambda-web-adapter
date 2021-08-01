use futures::stream::StreamExt;
use http::HeaderMap;
use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    Body, IntoResponse, Request, Response,
};
use log::*;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use once_cell::sync::OnceCell;
use reqwest::{redirect, Client};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
static HTTP_CLIENT: OnceCell<Client> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // parse arguments
    let args: Vec<String> = env::args().collect();
    debug!("{:?}", args);
    if args.len() < 2 {
        panic!("missing arguments. At least one argument is required.");
    }
    let command = &args[1];
    let command_args = &args[2..];

    // check runtime environment
    match env::var("AWS_LAMBDA_RUNTIME_API") {
        // in lambda
        Ok(_val) => {
            // start the application in a new process
            let app_process = Command::new(command)
                .args(command_args)
                .spawn()
                .expect("failed to start user application");

            // setup signal handler for SIGTERM
            let signals = Signals::new(&[SIGTERM])?;
            let handle = signals.handle();
            let _signals_task = tokio::spawn(handle_signals(
                signals,
                app_process,
                Duration::from_millis(290),
            ));

            // check if the application is ready every 10 milliseconds
            Retry::spawn(FixedInterval::from_millis(10), || {
                let readiness_check_url = format!(
                    "http://127.0.0.1:{}{}",
                    env::var("READINESS_CHECK_PORT").unwrap_or("8080".to_string()),
                    env::var("READINESS_CHECK_PORT").unwrap_or("/".to_string())
                );
                reqwest::get(readiness_check_url)
            })
            .await?;

            // start lambda runtime
            HTTP_CLIENT
                .set(
                    Client::builder()
                        .redirect(redirect::Policy::none())
                        .build()
                        .unwrap(),
                )
                .unwrap();
            lambda_runtime::run(handler(http_proxy_handler)).await?;
            handle.close();
            Ok(())
        }
        // not in lambda
        Err(_e) => {
            // execute the application in this process
            Command::new(command).args(command_args).exec();
            Ok(())
        }
    }
}

async fn handle_signals(
    signals: Signals,
    mut process: Child,
    timeout: Duration,
) -> Result<(), Error> {
    let mut signals = signals.fuse();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM => {
                debug!("SIGTERM received.");
                let pid = Pid::from_raw(process.id() as i32);
                debug!("send SIGTERM to the application process");
                match kill(pid, Signal::SIGTERM) {
                    Ok(()) => {
                        let expire = Instant::now() + timeout;
                        while let Ok(None) = process.try_wait() {
                            if Instant::now() > expire {
                                break;
                            }
                            thread::sleep(Duration::from_millis(10));
                        }
                        if let Ok(None) = process.try_wait() {
                            if let Ok(()) = process.kill() {
                                process.wait()?;
                                debug!("Application process {} is killed", process.id());
                            } else {
                                debug!("Application process {} is terminated", process.id());
                            }
                        }
                        debug!("Application process {} is stopped", process.id());
                    }
                    Err(e) => {
                        error!(
                            "Failed to kill application process {}. Error: {:?}",
                            process.id(),
                            e
                        );
                    }
                };
                debug!("exiting the main process");
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn http_proxy_handler(event: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let app_host = format!(
        "http://127.0.0.1:{}",
        env::var("PORT").unwrap_or("8080".to_string())
    );
    let (parts, body) = event.into_parts();
    let app_url = app_host + parts.uri.path_and_query().unwrap().as_str();
    let app_response = HTTP_CLIENT
        .get()
        .unwrap()
        .request(parts.method, app_url)
        .headers(parts.headers)
        .body(body.to_vec())
        .send()
        .await?;

    let mut lambda_response = Response::builder();
    copy_headers(
        app_response.headers().clone(),
        lambda_response.headers_mut().unwrap(),
    );
    Ok(lambda_response
        .status(app_response.status())
        .body(convert_body(app_response).await?)
        .expect("failed to send response"))
}

async fn convert_body(app_response: reqwest::Response) -> Result<Body, Error> {
    let content_type;
    if let Some(value) = app_response.headers().get(http::header::CONTENT_TYPE) {
        content_type = value.to_str().unwrap_or_default();
    } else {
        // default to "application/json" if content-type header is not available in the response
        content_type = "application/json";
    }

    if app_response.content_length().unwrap_or_default() == 0 {
        Ok(Body::Empty)
    } else if content_type.starts_with("text")
        || content_type.eq("application/json")
        || content_type.eq("application/javascript")
        || content_type.eq("application/xml")
        || content_type.eq("image/svg+xml")
    {
        Ok(Body::Text(app_response.text().await?))
    } else {
        Ok(Body::Binary(app_response.bytes().await?.to_vec()))
    }
}

fn copy_headers(src: HeaderMap, dst: &mut HeaderMap) {
    let mut prev_name = None;
    for (key, value) in src {
        match key {
            Some(key) => {
                dst.insert(key.clone(), value);
                prev_name = Some(key);
            }
            None => match prev_name {
                Some(ref key) => {
                    dst.append(key.clone(), value);
                }
                None => unreachable!("HeaderMap::into_iter yielded None first"),
            },
        }
    }
}
