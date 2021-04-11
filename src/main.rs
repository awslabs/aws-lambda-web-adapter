use http::HeaderMap;
use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    IntoResponse, Request, Response,
};
use log::*;
use reqwest::Client;
use std::env;
use std::process::Command;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

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

    // start application process
    let mut app_process = Command::new(command)
        .args(command_args)
        .spawn()
        .expect("failed to start user application");

    // TODO improve signal handling
    let app_process_id = app_process.id() as i32;
    ctrlc::set_handler(move || {
        unsafe {
            // send SIGINT to application process
            libc::kill(app_process_id, libc::SIGINT);
        }
    })
    .expect("Error setting Ctrl-C handler");

    // check if the application is live every 10 milliseconds
    Retry::spawn(FixedInterval::from_millis(10), || {
        let liveness_check_url = format!(
            "http://127.0.0.1:{}{}",
            env::var("LIVENESS_CHECK_PORT").unwrap_or("8080".to_string()),
            env::var("LIVENESS_CHECK_PATH").unwrap_or("/healthz".to_string())
        );
        reqwest::get(liveness_check_url)
    })
    .await?;

    // check runtime environment
    match env::var("AWS_LAMBDA_RUNTIME_API") {
        // in lambda runtime, start lambda runtime
        Ok(_val) => {
            lambda_runtime::run(handler(http_proxy_handler)).await?;
            Ok(())
        }
        // not in lambda, just wait for app process
        Err(_e) => {
            app_process.wait().unwrap();
            Ok(())
        }
    }
}

async fn http_proxy_handler(event: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let app_host = format!(
        "http://127.0.0.1:{}",
        env::var("PORT").unwrap_or("8080".to_string())
    );
    let (parts, body) = event.into_parts();
    let app_url = app_host + parts.uri.path_and_query().unwrap().as_str();
    let app_response = Client::new()
        .request(parts.method, app_url)
        .headers(parts.headers)
        .body(body.to_vec())
        .send()
        .await?;

    let mut lambda_response = Response::builder();
    replace_headers(
        lambda_response.headers_mut().unwrap(),
        app_response.headers().clone(),
    );
    Ok(lambda_response
        .status(app_response.status())
        .body(app_response.text().await?)
        .expect("failed to send response"))
}

fn replace_headers(dst: &mut HeaderMap, src: HeaderMap) {
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
