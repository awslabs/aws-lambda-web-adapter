// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    borrow::Cow,
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

use encoding_rs::{Encoding, UTF_8};
use http::{
    header::{HeaderName, HeaderValue},
    Method, StatusCode, Uri,
};
use hyper::{
    body::HttpBody,
    client::{Client, HttpConnector},
    Body,
};
use lambda_http::aws_lambda_events::serde_json;
pub use lambda_http::Error;
use lambda_http::{Body as LambdaBody, Request, RequestExt, Response};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_retry::{strategy::FixedInterval, Retry};
use tower::Service;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    Http,
    Tcp,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Http
    }
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

#[derive(Default)]
pub struct AdapterOptions {
    pub host: String,
    pub port: String,
    pub readiness_check_port: String,
    pub readiness_check_path: String,
    pub readiness_check_protocol: Protocol,
    pub base_path: Option<String>,
    pub async_init: bool,
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
        }
    }
}

pub struct Adapter {
    client: Arc<Client<HttpConnector>>,
    healthcheck_url: Uri,
    healthcheck_protocol: Protocol,
    async_init: bool,
    ready_at_init: Arc<AtomicBool>,
    domain: Uri,
    base_path: Option<String>,
}

impl Adapter {
    /// Create a new Adapter instance.
    /// This function initializes a new HTTP client
    /// to talk with the web server.
    pub fn new(options: &AdapterOptions) -> Adapter {
        let client = Client::builder().pool_idle_timeout(Duration::from_secs(4)).build_http();

        let healthcheck_url = format!(
            "http://{}:{}{}",
            options.host, options.readiness_check_port, options.readiness_check_path
        )
        .parse()
        .unwrap();

        let domain = format!("http://{}:{}", options.host, options.port).parse().unwrap();

        Adapter {
            client: Arc::new(client),
            healthcheck_url,
            healthcheck_protocol: options.readiness_check_protocol,
            domain,
            base_path: options.base_path.clone(),
            async_init: options.async_init,
            ready_at_init: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Switch the default HTTP client with a different one.
    pub fn with_client(self, client: Client<HttpConnector>) -> Self {
        Adapter {
            client: Arc::new(client),
            ..self
        }
    }

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
        is_web_ready(&url, &protocol).await
    }

    /// Run the adapter to take events from Lambda.
    pub async fn run(self) -> Result<(), Error> {
        lambda_http::run(self).await
    }
}

/// Implement a `Tower.Service` that sends the requests
/// to the web server.
impl Service<Request> for Adapter {
    type Response = Response<LambdaBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: Request) -> Self::Future {
        let async_init = self.async_init;
        let client = self.client.clone();
        let ready_at_init = self.ready_at_init.clone();
        let healthcheck_url = self.healthcheck_url.clone();
        let healthcheck_protocol = self.healthcheck_protocol;
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
                healthcheck_protocol,
                event,
            )
            .await
        })
    }
}

#[allow(clippy::too_many_arguments)]
async fn fetch_response(
    async_init: bool,
    ready_at_init: Arc<AtomicBool>,
    client: Arc<Client<HttpConnector>>,
    base_path: Option<String>,
    domain: Uri,
    healthcheck_url: Uri,
    healthcheck_protocol: Protocol,
    event: Request,
) -> Result<Response<LambdaBody>, Error> {
    if async_init && !ready_at_init.load(Ordering::SeqCst) {
        is_web_ready(&healthcheck_url, &healthcheck_protocol).await;
        ready_at_init.store(true, Ordering::SeqCst);
    }

    let request_context = event.request_context();
    let path = event.raw_http_path();
    let mut path = path.as_str();
    let (parts, body) = event.into_parts();

    // strip away Base Path if environment variable REMOVE_BASE_PATH is set.
    if let Some(base_path) = base_path.as_deref() {
        path = path.trim_start_matches(base_path);
    }

    let mut req_headers = parts.headers;

    // include request context in http header "x-amzn-request-context"
    req_headers.append(
        HeaderName::from_static("x-amzn-request-context"),
        HeaderValue::from_bytes(serde_json::to_string(&request_context)?.as_bytes())?,
    );

    let mut pq = path.to_string();
    if let Some(q) = parts.uri.query() {
        pq.push('?');
        pq.push_str(q);
    }

    let mut app_parts = domain.into_parts();
    app_parts.path_and_query = Some(pq.parse()?);
    let app_url = Uri::from_parts(app_parts)?;

    tracing::debug!(app_url = %app_url, req_headers = ?req_headers, "sending request to app server");

    let mut builder = hyper::Request::builder().method(parts.method).uri(app_url);
    if let Some(headers) = builder.headers_mut() {
        headers.extend(req_headers);
    }

    let request = builder.body(hyper::Body::from(body.to_vec()))?;

    let app_response = client.request(request).await?;
    let app_headers = app_response.headers().clone();
    let status = app_response.status();

    let body = convert_body(app_response).await?;
    tracing::debug!(status = %status, body_size = body.size_hint().lower(), app_headers = ?app_headers, "responding to lambda event");

    let mut lambda_response = Response::builder();
    if let Some(headers) = lambda_response.headers_mut() {
        let _ = mem::replace(headers, app_headers);
    }
    let resp = lambda_response.status(status).body(body).map_err(Box::new)?;

    Ok(resp)
}

async fn is_web_ready(url: &Uri, protocol: &Protocol) -> bool {
    Retry::spawn(FixedInterval::from_millis(10), || check_web_readiness(url, protocol))
        .await
        .is_ok()
}

async fn check_web_readiness(url: &Uri, protocol: &Protocol) -> Result<(), i8> {
    match protocol {
        Protocol::Http => match Client::new().get(url.clone()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(-1),
        },
        Protocol::Tcp => match TcpStream::connect(format!("{}:{}", url.host().unwrap(), url.port().unwrap())).await {
            Ok(_) => Ok(()),
            Err(_) => Err(-1),
        },
    }
}

async fn convert_body(app_response: hyper::Response<hyper::Body>) -> Result<LambdaBody, Error> {
    let (parts, body) = app_response.into_parts();
    let body = hyper::body::to_bytes(body).await?;

    if parts.headers.get(http::header::CONTENT_ENCODING).is_some() {
        return if body.is_empty() {
            Ok(LambdaBody::Empty)
        } else {
            Ok(LambdaBody::Binary(body.to_vec()))
        };
    }

    let content_type = parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());

    if serialize_as_text(content_type) {
        let content_type = content_type.and_then(|value| value.parse::<mime::Mime>().ok());

        let encoding_name = content_type
            .as_ref()
            .and_then(|mime| mime.get_param("charset").map(|charset| charset.as_str()))
            .unwrap_or("utf-8");
        let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);

        let (text, _, _) = encoding.decode(&body);
        match text {
            Cow::Owned(s) => Ok(LambdaBody::Text(s)),
            _ => unsafe {
                // these bytes are already valid utf8. This is the same operation that reqwest performs
                Ok(LambdaBody::Text(String::from_utf8_unchecked(body.to_vec())))
            },
        }
    } else if body.is_empty() {
        Ok(LambdaBody::Empty)
    } else {
        Ok(LambdaBody::Binary(body.to_vec()))
    }
}

fn serialize_as_text(content_type: Option<&str>) -> bool {
    match content_type {
        None => true,
        Some(content_type) => {
            content_type.starts_with("text")
                || content_type.starts_with("application/json")
                || content_type.starts_with("application/javascript")
                || content_type.starts_with("application/xml")
        }
    }
}
