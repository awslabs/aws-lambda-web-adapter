use lambda_extension::Extension;
use lambda_http::{Body, Request, RequestExt, Response};
use reqwest::{redirect, Client, Url};
use std::{env, future::Future, mem, pin::Pin, time::Duration};
use tokio::{runtime::Handle, time::timeout};
use tokio_retry::{strategy::FixedInterval, Retry};
use tower::Service;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

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
    client: Client,
    healthcheck_url: String,
    async_init: bool,
    ready_at_init: bool,
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
            client,
            healthcheck_url,
            domain,
            base_path: options.base_path.clone(),
            async_init: options.async_init,
            ready_at_init: false,
        }
    }

    /// Switch the default HTTP client with a different one.
    pub fn with_client(self, client: Client) -> Self {
        Adapter { client, ..self }
    }

    /// Check if the web server has been initialized.
    /// If `Adapter.async_init` is true, cancel this check before
    /// Lambda's init 10s timeout, and let the server boot in the background.
    pub async fn check_init_health(&mut self) {
        self.ready_at_init = if self.async_init {
            timeout(Duration::from_secs_f32(9.8), self.check_readiness())
                .await
                .unwrap_or_default()
        } else {
            self.check_readiness().await
        };
    }

    async fn check_readiness(&self) -> bool {
        Retry::spawn(FixedInterval::from_millis(10), || {
            let fut = self.check_health_url();
            async move { fut.await }
        })
        .await
        .unwrap()
    }

    async fn check_health_url(&self) -> Result<bool, std::convert::Infallible> {
        self.client
            .head(&self.healthcheck_url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .or(Ok(false))
    }
}

/// Implement a `Tower.Service` that sends the requests
/// to the web server.
impl Service<Request> for Adapter {
    type Error = Error;
    type Response = http::Response<Body>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Error>>>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: Request) -> Self::Future {
        if self.async_init && !self.ready_at_init {
            let handle = Handle::current();
            handle.block_on(self.check_readiness());
            self.ready_at_init = true;
        }

        let path = event.raw_http_path();
        let mut path = path.as_str();
        let (parts, body) = event.into_parts();

        // strip away Base Path if environment variable REMOVE_BASE_PATH is set.
        if let Some(base_path) = self.base_path.as_deref() {
            path = path.trim_start_matches(base_path);
        }

        let mut app_url = self.domain.clone();
        app_url.set_path(path);
        app_url.set_query(parts.uri.query());
        tracing::debug!(app_url = %app_url, "sending request to server");

        let app_response = self
            .client
            .request(parts.method, app_url.to_string())
            .headers(parts.headers)
            .body(body.to_vec())
            .send();

        Box::pin(async move {
            let app_response = app_response.await?;
            let mut lambda_response = Response::builder();
            let _ = mem::replace(lambda_response.headers_mut().unwrap(), app_response.headers().clone());

            let status = app_response.status();
            let body = convert_body(app_response).await?;
            let resp = lambda_response.status(status).body(body).map_err(Box::new)?;

            Ok(resp)
        })
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
