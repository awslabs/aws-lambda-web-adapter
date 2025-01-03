// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

mod readiness;

use http::{
    header::{HeaderName, HeaderValue},
    Method, StatusCode,
};
use http_body::Body as HttpBody;
use hyper::body::Incoming;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use lambda_http::request::RequestContext;
use lambda_http::Body;
pub use lambda_http::Error;
use lambda_http::{Request, RequestExt, Response};
use readiness::Checkpoint;
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
use std::{fmt::Debug, time::SystemTime};
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

pub struct AdapterOptions {
    pub host: String,
    pub port: String,
    pub readiness_check_port: String,
    pub readiness_check_path: String,
    pub readiness_check_protocol: Protocol,
    pub readiness_check_min_unhealthy_status: u16,
    pub base_path: Option<String>,
    pub pass_through_path: String,
    pub async_init: bool,
    pub compression: bool,
    pub invoke_mode: LambdaInvokeMode,
    pub authorization_source: Option<String>,
    pub error_status_codes: Option<Vec<u16>>,
    pub client_idle_timeout_ms: u64,
}

impl Default for AdapterOptions {
    fn default() -> Self {
        AdapterOptions {
            host: env::var("AWS_LWA_HOST").unwrap_or(env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string())),
            port: env::var("AWS_LWA_PORT").unwrap_or(env::var("PORT").unwrap_or_else(|_| "8080".to_string())),
            readiness_check_port: env::var("AWS_LWA_READINESS_CHECK_PORT").unwrap_or(
                env::var("READINESS_CHECK_PORT").unwrap_or(
                    env::var("AWS_LWA_PORT")
                        .unwrap_or_else(|_| env::var("PORT").unwrap_or_else(|_| "8080".to_string())),
                ),
            ),
            readiness_check_min_unhealthy_status: env::var("AWS_LWA_READINESS_CHECK_MIN_UNHEALTHY_STATUS")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .unwrap_or(500),
            readiness_check_path: env::var("AWS_LWA_READINESS_CHECK_PATH")
                .unwrap_or(env::var("READINESS_CHECK_PATH").unwrap_or_else(|_| "/".to_string())),
            readiness_check_protocol: env::var("AWS_LWA_READINESS_CHECK_PROTOCOL")
                .unwrap_or(env::var("READINESS_CHECK_PROTOCOL").unwrap_or_else(|_| "HTTP".to_string()))
                .as_str()
                .into(),
            base_path: env::var("AWS_LWA_REMOVE_BASE_PATH").map_or_else(|_| env::var("REMOVE_BASE_PATH").ok(), Some),
            pass_through_path: env::var("AWS_LWA_PASS_THROUGH_PATH").unwrap_or_else(|_| "/events".to_string()),
            async_init: env::var("AWS_LWA_ASYNC_INIT")
                .unwrap_or(env::var("ASYNC_INIT").unwrap_or_else(|_| "false".to_string()))
                .parse()
                .unwrap_or(false),
            compression: env::var("AWS_LWA_ENABLE_COMPRESSION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            invoke_mode: env::var("AWS_LWA_INVOKE_MODE")
                .unwrap_or("buffered".to_string())
                .as_str()
                .into(),
            authorization_source: env::var("AWS_LWA_AUTHORIZATION_SOURCE").ok(),
            error_status_codes: env::var("AWS_LWA_ERROR_STATUS_CODES")
                .ok()
                .map(|codes| parse_status_codes(&codes)),
            client_idle_timeout_ms: env::var("AWS_LWA_CLIENT_IDLE_TIMEOUT_MS")
                .ok()
                .map(|s| s.parse().unwrap())
                .unwrap_or(4000),
        }
    }
}

fn parse_status_codes(input: &str) -> Vec<u16> {
    input
        .split(',')
        .flat_map(|part| {
            let part = part.trim();
            if part.contains('-') {
                let range: Vec<&str> = part.split('-').collect();
                if range.len() == 2 {
                    if let (Ok(start), Ok(end)) = (range[0].parse::<u16>(), range[1].parse::<u16>()) {
                        return (start..=end).collect::<Vec<_>>();
                    }
                }
                tracing::warn!("Failed to parse status code range: {}", part);
                vec![]
            } else {
                part.parse::<u16>().map_or_else(
                    |_| {
                        if !part.is_empty() {
                            tracing::warn!("Failed to parse status code: {}", part);
                        }
                        vec![]
                    },
                    |code| vec![code],
                )
            }
        })
        .collect()
}

#[derive(Clone)]
pub struct Adapter<C, B> {
    client: Arc<Client<C, B>>,
    healthcheck_url: Url,
    healthcheck_protocol: Protocol,
    healthcheck_min_unhealthy_status: u16,
    async_init: bool,
    ready_at_init: Arc<AtomicBool>,
    domain: Url,
    base_path: Option<String>,
    path_through_path: String,
    compression: bool,
    invoke_mode: LambdaInvokeMode,
    authorization_source: Option<String>,
    error_status_codes: Option<Vec<u16>>,
    client_idle_timeout_ms: u64,
    // be sure to use `SystemTime` (CLOCK_REALTIME) instead of `Duration` (CLOCK_MONOTONIC)
    // to avoid issues when restored from Lambda SnapStart
    last_invoke: SystemTime,
}

impl Adapter<HttpConnector, Body> {
    fn new_client(timeout_ms: u64) -> Arc<Client<HttpConnector, Body>> {
        Arc::new(
            Client::builder(hyper_util::rt::TokioExecutor::new())
                .pool_idle_timeout(Duration::from_millis(timeout_ms))
                .build(HttpConnector::new()),
        )
    }

    /// Create a new HTTP Adapter instance.
    /// This function initializes a new HTTP client
    /// to talk with the web server.
    pub fn new(options: &AdapterOptions) -> Adapter<HttpConnector, Body> {
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
            client: Self::new_client(options.client_idle_timeout_ms),
            healthcheck_url,
            healthcheck_protocol: options.readiness_check_protocol,
            healthcheck_min_unhealthy_status: options.readiness_check_min_unhealthy_status,
            domain,
            base_path: options.base_path.clone(),
            path_through_path: options.pass_through_path.clone(),
            async_init: options.async_init,
            ready_at_init: Arc::new(AtomicBool::new(false)),
            compression: options.compression,
            invoke_mode: options.invoke_mode,
            authorization_source: options.authorization_source.clone(),
            error_status_codes: options.error_status_codes.clone(),
            client_idle_timeout_ms: options.client_idle_timeout_ms,
            // it's ok to use `now` here since there is no connections in the connection pool yet
            last_invoke: SystemTime::now(),
        }
    }
}

impl Adapter<HttpConnector, Body> {
    /// Register a Lambda Extension to ensure
    /// that the adapter is loaded before any Lambda function
    /// associated with it.
    pub fn register_default_extension(&self) {
        // register as an external extension
        tokio::task::spawn(async move {
            let aws_lambda_runtime_api: String =
                env::var("AWS_LAMBDA_RUNTIME_API").unwrap_or_else(|_| "127.0.0.1:9001".to_string());
            let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpConnector::new());
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
                .body(Body::Empty)
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
        let mut checkpoint = Checkpoint::new();
        Retry::spawn(FixedInterval::from_millis(10), || {
            if checkpoint.lapsed() {
                tracing::info!(url = %url.to_string(), "app is not ready after {}ms", checkpoint.next_ms());
                checkpoint.increment();
            }
            self.check_web_readiness(url, protocol)
        })
        .await
        .is_ok()
    }

    async fn check_web_readiness(&self, url: &Url, protocol: &Protocol) -> Result<(), i8> {
        match protocol {
            Protocol::Http => match self.client.get(url.to_string().parse().unwrap()).await {
                Ok(response)
                    if {
                        self.healthcheck_min_unhealthy_status > response.status().as_u16()
                            && response.status().as_u16() >= 100
                    } =>
                {
                    tracing::debug!("app is ready");
                    Ok(())
                }
                _ => {
                    tracing::trace!("app is not ready");
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

    async fn fetch_response(&self, event: Request) -> Result<Response<Incoming>, Error> {
        if self.async_init && !self.ready_at_init.load(Ordering::SeqCst) {
            self.is_web_ready(&self.healthcheck_url, &self.healthcheck_protocol)
                .await;
            self.ready_at_init.store(true, Ordering::SeqCst);
        }

        let request_context = event.request_context();
        let lambda_context = event.lambda_context();
        let path = event.raw_http_path().to_string();
        let mut path = path.as_str();
        let (parts, body) = event.into_parts();

        // strip away Base Path if environment variable REMOVE_BASE_PATH is set.
        if let Some(base_path) = self.base_path.as_deref() {
            path = path.trim_start_matches(base_path);
        }

        if matches!(request_context, RequestContext::PassThrough) && parts.method == Method::POST {
            path = self.path_through_path.as_str();
        }

        let mut req_headers = parts.headers;

        // include request context in http header "x-amzn-request-context"
        req_headers.insert(
            HeaderName::from_static("x-amzn-request-context"),
            HeaderValue::from_bytes(serde_json::to_string(&request_context)?.as_bytes())?,
        );

        // include lambda context in http header "x-amzn-lambda-context"
        req_headers.insert(
            HeaderName::from_static("x-amzn-lambda-context"),
            HeaderValue::from_bytes(serde_json::to_string(&lambda_context)?.as_bytes())?,
        );

        if let Some(authorization_source) = self.authorization_source.as_deref() {
            if req_headers.contains_key(authorization_source) {
                let original = req_headers.remove(authorization_source).unwrap();
                req_headers.insert("authorization", original);
            } else {
                tracing::warn!("\"{}\" header not found in request headers", authorization_source);
            }
        }

        let mut app_url = self.domain.clone();
        app_url.set_path(path);
        app_url.set_query(parts.uri.query());

        tracing::debug!(app_url = %app_url, req_headers = ?req_headers, "sending request to app server");

        let mut builder = hyper::Request::builder().method(parts.method).uri(app_url.to_string());
        if let Some(headers) = builder.headers_mut() {
            headers.extend(req_headers);
        }

        let request = builder.body(Body::from(body.to_vec()))?;

        let mut app_response = self.client.request(request).await?;

        // Check if status code should trigger an error
        if let Some(error_codes) = &self.error_status_codes {
            let status = app_response.status().as_u16();
            if error_codes.contains(&status) {
                return Err(Error::from(format!(
                    "Request failed with configured error status code: {}",
                    status
                )));
            }
        }

        // remove "transfer-encoding" from the response to support "sam local start-api"
        app_response.headers_mut().remove("transfer-encoding");

        tracing::debug!(status = %app_response.status(), body_size = ?app_response.body().size_hint().lower(),
            app_headers = ?app_response.headers().clone(), "responding to lambda event");

        Ok(app_response)
    }

    /// Return whether the client has been idle for longer than the [`Self::client_idle_timeout_ms`].
    fn client_timeout_has_expired(&self) -> bool {
        self.last_invoke
            .elapsed()
            .map(|d| d.as_millis() > self.client_idle_timeout_ms.into())
            // if the last_invoke is in the future, it's ok to re-use the client
            .unwrap_or(false)
    }
}

/// Implement a `Tower.Service` that sends the requests
/// to the web server.
impl Service<Request> for Adapter<HttpConnector, Body> {
    type Response = Response<Incoming>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: Request) -> Self::Future {
        if self.client_timeout_has_expired() {
            // client timeout, create a new client with a new connection pool.
            // this is to prevent the pool from using a to-be-disconnected connection after restoring from Lambda SnapStart
            tracing::debug!("Client timeout, creating a new client");
            self.client = Self::new_client(self.client_idle_timeout_ms);
        }

        let adapter = self.clone();
        self.last_invoke = SystemTime::now();
        Box::pin(async move { adapter.fetch_response(event).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};

    #[test]
    fn test_parse_status_codes() {
        assert_eq!(parse_status_codes("500,502-504,422"), vec![500, 502, 503, 504, 422]);
        assert_eq!(
            parse_status_codes("500, 502-504, 422"), // with spaces
            vec![500, 502, 503, 504, 422]
        );
        assert_eq!(parse_status_codes("500"), vec![500]);
        assert_eq!(parse_status_codes("500-502"), vec![500, 501, 502]);
        assert_eq!(parse_status_codes("invalid"), Vec::<u16>::new());
        assert_eq!(parse_status_codes("500-invalid"), Vec::<u16>::new());
        assert_eq!(parse_status_codes(""), Vec::<u16>::new());
    }

    #[tokio::test]
    async fn test_status_200_is_ok() {
        // Start app server
        let app_server = MockServer::start();
        let healthcheck = app_server.mock(|when, then| {
            when.method(GET).path("/healthcheck");
            then.status(200).body("OK");
        });

        // Prepare adapter configuration
        let options = AdapterOptions {
            host: app_server.host(),
            port: app_server.port().to_string(),
            readiness_check_port: app_server.port().to_string(),
            readiness_check_path: "/healthcheck".to_string(),
            ..Default::default()
        };

        // Initialize adapter and do readiness check
        let adapter = Adapter::new(&options);

        let url = adapter.healthcheck_url.clone();
        let protocol = adapter.healthcheck_protocol;

        //adapter.check_init_health().await;

        assert!(adapter.check_web_readiness(&url, &protocol).await.is_ok());

        // Assert app server's healthcheck endpoint got called
        healthcheck.assert();
    }

    #[tokio::test]
    async fn test_status_500_is_bad() {
        // Start app server
        let app_server = MockServer::start();
        let healthcheck = app_server.mock(|when, then| {
            when.method(GET).path("/healthcheck");
            then.status(500).body("OK");
        });

        // Prepare adapter configuration
        let options = AdapterOptions {
            host: app_server.host(),
            port: app_server.port().to_string(),
            readiness_check_port: app_server.port().to_string(),
            readiness_check_path: "/healthcheck".to_string(),
            ..Default::default()
        };

        // Initialize adapter and do readiness check
        let adapter = Adapter::new(&options);

        let url = adapter.healthcheck_url.clone();
        let protocol = adapter.healthcheck_protocol;

        //adapter.check_init_health().await;

        assert!(adapter.check_web_readiness(&url, &protocol).await.is_err());

        // Assert app server's healthcheck endpoint got called
        healthcheck.assert();
    }

    #[tokio::test]
    async fn test_status_403_is_bad_when_configured() {
        // Start app server
        let app_server = MockServer::start();
        let healthcheck = app_server.mock(|when, then| {
            when.method(GET).path("/healthcheck");
            then.status(403).body("OK");
        });

        // Prepare adapter configuration
        let options = AdapterOptions {
            host: app_server.host(),
            port: app_server.port().to_string(),
            readiness_check_port: app_server.port().to_string(),
            readiness_check_path: "/healthcheck".to_string(),
            readiness_check_min_unhealthy_status: 400,
            ..Default::default()
        };

        // Initialize adapter and do readiness check
        let adapter = Adapter::new(&options);

        let url = adapter.healthcheck_url.clone();
        let protocol = adapter.healthcheck_protocol;

        //adapter.check_init_health().await;

        assert!(adapter.check_web_readiness(&url, &protocol).await.is_err());

        // Assert app server's healthcheck endpoint got called
        healthcheck.assert();
    }

    #[test]
    fn test_client_idle_timeout() {
        let mut adapter = Adapter::new(&AdapterOptions::default());
        assert!(!adapter.client_timeout_has_expired());

        adapter.last_invoke = SystemTime::now() - Duration::from_millis(5000);
        assert!(adapter.client_timeout_has_expired());

        adapter.last_invoke = SystemTime::now() + Duration::from_millis(5000);
        assert!(!adapter.client_timeout_has_expired());
    }
}
