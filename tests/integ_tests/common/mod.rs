use std::collections::HashMap;
use std::str::FromStr;

use http::header::HeaderName;
use http::{HeaderMap, HeaderValue, Method};
use lambda_http::aws_lambda_events::alb::{AlbTargetGroupRequest, AlbTargetGroupRequestContext, ElbContext};
use lambda_http::aws_lambda_events::query_map::QueryMap;
use lambda_http::request::LambdaRequest;

#[derive(Debug)]
pub enum LambdaEventType {
    ALB,
    // TODO: Add other event types
}

impl Default for LambdaEventType {
    fn default() -> Self {
        LambdaEventType::ALB
    }
}

/// Makes it easy to construct events for testing purposes.
///
/// Example:
///
/// ```rs
/// LambdaEventBuilder::new()
///  .with_method(http::Method::POST)
///  .with_path("/hello")
///  .with_query("foo", "bar")
///  .with_query("fizz", "buzz")
///  .with_header("ContentType", "application/json")
///  .build()
///
///  let req: http::Request<Body> = req.into();
// ```
#[derive(Debug, Default)]
pub struct LambdaEventBuilder {
    path: String,
    method: Method,
    query: HashMap<String, String>,
    headers: HeaderMap,
    event_type: LambdaEventType,
    body: Option<String>,
}

impl LambdaEventBuilder {
    pub fn new() -> Self {
        Self {
            path: "/".into(),
            method: Method::GET,
            query: HashMap::new(),
            headers: HeaderMap::new(),
            event_type: LambdaEventType::ALB,
            body: None,
        }
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.into();
        self
    }

    pub fn with_query(mut self, key: &str, value: &str) -> Self {
        self.query.insert(key.into(), value.into());
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

    pub fn with_event_type(mut self, event_type: LambdaEventType) -> Self {
        self.event_type = event_type;
        self
    }

    pub fn build(self) -> LambdaRequest {
        match self.event_type {
            LambdaEventType::ALB => LambdaRequest::Alb({
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
                alb_request.is_base64_encoded = false;
                alb_request.body = self.body;
                alb_request.request_context = request_context;
                alb_request
            }),
        }
    }
}
