// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;
use std::{
    env,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use http::{
    header::{HeaderName, HeaderValue},
    Method, StatusCode,
};
use hyper::{
    body::HttpBody,
    client::{connect::Connect, Client, HttpConnector},
    Body,
};
use hyper_rustls::HttpsConnector;
use lambda_http::aws_lambda_events::serde_json;
pub use lambda_http::Error;
use lambda_http::{Request, RequestExt, Response};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_retry::{strategy::FixedInterval, Retry};
use tower::{Service, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use url::Url;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Protocol {
    #[default]
    Http,
    Tcp,
}

impl From<&str> for Protocol {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "http" => Protocol::Http,
            "tcp" => Protocol::Tcp,
            _ => Protocol::Http,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LambdaInvokeMode {
    #[default]
    Buffered,
    ResponseStream,
}

impl From<&str> for LambdaInvokeMode {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "buffered" => LambdaInvokeMode::Buffered,
            "response_stream" => LambdaInvokeMode::ResponseStream,
            _ => LambdaInvokeMode::Buffered,
        }
    }
}

#[derive(Default)]
pub struct AdapterOptions {
    pub host: String,
    pub port: String,
    pub readiness_check_port: String,
    pub readiness_check_path: String,
    pub readiness_check_protocol: Protocol,
    pub base_path: Option<String>,
    pub async_init: bool,
    pub compression: bool,
    pub enable_tls: bool,
    pub tls_server_name: Option<String>,
    pub tls_cert_file: Option<String>,
    pub invoke_mode: LambdaInvokeMode,
}

impl AdapterOptions {
    pub fn from_env() -> Self {
        AdapterOptions {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            readiness_check_port: env::var("READINESS_CHECK_PORT")
                .unwrap_or_else(|_| env::var("PORT").unwrap_or_else(|_| "8080".to_string())),
            readiness_check_path: env::var("READINESS_CHECK_PATH").unwrap_or_else(|_| "/".to_string()),
            readiness_check_protocol: env::var("READINESS_CHECK_PROTOCOL")
                .unwrap_or_else(|_| "HTTP".to_string())
                .as_str()
                .into(),
            base_path: env::var("REMOVE_BASE_PATH").ok(),
            async_init: env::var("ASYNC_INIT")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            compression: env::var("AWS_LWA_ENABLE_COMPRESSION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            enable_tls: env::var("AWS_LWA_ENABLE_TLS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            tls_server_name: env::var("AWS_LWA_TLS_SERVER_NAME").ok(),
            tls_cert_file: env::var("AWS_LWA_TLS_CERT_FILE").ok(),
            invoke_mode: env::var("AWS_LWA_INVOKE_MODE")
                .unwrap_or("buffered".to_string())
                .as_str()
                .into(),
        }
    }
}

#[derive(Clone)]
pub struct Adapter<C> {
    client: Arc<Client<C>>,
    healthcheck_url: Url,
    healthcheck_protocol: Protocol,
    async_init: bool,
    ready_at_init: Arc<AtomicBool>,
    domain: Url,
    base_path: Option<String>,
    compression: bool,
    invoke_mode: LambdaInvokeMode,
}

impl Adapter<HttpsConnector<HttpConnector>> {
    /// Create a new HTTPS Adapter instance.
    /// This function initializes a new HTTPS client
    /// to talk with the web server.
    pub fn new_https(options: &AdapterOptions) -> Adapter<HttpsConnector<HttpConnector>> {
        if let Some(cert_file) = &options.tls_cert_file {
            env::set_var("SSL_CERT_FILE", cert_file);
        }

        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .with_server_name(
                options
                    .tls_server_name
                    .clone()
                    .unwrap_or_else(|| "localhost".to_string()),
            )
            .enable_http1()
            .build();

        let client = Client::builder().pool_idle_timeout(Duration::from_secs(4)).build(https);

        let schema = "https";

        let healthcheck_url = format!(
            "{}://{}:{}{}",
            schema, options.host, options.readiness_check_port, options.readiness_check_path
        )
        .parse()
        .unwrap();

        let domain = format!("{}://{}:{}", schema, options.host, options.port)
            .parse()
            .unwrap();

        Adapter {
            client: Arc::new(client),
            healthcheck_url,
            healthcheck_protocol: options.readiness_check_protocol,
            domain,
            base_path: options.base_path.clone(),
            async_init: options.async_init,
            ready_at_init: Arc::new(AtomicBool::new(false)),
            compression: options.compression,
            invoke_mode: options.invoke_mode,
        }
    }
}

impl Adapter<HttpConnector> {
    /// Create a new HTTP Adapter instance.
    /// This function initializes a new HTTP client
    /// to talk with the web server.
    pub fn new(options: &AdapterOptions) -> Adapter<HttpConnector> {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(4))
            .build(HttpConnector::new());

        let schema = "http";

        let healthcheck_url = format!(
            "{}://{}:{}{}",
            schema, options.host, options.readiness_check_port, options.readiness_check_path
        )
        .parse()
        .unwrap();

        let domain = format!("{}://{}:{}", schema, options.host, options.port)
            .parse()
            .unwrap();

        Adapter {
            client: Arc::new(client),
            healthcheck_url,
            healthcheck_protocol: options.readiness_check_protocol,
            domain,
            base_path: options.base_path.clone(),
            async_init: options.async_init,
            ready_at_init: Arc::new(AtomicBool::new(false)),
            compression: options.compression,
            invoke_mode: options.invoke_mode,
        }
    }
}

impl<C> Adapter<C>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    /// Register a Lambda Extension to ensure
    /// that the adapter is loaded before any Lambda function
    /// associated with it.
    pub fn register_default_extension(&self) {
        // register as an external extension
        tokio::task::spawn(async move {
            let aws_lambda_runtime_api: String =
                env::var("AWS_LAMBDA_RUNTIME_API").unwrap_or_else(|_| "127.0.0.1:9001".to_string());
            let client = hyper::Client::new();
            let register_req = hyper::Request::builder()
                .method(Method::POST)
                .uri(format!("http://{aws_lambda_runtime_api}/2020-01-01/extension/register"))
                .header("Lambda-Extension-Name", "lambda-adapter")
                .body(Body::from("{ \"events\": [] }"))
                .unwrap();
            let register_res = client.request(register_req).await.unwrap();
            if register_res.status() != StatusCode::OK {
                panic!("extension registration failure");
            }
            let next_req = hyper::Request::builder()
                .method(Method::GET)
                .uri(format!(
                    "http://{aws_lambda_runtime_api}/2020-01-01/extension/event/next"
                ))
                .header(
                    "Lambda-Extension-Identifier",
                    register_res.headers().get("Lambda-Extension-Identifier").unwrap(),
                )
                .body(Body::empty())
                .unwrap();
            client.request(next_req).await.unwrap();
        });
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
        let protocol = self.healthcheck_protocol;
        self.is_web_ready(&url, &protocol).await
    }

    async fn is_web_ready(&self, url: &Url, protocol: &Protocol) -> bool {
        Retry::spawn(FixedInterval::from_millis(10), || {
            self.check_web_readiness(url, protocol)
        })
        .await
        .is_ok()
    }

    async fn check_web_readiness(&self, url: &Url, protocol: &Protocol) -> Result<(), i8> {
        match protocol {
            Protocol::Http => match self.client.get(url.to_string().parse().unwrap()).await {
                Ok(response) if { 500 > response.status().as_u16() && response.status().as_u16() >= 100 } => Ok(()),
                _ => {
                    tracing::debug!("app is not ready");
                    Err(-1)
                }
            },
            Protocol::Tcp => match TcpStream::connect(format!("{}:{}", url.host().unwrap(), url.port().unwrap())).await
            {
                Ok(_) => Ok(()),
                Err(_) => Err(-1),
            },
        }
    }

    /// Run the adapter to take events from Lambda.
    pub async fn run(self) -> Result<(), Error> {
        let compression = self.compression;
        let invoke_mode = self.invoke_mode;

        if compression {
            let svc = ServiceBuilder::new().layer(CompressionLayer::new()).service(self);
            match invoke_mode {
                LambdaInvokeMode::Buffered => lambda_http::run(svc).await,
                LambdaInvokeMode::ResponseStream => lambda_http::run_with_streaming_response(svc).await,
            }
        } else {
            match invoke_mode {
                LambdaInvokeMode::Buffered => lambda_http::run(self).await,
                LambdaInvokeMode::ResponseStream => lambda_http::run_with_streaming_response(self).await,
            }
        }
    }

    async fn fetch_response(&self, event: Request) -> Result<Response<Body>, Error> {
        if self.async_init && !self.ready_at_init.load(Ordering::SeqCst) {
            self.is_web_ready(&self.healthcheck_url, &self.healthcheck_protocol)
                .await;
            self.ready_at_init.store(true, Ordering::SeqCst);
        }

        let request_context = event.request_context();
        let path = event.raw_http_path().to_string();
        let mut path = path.as_str();
        let (parts, body) = event.into_parts();

        // strip away Base Path if environment variable REMOVE_BASE_PATH is set.
        if let Some(base_path) = self.base_path.as_deref() {
            path = path.trim_start_matches(base_path);
        }

        let mut req_headers = parts.headers;

        // include request context in http header "x-amzn-request-context"
        req_headers.append(
            HeaderName::from_static("x-amzn-request-context"),
            HeaderValue::from_bytes(serde_json::to_string(&request_context)?.as_bytes())?,
        );

        let mut app_url = self.domain.clone();
        app_url.set_path(path);
        app_url.set_query(parts.uri.query());

        tracing::debug!(app_url = %app_url, req_headers = ?req_headers, "sending request to app server");

        let mut builder = hyper::Request::builder().method(parts.method).uri(app_url.to_string());
        if let Some(headers) = builder.headers_mut() {
            headers.extend(req_headers);
        }

        let request = builder.body(hyper::Body::from(body.to_vec()))?;

        let app_response = self.client.request(request).await?;
        tracing::debug!(status = %app_response.status(), body_size = app_response.body().size_hint().lower(),
            app_headers = ?app_response.headers().clone(), "responding to lambda event");

        Ok(app_response)
    }
}

/// Implement a `Tower.Service` that sends the requests
/// to the web server.
impl<C> Service<Request> for Adapter<C>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: Request) -> Self::Future {
        let adapter = self.clone();
        Box::pin(async move { adapter.fetch_response(event).await })
    }
}
