pub mod common;

use std::env;
use std::io;
use std::io::prelude::*;
use std::sync::Arc;

use crate::common::LambdaEventBuilder;
use http::HeaderMap;
use http::Request;
use http::{Method, Response};
use httpmock::{
    Method::{DELETE, GET, POST, PUT},
    MockServer,
};
use hyper::body::Incoming;
use lambda_http::Body;
use lambda_http::Context;
use lambda_web_adapter::{Adapter, AdapterOptions, LambdaInvokeMode, Protocol};
use tower::{Service, ServiceBuilder};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use http_body_util::BodyExt;
use lambda_http::lambda_runtime::Config;
use serde_json::json;
use tower_http::compression::{CompressionBody, CompressionLayer};

#[test]
fn test_adapter_options_from_env() {
    env::set_var("PORT", "3000");
    env::set_var("HOST", "localhost");
    env::set_var("READINESS_CHECK_PORT", "8000");
    env::set_var("READINESS_CHECK_PROTOCOL", "TCP");
    env::set_var("READINESS_CHECK_PATH", "/healthcheck");
    env::set_var("REMOVE_BASE_PATH", "/prod");
    env::set_var("ASYNC_INIT", "true");
    env::set_var("AWS_LWA_ENABLE_COMPRESSION", "true");
    env::set_var("AWS_LWA_ENABLE_TLS", "true");
    env::set_var("AWS_LWA_TLS_SERVER_NAME", "api.example.com");
    env::remove_var("AWS_LWA_TLS_CERT_FILE");
    env::set_var("AWS_LWA_INVOKE_MODE", "buffered");
    env::set_var("AWS_LWA_AUTHORIZATION_SOURCE", "auth-token");

    // Initialize adapter with env options
    let options = AdapterOptions::default();
    Adapter::new(&options);

    assert_eq!("3000", options.port);
    assert_eq!("localhost", options.host);
    assert_eq!("8000", options.readiness_check_port);
    assert_eq!("/healthcheck", options.readiness_check_path);
    assert_eq!(Protocol::Tcp, options.readiness_check_protocol);
    assert_eq!(Some("/prod".into()), options.base_path);
    assert!(options.async_init);
    assert!(options.compression);
    assert_eq!(LambdaInvokeMode::Buffered, options.invoke_mode);
    assert_eq!(Some("auth-token".into()), options.authorization_source);
}

#[test]
fn test_adapter_options_from_namespaced_env() {
    env::set_var("AWS_LWA_PORT", "3000");
    env::set_var("AWS_LWA_HOST", "localhost");
    env::set_var("AWS_LWA_READINESS_CHECK_MIN_UNHEALTHY_STATUS", "400");
    env::set_var("AWS_LWA_READINESS_CHECK_PORT", "8000");
    env::set_var("AWS_LWA_READINESS_CHECK_PROTOCOL", "TCP");
    env::set_var("AWS_LWA_READINESS_CHECK_PATH", "/healthcheck");
    env::set_var("AWS_LWA_REMOVE_BASE_PATH", "/prod");
    env::set_var("AWS_LWA_ASYNC_INIT", "true");
    env::set_var("AWS_LWA_ENABLE_COMPRESSION", "true");
    env::set_var("AWS_LWA_INVOKE_MODE", "response_stream");
    env::set_var("AWS_LWA_AUTHORIZATION_SOURCE", "auth-token");

    // Initialize adapter with env options
    let options = AdapterOptions::default();
    Adapter::new(&options);

    assert_eq!("3000", options.port);
    assert_eq!(400, options.readiness_check_min_unhealthy_status);
    assert_eq!("localhost", options.host);
    assert_eq!("8000", options.readiness_check_port);
    assert_eq!("/healthcheck", options.readiness_check_path);
    assert_eq!(Protocol::Tcp, options.readiness_check_protocol);
    assert_eq!(Some("/prod".into()), options.base_path);
    assert!(options.async_init);
    assert!(options.compression);
    assert_eq!(LambdaInvokeMode::ResponseStream, options.invoke_mode);
    assert_eq!(Some("auth-token".into()), options.authorization_source);
}

#[test]
fn test_readiness_check_port_fallback_to_lwa_port() {
    env::remove_var("AWS_LWA_READINESS_CHECK_PORT");
    env::remove_var("READINESS_CHECK_PORT");
    env::set_var("AWS_LWA_PORT", "3000");

    // Initialize adapter with env options
    let options = AdapterOptions::default();
    Adapter::new(&options);

    assert_eq!("3000", options.readiness_check_port);
}

#[tokio::test]
async fn test_http_readiness_check() {
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
    let mut adapter = Adapter::new(&options);
    adapter.check_init_health().await;

    // Assert app server's healthcheck endpoint got called
    healthcheck.assert();
}

#[tokio::test]
async fn test_http_basic_request() {
    // Start app server
    let app_server = MockServer::start();
    let hello = app_server.mock(|when, then| {
        when.method(GET).path("/hello");
        then.status(200).body("Hello World");
    });

    // Initialize adapter
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new().with_path("/hello").build();

    // We convert to Request object because it allows us to add
    // the lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    hello.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!(response.headers().get("content-length").unwrap(), "11");
    assert_eq!("Hello World", body_to_string(response).await);
}

#[tokio::test]
async fn test_http_headers() {
    // Start app server
    let app_server = MockServer::start();

    // An endpoint that expects and returns headers
    let test_endpoint = app_server.mock(|when, then| {
        when.method(GET).path("/").header("foo", "bar");
        then.status(200).header("fizz", "buzz").body("OK");
    });

    // Initialize adapter and do readiness check
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // Prepare request
    let req = LambdaEventBuilder::new()
        .with_path("/")
        .with_header("foo", "bar")
        .build();

    // We convert to Request object because it allows us to add
    // the Lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    // Call the adapter service with request
    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    test_endpoint.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert!(response.headers().contains_key("fizz"));
    assert_eq!("buzz", response.headers().get("fizz").unwrap());
    assert_eq!("OK", body_to_string(response).await);
}

#[tokio::test]
async fn test_http_path_encoding() {
    // Start app server
    let app_server = MockServer::start();

    // An endpoint that expects and returns headers
    let test_endpoint = app_server.mock(|when, then| {
        when.method(GET).path("/A%C3%B1o_1234");
        then.status(200).body("Ok");
    });

    // Initialize adapter and do readiness check
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // Prepare request
    let req = LambdaEventBuilder::new().with_path("/Año_1234").build();

    // We convert to Request object because it allows us to add
    // the lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    // Call the adapter service with request
    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    test_endpoint.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!("Ok", body_to_string(response).await);
}

#[tokio::test]
async fn test_http_query_params() {
    // Start app server
    let app_server = MockServer::start();

    // An endpoint that expects and returns headers
    let test_endpoint = app_server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .query_param("foo", "bar")
            .query_param("fizz", "buzz");
        then.status(200).body("OK");
    });

    // Initialize adapter
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // Prepare request
    let req = LambdaEventBuilder::new()
        .with_path("/")
        .with_query("foo", "bar")
        .with_query("fizz", "buzz")
        .build();

    // We convert to Request object because it allows us to add
    // the lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    // Call the adapter service with request
    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    test_endpoint.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!("OK", body_to_string(response).await);
}

#[tokio::test]
async fn test_http_post_put_delete() {
    // Start app server
    let app_server = MockServer::start();
    let post_endpoint = app_server.mock(|when, then| {
        when.path("/").method(POST);
        then.status(200).body("POST Success");
    });

    let put_endpoint = app_server.mock(|when, then| {
        when.path("/").method(PUT);
        then.status(200).body("PUT Success");
    });

    let delete_endpoint = app_server.mock(|when, then| {
        when.path("/").method(DELETE);
        then.status(200).body("DELETE Success");
    });

    // Initialize adapter
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // Prepare requests
    let post_req = LambdaEventBuilder::new()
        .with_method(Method::POST)
        .with_path("/")
        .build();

    let put_req = LambdaEventBuilder::new()
        .with_method(Method::PUT)
        .with_path("/")
        .build();

    let delete_req = LambdaEventBuilder::new()
        .with_method(Method::DELETE)
        .with_path("/")
        .build();
    // We convert to Request object because it allows us to add the lambda Context
    let mut post_request = Request::from(post_req);
    add_lambda_context_to_request(&mut post_request);

    let mut put_request = Request::from(put_req);
    add_lambda_context_to_request(&mut put_request);

    let mut delete_request = Request::from(delete_req);
    add_lambda_context_to_request(&mut delete_request);

    // Call the adapter service with requests
    let post_response = adapter.call(post_request).await.expect("Request failed");
    let put_response = adapter.call(put_request).await.expect("Request failed");
    let delete_response = adapter.call(delete_request).await.expect("Request failed");

    // Assert endpoints were called
    post_endpoint.assert();
    put_endpoint.assert();
    delete_endpoint.assert();

    assert_eq!(200, post_response.status());
    assert_eq!(200, put_response.status());
    assert_eq!(200, delete_response.status());
    assert_eq!("POST Success", body_to_string(post_response).await);
    assert_eq!("PUT Success", body_to_string(put_response).await);
    assert_eq!("DELETE Success", body_to_string(delete_response).await);
}

#[tokio::test]
async fn test_http_compress() {
    // Start app server
    let app_server = MockServer::start();
    let hello = app_server.mock(|when, then| {
        when.method(GET).path("/hello");
        then.status(200)
            .header("content-type", "text/plain")
            .body("Hello World Hello World Hello World Hello World Hello World");
    });

    // Initialize adapter
    let adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        compression: true,
        ..Default::default()
    });

    let mut svc = ServiceBuilder::new().layer(CompressionLayer::new()).service(adapter);

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new()
        .with_path("/hello")
        .with_header("accept-encoding", "gzip")
        .build();

    // We convert to Request object because it allows us to add
    // the lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    let response = svc.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    hello.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!(response.headers().get("content-encoding").unwrap(), "gzip");
    assert_eq!(
        "Hello World Hello World Hello World Hello World Hello World",
        compressed_body_to_string(response).await
    );
}

#[tokio::test]
async fn test_http_compress_disallowed_type() {
    // Start app server
    let app_server = MockServer::start();
    let hello = app_server.mock(|when, then| {
        when.method(GET).path("/hello");
        then.status(200)
            .header("content-type", "application/octet-stream")
            .body("Hello World Hello World Hello World Hello World Hello World");
    });

    // Initialize adapter
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        compression: true,
        ..Default::default()
    });

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new()
        .with_path("/hello")
        .with_header("accept-encoding", "gzip")
        .build();

    // We convert to Request object because it allows us to add
    // the lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    hello.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!(response.headers().get("content-length").unwrap(), "59"); // uncompressed: 59
    assert!(!response.headers().contains_key("content-encoding"));
    assert_eq!(
        "Hello World Hello World Hello World Hello World Hello World",
        body_to_string(response).await
    );
}

#[tokio::test]
async fn test_http_compress_already_compressed() {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(b"Hello World Hello World Hello World Hello World Hello World")
        .unwrap();
    let gzipped_body = encoder.finish().unwrap();

    // Start app server
    let app_server = MockServer::start();
    let hello = app_server.mock(|when, then| {
        when.method(GET).path("/hello");
        then.status(200).header("content-encoding", "gzip").body(&gzipped_body);
    });

    // Initialize adapter
    let adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        compression: true,
        ..Default::default()
    });

    let mut svc = ServiceBuilder::new().layer(CompressionLayer::new()).service(adapter);

    // Call the adapter service with basic request
    let req = LambdaEventBuilder::new()
        .with_path("/hello")
        .with_header("accept-encoding", "gzip")
        .build();

    // We convert to Request object because it allows us to add
    // the lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    let response = svc.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    hello.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!(response.headers().get("content-length").unwrap(), "48"); // uncompressed: 59
    assert_eq!(response.headers().get("content-encoding").unwrap(), "gzip");
    assert_eq!(
        "Hello World Hello World Hello World Hello World Hello World",
        compressed_body_to_string(response).await
    );
}

#[tokio::test]
async fn test_http_context_headers() {
    // Start app server
    let app_server = MockServer::start();

    // An endpoint that expects and returns headers
    let test_endpoint = app_server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .header_exists("x-amzn-lambda-context")
            .header_exists("x-amzn-request-context");
        then.status(200).header("fizz", "buzz").body("OK");
    });

    // Initialize adapter and do readiness check
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // Prepare request
    let req = LambdaEventBuilder::new().with_path("/").build();

    // We convert to Request object because it allows us to add
    // the Lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    // Call the adapter service with request
    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    test_endpoint.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert!(response.headers().contains_key("fizz"));
    assert_eq!("buzz", response.headers().get("fizz").unwrap());
    assert_eq!("OK", body_to_string(response).await);
}

#[tokio::test]
async fn test_http_content_encoding_suffix() {
    // Start app server
    let app_server = MockServer::start();

    let json_data = json!({
        "name": "John Doe",
        "age": 43,
        "gender": "Male"
    })
    .to_string();

    // An endpoint that expects and returns headers
    let test_endpoint = app_server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/graphql-response+json; charset=utf-8")
            .body(json_data.to_owned());
    });

    // Initialize adapter and do readiness check
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // Prepare request
    let req = LambdaEventBuilder::new().with_path("/json").build();

    // We convert to Request object because it allows us to add
    // the Lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    // Call the adapter service with request
    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    test_endpoint.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert!(response.headers().contains_key("content-type"));
    assert_eq!(
        "application/graphql-response+json; charset=utf-8",
        response.headers().get("content-type").unwrap()
    );
    assert_eq!(json_data.to_owned(), body_to_string(response).await);
}

#[tokio::test]
async fn test_http_error_status_codes() {
    // Start app server
    let app_server = MockServer::start();
    let error_endpoint = app_server.mock(|when, then| {
        when.method(GET).path("/error");
        then.status(502).body("Bad Gateway");
    });

    // Initialize adapter with error status codes
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        error_status_codes: Some(vec![500, 502, 503, 504]),
        ..Default::default()
    });

    // Call the adapter service with request that should trigger error
    let req = LambdaEventBuilder::new().with_path("/error").build();
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    let result = adapter.call(request).await;
    assert!(result.is_err(), "Expected error response for status code 502");
    assert!(result.unwrap_err().to_string().contains("502"));

    // Assert endpoint was called
    error_endpoint.assert();
}

#[tokio::test]
async fn test_http_authorization_source() {
    // Start app server
    let app_server = MockServer::start();
    let hello = app_server.mock(|when, then| {
        when.method(GET).path("/hello").header_exists("Authorization");
        then.status(200).body("Hello World");
    });

    // Initialize adapter
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        authorization_source: Some("auth-token".to_string()),
        ..Default::default()
    });

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new()
        .with_path("/hello")
        .with_header("auth-token", "Bearer token")
        .build();

    // We convert to Request object because it allows us to add
    // the lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    hello.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!(response.headers().get("content-length").unwrap(), "11");
    assert_eq!("Hello World", body_to_string(response).await);
}

#[tokio::test]
async fn test_http_context_multi_headers() {
    // Start app server
    let app_server = MockServer::start();

    // An endpoint that expects and returns headers
    let test_endpoint = app_server.mock(|when, then| {
        when.method(GET)
            .path("/")
            .matches(|req| {
                req.headers
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|(key, _value)| key == "x-amzn-lambda-context")
                    .count()
                    == 1
            })
            .matches(|req| {
                req.headers
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|(key, _value)| key == "x-amzn-request-context")
                    .count()
                    == 1
            });
        then.status(200).header("fizz", "buzz").body("OK");
    });

    // Initialize adapter and do readiness check
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        ..Default::default()
    });

    // Prepare request
    let req = LambdaEventBuilder::new()
        .with_path("/")
        .with_header("x-amzn-lambda-context", "header_from_client_1")
        .with_header("x-amzn-lambda-context", "header_from_client_2")
        .with_header("x-amzn-request-context", "header_from_client_1")
        .with_header("x-amzn-request-context", "header_from_client_2")
        .build();

    // We convert to Request object because it allows us to add
    // the Lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    // Call the adapter service with request
    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    test_endpoint.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert!(response.headers().contains_key("fizz"));
    assert_eq!("buzz", response.headers().get("fizz").unwrap());
    assert_eq!("OK", body_to_string(response).await);
}

#[tokio::test]
async fn test_http_strip_response_headers() {
    // Start app server
    let app_server = MockServer::start();

    // An endpoint that returns multiple headers
    let test_endpoint = app_server.mock(|when, then| {
        when.method(GET).path("/");
        then.status(200)
            .header("x-custom-header", "value")
            .header("server", "test-server")
            .header("content-type", "application/json")
            .body("{}");
    });

    // Initialize adapter with headers to strip
    let mut adapter = Adapter::new(&AdapterOptions {
        host: app_server.host(),
        port: app_server.port().to_string(),
        readiness_check_port: app_server.port().to_string(),
        readiness_check_path: "/healthcheck".to_string(),
        strip_response_headers: Some(vec!["x-custom-header".to_string(), "server".to_string()]),
        ..Default::default()
    });

    // Prepare request
    let req = LambdaEventBuilder::new().with_path("/").build();

    // Convert to Request object and add Lambda Context
    let mut request = Request::from(req);
    add_lambda_context_to_request(&mut request);

    // Call the adapter service with request
    let response = adapter.call(request).await.expect("Request failed");

    // Assert endpoint was called once
    test_endpoint.assert();

    // Verify headers were stripped
    assert!(
        !response.headers().contains_key("x-custom-header"),
        "x-custom-header should be stripped"
    );
    assert!(
        !response.headers().contains_key("server"),
        "server header should be stripped"
    );

    // Verify other headers remain
    assert!(
        response.headers().contains_key("content-type"),
        "content-type header should remain"
    );
    assert_eq!("{}", body_to_string(response).await);
}

async fn body_to_string(res: Response<Incoming>) -> String {
    let body_bytes = res.collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&body_bytes).to_string()
}

async fn compressed_body_to_string(res: Response<CompressionBody<Incoming>>) -> String {
    let body_bytes = res.collect().await.unwrap().to_bytes();
    decode_reader(&body_bytes).unwrap()
}

fn decode_reader(bytes: &[u8]) -> io::Result<String> {
    let mut gz = GzDecoder::new(bytes);
    let mut s = String::new();
    gz.read_to_string(&mut s)?;
    Ok(s)
}

fn add_lambda_context_to_request(request: &mut Request<Body>) {
    // create a HeaderMap to build the lambda context
    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", "my_id".parse().unwrap());
    headers.insert("lambda-runtime-deadline-ms", "123".parse().unwrap());
    headers.insert("lambda-runtime-client-context", "{}".parse().unwrap());

    let conf = Config {
        function_name: "test_function".into(),
        memory: 128,
        version: "latest".into(),
        log_stream: "/aws/lambda/test_function".into(),
        log_group: "2023/09/15/[$LATEST]ab831cef03e94457a94b6efcbe22406a".into(),
    };

    // converts HeaderMap to Context
    let context = Context::new("my_id", Arc::new(conf), &headers).expect("Couldn't convert HeaderMap to Context");

    // add Context to the request
    request.extensions_mut().insert(context);
}
