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
    let app_host: &str = "http://127.0.0.1:8080/";
    let health_check_path: &str = "healthz";

    // parse arguments
    let args: Vec<String> = env::args().collect();
    debug!("{:?}", args);
    if args.len() < 2 {
        panic!("missing arguments. At least one argument is required.");
    }
    let command = &args[1];
    let command_args = &args[2..];

    // start user application
    Command::new(command)
        .args(command_args)
        .spawn()
        .expect("failed to start user application");

    // health check the application
    Retry::spawn(FixedInterval::from_millis(10), move || {
        reqwest::get(app_host.to_string() + health_check_path)
    })
    .await?;

    // start lambda runtime
    lambda_runtime::run(handler(http_proxy_handler)).await?;
    Ok(())
}

async fn http_proxy_handler(event: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let app_host: &str = "http://127.0.0.1:8080/";

    let (parts, body) = event.into_parts();
    let res = Client::new()
        .request(parts.method, app_host)
        .headers(parts.headers)
        .body(body.to_vec())
        .send()
        .await?;

    Ok(Response::builder()
        .status(res.status())
        .body(res.text().await?)
        .expect("failed to send response")
    )
}
