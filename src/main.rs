use http::{HeaderMap};
use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    Body, IntoResponse, Request, Response,
};
use log::*;
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::env;
use std::process::Command;
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

    // start application process
    let mut app_process = Command::new(command)
        .args(command_args)
        .spawn()
        .expect("failed to start user application");

    // TODO improve signal handling for SIGTERM and SIGCHLD
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
            env::var("LIVENESS_CHECK_PATH").unwrap_or("/".to_string())
        );
        reqwest::get(liveness_check_url)
    })
    .await?;

    // check runtime environment
    match env::var("AWS_LAMBDA_RUNTIME_API") {
        // in lambda runtime, start lambda runtime
        Ok(_val) => {
            HTTP_CLIENT.set(Client::new()).unwrap();
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
    // TODO handle binary data
    Ok(lambda_response
        .status(app_response.status())
        // .body(Body::Text(app_response.text().await?))
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
