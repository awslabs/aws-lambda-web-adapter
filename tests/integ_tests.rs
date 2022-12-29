pub mod events;

use std::env;

use crate::events::LambdaEventBuilder;
use http::{Method, Response};
use httpmock::{
    Method::{DELETE, GET, POST, PUT},
    MockServer,
};
use lambda_extension::Service;

use lambda_http::Body;
use lambda_web_adapter::{Adapter, AdapterOptions, Protocol};

#[test]
fn test_adapter_options_from_env() {
    env::set_var("PORT", "3000");
    env::set_var("HOST", "localhost");
    env::set_var("READINESS_CHECK_PORT", "8000");
    env::set_var("READINESS_CHECK_PROTOCOL", "TCP");
    env::set_var("READINESS_CHECK_PATH", "/healthcheck");
    env::set_var("REMOVE_BASE_PATH", "/prod");
    env::set_var("ASYNC_INIT", "true");

    // Initialize adapter with env options
    let options = AdapterOptions::from_env();
    Adapter::new(&options);

    assert_eq!("3000", options.port);
    assert_eq!("localhost", options.host);
    assert_eq!("8000", options.readiness_check_port);
    assert_eq!("/healthcheck", options.readiness_check_path);
    assert_eq!(Protocol::Tcp, options.readiness_check_protocol);
    assert_eq!(Some("/prod".into()), options.base_path);
    assert_eq!(true, options.async_init);
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
    });

    // // Call the adapter service with basic request
    let req = LambdaEventBuilder::new().with_path("/hello").build();
    let response = adapter.call(req.into()).await.expect("Request failed");

    // Assert endpoint was called once
    hello.assert();

    // and response has expected content
    assert_eq!(200, response.status());
    assert_eq!("Hello World", body_to_string(&response));
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
    assert_eq!("OK", body_to_string(&response));
    assert_eq!(true, response.headers().contains_key("fizz"));
    assert_eq!("buzz", response.headers().get("fizz").unwrap());
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
    assert_eq!("OK", body_to_string(&response));
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
    assert_eq!("POST Success", body_to_string(&post_response));
    assert_eq!("PUT Success", body_to_string(&put_response));
    assert_eq!("DELETE Success", body_to_string(&delete_response));
}

fn body_to_string(res: &Response<Body>) -> String {
    String::from_utf8(res.body().to_vec()).unwrap()
}
