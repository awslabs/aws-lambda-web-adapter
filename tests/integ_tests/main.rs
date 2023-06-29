pub mod common;

use std::env;
use std::io;
use std::io::prelude::*;

use crate::common::LambdaEventBuilder;
use http::{Method, Response};
use httpmock::{
    Method::{DELETE, GET, POST, PUT},
    MockServer,
};
use hyper::{body, Body};
use lambda_web_adapter::{Adapter, AdapterOptions, LambdaInvokeMode, Protocol};
use tower::{Service, ServiceBuilder};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
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

    // Initialize adapter with env options
    let options = AdapterOptions::from_env();
    Adapter::new(&options);

    assert_eq!("3000", options.port);
    assert_eq!("localhost", options.host);
    assert_eq!("8000", options.readiness_check_port);
    assert_eq!("/healthcheck", options.readiness_check_path);
    assert_eq!(Protocol::Tcp, options.readiness_check_protocol);
    assert_eq!(Some("/prod".into()), options.base_path);
    assert!(options.async_init);
    assert!(options.compression);
    assert!(options.enable_tls);
    assert_eq!(Some("api.example.com".into()), options.tls_server_name);
    assert_eq!(None, options.tls_cert_file);
    assert_eq!(LambdaInvokeMode::Buffered, options.invoke_mode);
}

#[test]
fn test_adapter_options_from_namespaced_env() {
    env::set_var("AWS_LWA_PORT", "3000");
    env::set_var("AWS_LWA_HOST", "localhost");
    env::set_var("AWS_LWA_READINESS_CHECK_PORT", "8000");
    env::set_var("AWS_LWA_READINESS_CHECK_PROTOCOL", "TCP");
    env::set_var("AWS_LWA_READINESS_CHECK_PATH", "/healthcheck");
    env::set_var("AWS_LWA_REMOVE_BASE_PATH", "/prod");
    env::set_var("AWS_LWA_ASYNC_INIT", "true");
    env::set_var("AWS_LWA_ENABLE_COMPRESSION", "true");
    env::set_var("AWS_LWA_ENABLE_TLS", "true");
    env::set_var("AWS_LWA_TLS_SERVER_NAME", "api.example.com");
    env::remove_var("AWS_LWA_TLS_CERT_FILE");
    env::set_var("AWS_LWA_INVOKE_MODE", "response_stream");

    // Initialize adapter with env options
    let options = AdapterOptions::from_env();
    Adapter::new(&options);

    assert_eq!("3000", options.port);
    assert_eq!("localhost", options.host);
    assert_eq!("8000", options.readiness_check_port);
    assert_eq!("/healthcheck", options.readiness_check_path);
    assert_eq!(Protocol::Tcp, options.readiness_check_protocol);
    assert_eq!(Some("/prod".into()), options.base_path);
    assert!(options.async_init);
    assert!(options.compression);
    assert!(options.enable_tls);
    assert_eq!(Some("api.example.com".into()), options.tls_server_name);
    assert_eq!(None, options.tls_cert_file);
    assert_eq!(LambdaInvokeMode::ResponseStream, options.invoke_mode);
}

#[test]
fn test_readiness_check_port_fallback_to_lwa_port() {
    env::remove_var("AWS_LWA_READINESS_CHECK_PORT");
    env::remove_var("READINESS_CHECK_PORT");
    env::set_var("AWS_LWA_PORT", "3000");

    // Initialize adapter with env options
    let options = AdapterOptions::from_env();
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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: false,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: false,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
    });

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new().with_path("/hello").build();
    let response = adapter.call(req.into()).await.expect("Request failed");

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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: false,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
    });

    // Prepare request
    let req = LambdaEventBuilder::new()
        .with_path("/")
        .with_header("foo", "bar")
        .build();

    // Call the adapter service with request
    let response = adapter.call(req.into()).await.expect("Request failed");

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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: false,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
    });

    // Prepare request
    let req = LambdaEventBuilder::new().with_path("/Año_1234").build();

    // Call the adapter service with request
    let response = adapter.call(req.into()).await.expect("Request failed");

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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: false,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
    });

    // Prepare request
    let req = LambdaEventBuilder::new()
        .with_path("/")
        .with_query("foo", "bar")
        .with_query("fizz", "buzz")
        .build();

    // Call the adapter service with request
    let response = adapter.call(req.into()).await.expect("Request failed");

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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: false,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
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

    // Call the adapter service with requests
    let post_response = adapter.call(post_req.into()).await.expect("Request failed");
    let put_response = adapter.call(put_req.into()).await.expect("Request failed");
    let delete_response = adapter.call(delete_req.into()).await.expect("Request failed");

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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: true,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
    });

    let mut svc = ServiceBuilder::new().layer(CompressionLayer::new()).service(adapter);

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new()
        .with_path("/hello")
        .with_header("accept-encoding", "gzip")
        .build();
    let response = svc.call(req.into()).await.expect("Request failed");

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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: true,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
    });

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new()
        .with_path("/hello")
        .with_header("accept-encoding", "gzip")
        .build();
    let response = adapter.call(req.into()).await.expect("Request failed");

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
        readiness_check_protocol: Protocol::Http,
        async_init: false,
        base_path: None,
        compression: true,
        enable_tls: false,
        tls_server_name: None,
        tls_cert_file: None,
        invoke_mode: LambdaInvokeMode::Buffered,
    });

    let mut svc = ServiceBuilder::new().layer(CompressionLayer::new()).service(adapter);

    // Call the adapter service with basic request
    let req = LambdaEventBuilder::new()
        .with_path("/hello")
        .with_header("accept-encoding", "gzip")
        .build();
    let response = svc.call(req.into()).await.expect("Request failed");

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

async fn body_to_string(res: Response<Body>) -> String {
    let body_bytes = body::to_bytes(res.into_body()).await.unwrap();
    String::from_utf8_lossy(&body_bytes).to_string()
}

async fn compressed_body_to_string(res: Response<CompressionBody<Body>>) -> String {
    let body_bytes = body::to_bytes(res.into_body()).await.unwrap();
    decode_reader(&body_bytes).unwrap()
}

fn decode_reader(bytes: &[u8]) -> io::Result<String> {
    let mut gz = GzDecoder::new(bytes);
    let mut s = String::new();
    gz.read_to_string(&mut s)?;
    Ok(s)
}
