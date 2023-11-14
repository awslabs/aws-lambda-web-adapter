use aws_sigv4::http_request::{sign, SignableBody, SignableRequest, SigningParams, SigningSettings};
use flate2::read::GzDecoder;
use http::Uri;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Method, Request};
use hyper_rustls::HttpsConnector;
use std::env;
use std::io;
use std::io::prelude::*;
use std::time::SystemTime;

#[derive(PartialEq)]
enum AuthType {
    Open,
    Iam,
}

impl From<&str> for AuthType {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "iam" => AuthType::Iam,
            _ => AuthType::Open,
        }
    }
}

struct TestConfig {
    pub endpoint: Uri,
    pub auth_type: AuthType,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            endpoint: env::var("API_ENDPOINT").unwrap().parse().unwrap(),
            auth_type: env::var("API_AUTH_TYPE").unwrap().as_str().into(),
        }
    }
}

fn decode_reader(bytes: &[u8]) -> io::Result<String> {
    let mut gz = GzDecoder::new(bytes);
    let mut s = String::new();
    gz.read_to_string(&mut s)?;
    Ok(s)
}

fn get_https_connector() -> HttpsConnector<HttpConnector> {
    hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build()
}

fn signing_request(conf: TestConfig, req: &mut Request<&str>) {
    if conf.auth_type == AuthType::Iam {
        let access_key_id = env::var("AWS_ACCESS_KEY_ID").unwrap();
        let secret_key = env::var("AWS_SECRET_ACCESS_KEY").unwrap();
        let session_token = env::var("AWS_SESSION_TOKEN").unwrap();
        let region = env::var("AWS_DEFAULT_REGION").unwrap();
        // Set up information and settings for the signing
        let signing_settings = SigningSettings::default();
        let signing_params = SigningParams::builder()
            .access_key(access_key_id.as_str())
            .secret_key(secret_key.as_str())
            .security_token(session_token.as_str())
            .region(region.as_str())
            .service_name("lambda")
            .time(SystemTime::now())
            .settings(signing_settings)
            .build()
            .unwrap();

        // Convert the HTTP request into a signable request
        let signable_request = SignableRequest::new(
            req.method(),
            req.uri(),
            req.headers(),
            SignableBody::Bytes(req.body().as_bytes()),
        );

        // Sign and then apply the signature to the request
        let (signing_instructions, _signature) = sign(signable_request, &signing_params).unwrap().into_parts();
        signing_instructions.apply_to_request(req);
    }
}

#[ignore]
#[tokio::test]
async fn test_http_basic_request() {
    let conf = TestConfig::default();
    let client = Client::builder().build::<_, Body>(get_https_connector());
    let uri = conf.endpoint.clone();
    let mut req = http::Request::builder().method(Method::GET).uri(uri).body("").unwrap();
    signing_request(conf, &mut req);
    let response = client.request(req.map(Body::from)).await.unwrap();

    assert_eq!(200, response.status());
}

#[ignore]
#[tokio::test]
async fn test_http_headers() {
    let conf = TestConfig::default();
    let client = Client::builder().build::<_, Body>(get_https_connector());
    let uri = conf.endpoint.to_string() + "get";
    let mut req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Foo", "Bar")
        .body("")
        .unwrap();
    signing_request(conf, &mut req);
    let resp = client.request(req.map(Body::from)).await.unwrap();
    let (parts, body) = resp.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap();
    let body = serde_json::from_slice::<serde_json::Value>(&body_bytes).unwrap();

    assert_eq!(200, parts.status.as_u16());
    assert!(body["headers"]["Foo"][0].is_string());
    assert_eq!(Some("Bar"), body["headers"]["Foo"][0].as_str());
}

#[ignore]
#[tokio::test]
async fn test_http_query_params() {
    let conf = TestConfig::default();
    let client = Client::builder().build::<_, Body>(get_https_connector());
    let parts = conf.endpoint.clone().into_parts();
    let uri = Uri::builder()
        .scheme(parts.scheme.unwrap())
        .authority(parts.authority.unwrap())
        .path_and_query("/get?foo=bar&fizz=buzz")
        .build()
        .unwrap();
    let mut req = Request::builder().method(Method::GET).uri(uri).body("").unwrap();
    signing_request(conf, &mut req);
    let resp = client.request(req.map(Body::from)).await.unwrap();
    let (parts, body) = resp.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap();
    let body = serde_json::from_slice::<serde_json::Value>(&body_bytes).unwrap();

    assert_eq!(200, parts.status.as_u16());
    assert!(body["args"]["fizz"][0].is_string());
    assert_eq!(Some("buzz"), body["args"]["fizz"][0].as_str());
    assert_eq!(Some("bar"), body["args"]["foo"][0].as_str());
}

#[ignore]
#[tokio::test]
async fn test_http_compress() {
    let conf = TestConfig::default();
    let client = Client::builder().build::<_, Body>(get_https_connector());
    let parts = conf.endpoint.clone().into_parts();
    let uri = Uri::builder()
        .scheme(parts.scheme.unwrap())
        .authority(parts.authority.unwrap())
        .path_and_query("/html")
        .build()
        .unwrap();
    let mut req = Request::builder()
        .method(Method::GET)
        .header("accept-encoding", "gzip")
        .uri(uri)
        .body("")
        .unwrap();
    signing_request(conf, &mut req);
    let resp = client.request(req.map(Body::from)).await.unwrap();
    let (parts, body) = resp.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap();
    let body = decode_reader(&body_bytes).unwrap();

    assert_eq!(200, parts.status.as_u16());
    assert!(body.contains("<html>"));
}
