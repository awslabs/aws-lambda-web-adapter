//! Common utilities for benchmarks

use std::collections::HashMap;
use std::str::FromStr;

use http::header::HeaderName;
use http::{HeaderMap, HeaderValue, Method};
use lambda_http::aws_lambda_events::alb::{AlbTargetGroupRequest, AlbTargetGroupRequestContext, ElbContext};
use lambda_http::aws_lambda_events::query_map::QueryMap;
use lambda_http::request::LambdaRequest;

/// Builder for creating Lambda events for benchmarking
#[derive(Debug, Default)]
pub struct LambdaEventBuilder {
    path: String,
    method: Method,
    query: HashMap<String, String>,
    headers: HeaderMap,
    body: Option<String>,
    is_base64_encoded: bool,
}

impl LambdaEventBuilder {
    pub fn new() -> Self {
        Self {
            path: "/".into(),
            method: Method::GET,
            query: HashMap::new(),
            headers: HeaderMap::new(),
            body: None,
            is_base64_encoded: false,
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.into();
        self
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        let value = HeaderValue::from_str(value).unwrap();
        let key = HeaderName::from_str(key).unwrap();
        self.headers.insert(key, value);
        self
    }

    /// Set a text body
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.into());
        self.is_base64_encoded = false;
        self
    }

    /// Set a pre-encoded base64 body (for benchmarks to avoid encoding overhead)
    pub fn with_base64_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self.is_base64_encoded = true;
        self
    }

    pub fn build(self) -> LambdaRequest {
        let mut elb_context = ElbContext::default();
        elb_context.target_group_arn = Some("arn:aws:us-east-1:123456789:elb/Foo".into());

        let mut request_context = AlbTargetGroupRequestContext::default();
        request_context.elb = elb_context;

        let mut alb_request = AlbTargetGroupRequest::default();
        alb_request.http_method = self.method;
        alb_request.path = Some(self.path);
        alb_request.query_string_parameters = QueryMap::from(self.query.clone());
        alb_request.multi_value_query_string_parameters = QueryMap::from(self.query);
        alb_request.headers = self.headers.clone();
        alb_request.multi_value_headers = self.headers;
        alb_request.is_base64_encoded = self.is_base64_encoded;
        alb_request.body = self.body;
        alb_request.request_context = request_context;

        LambdaRequest::Alb(alb_request)
    }
}
