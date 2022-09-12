// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lambda_extension::Extension;
use lambda_http::{Body, Request, RequestExt, Response};
use reqwest::{redirect, Client, Url};
use std::{
    env,
    future::Future,
    mem,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::timeout;
use tokio_retry::{strategy::FixedInterval, Retry};
use tower::Service;

pub use lambda_http::Error;

#[derive(Default)]
pub struct AdapterOptions {
    host: String,
    port: String,
    readiness_check_port: String,
    readiness_check_path: String,
    base_path: Option<String>,
    async_init: bool,
}

impl AdapterOptions {
    pub fn from_env() -> Self {
        AdapterOptions {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            readiness_check_port: env::var("READINESS_CHECK_PORT")
                .unwrap_or_else(|_| env::var("PORT").unwrap_or_else(|_| "8080".to_string())),
            readiness_check_path: env::var("READINESS_CHECK_PATH").unwrap_or_else(|_| "/".to_string()),
            base_path: env::var("REMOVE_BASE_PATH").ok(),
            async_init: env::var("ASYNC_INIT")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }
}

pub struct Adapter {
    client: Arc<Client>,
    healthcheck_url: String,
    async_init: bool,
    ready_at_init: Arc<AtomicBool>,
    domain: Url,
    base_path: Option<String>,
}

impl Adapter {
    /// Create a new Adapter instance.
    /// This function initializes a new HTTP client
    /// to talk with the web server.
    pub fn new(options: &AdapterOptions) -> Adapter {
        let client = Client::builder()
            .redirect(redirect::Policy::none())
            .pool_idle_timeout(Duration::from_secs(4))
            .build()
            .unwrap();

        let healthcheck_url = format!(
            "http://{}:{}{}",
            options.host, options.readiness_check_port, options.readiness_check_path
        );

        let domain = format!("http://{}:{}", options.host, options.port).parse().unwrap();

        Adapter {
            client: Arc::new(client),
            healthcheck_url,
            domain,
            base_path: options.base_path.clone(),
            async_init: options.async_init,
            ready_at_init: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Switch the default HTTP client with a different one.
    pub fn with_client(self, client: Client) -> Self {
        Adapter {
            client: Arc::new(client),
            ..self
        }
    }

    /// Check if the web server has been initialized.
    /// If `Adapter.async_init` is true, cancel this check before
    /// Lambda's init 10s timeout, and let the server boot in the background.
    pub async fn check_init_health(&mut self) {
        let ready_at_init = if self.async_init {
            timeout(Duration::from_secs_f32(9.8), self.check_readiness())
                .await
                .unwrap_or_default()
        } else {
            self.check_readiness().await
        };
        self.ready_at_init.store(ready_at_init, Ordering::SeqCst);
    }

    async fn check_readiness(&self) -> bool {
        let url = self.healthcheck_url.clone();
        is_web_ready(&url).await
    }

    pub async fn run(self) -> Result<(), Error> {
        lambda_http::run(self).await
    }
}

/// Implement a `Tower.Service` that sends the requests
/// to the web server.
impl Service<Request> for Adapter {
    type Response = http::Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: Request) -> Self::Future {
        let async_init = self.async_init;
        let client = self.client.clone();
        let ready_at_init = self.ready_at_init.clone();
        let healthcheck_url = self.healthcheck_url.clone();
        let domain = self.domain.clone();
        let base_path = self.base_path.clone();

        Box::pin(async move {
            fetch_response(
                async_init,
                ready_at_init,
                client,
                base_path,
                domain,
                healthcheck_url,
                event,
            )
            .await
        })
    }
}

async fn fetch_response(
    async_init: bool,
    ready_at_init: Arc<AtomicBool>,
    client: Arc<Client>,
    base_path: Option<String>,
    domain: Url,
    healthcheck_url: String,
    event: Request,
) -> Result<http::Response<Body>, Error> {
    if async_init && !ready_at_init.load(Ordering::SeqCst) {
        is_web_ready(&healthcheck_url).await;
        ready_at_init.store(true, Ordering::SeqCst);
    }

    let path = event.raw_http_path();
    let mut path = path.as_str();
    let (parts, body) = event.into_parts();

    // strip away Base Path if environment variable REMOVE_BASE_PATH is set.
    if let Some(base_path) = base_path.as_deref() {
        path = path.trim_start_matches(base_path);
    }

    let mut app_url = domain;
    app_url.set_path(path);
    app_url.set_query(parts.uri.query());
    tracing::debug!(app_url = %app_url, "sending request to server");

    let app_response = client
        .request(parts.method, app_url.to_string())
        .headers(parts.headers)
        .body(body.to_vec())
        .send()
        .await?;

    let mut lambda_response = Response::builder();
    let _ = mem::replace(lambda_response.headers_mut().unwrap(), app_response.headers().clone());

    let status = app_response.status();
    let body = convert_body(app_response).await?;
    let resp = lambda_response.status(status).body(body).map_err(Box::new)?;

    Ok(resp)
}

async fn is_web_ready(url: &str) -> bool {
    Retry::spawn(FixedInterval::from_millis(10), || check_web_readiness(url))
        .await
        .is_ok()
}

async fn check_web_readiness(url: &str) -> Result<(), i8> {
    match reqwest::get(url).await {
        Ok(response) if { response.status().is_success() } => Ok(()),
        _ => Err(-1),
    }
}

/// Register a Lambda Extension to ensure
/// that the adapter is loaded before any Lambda function
/// associated with it.
pub fn register_default_extension() {
    // register as an external extension
    tokio::task::spawn(async move {
        match Extension::new().with_events(&[]).run().await {
            Ok(_) => {}
            Err(err) => {
                tracing::error!(err = err, "extension terminated unexpectedly");
                panic!("extension thread execution");
            }
        }
    });
}

async fn convert_body(app_response: reqwest::Response) -> Result<Body, Error> {
    tracing::debug!(resp_headers = ?app_response.headers(), "converting response body");

    if app_response.headers().get(http::header::CONTENT_ENCODING).is_some() {
        let content = app_response.bytes().await?;
        return Ok(Body::Binary(content.to_vec()));
    }

    match app_response.headers().get(http::header::CONTENT_TYPE) {
        Some(value) => {
            let content_type = value.to_str().unwrap_or_default();

            if content_type.starts_with("text")
                || content_type.starts_with("application/json")
                || content_type.starts_with("application/javascript")
                || content_type.starts_with("application/xml")
            {
                Ok(Body::Text(app_response.text().await?))
            } else {
                let content = app_response.bytes().await?;
                if content.is_empty() {
                    Ok(Body::Empty)
                } else {
                    Ok(Body::Binary(content.to_vec()))
                }
            }
        }
        None => Ok(Body::Text(app_response.text().await?)),
    }
}
