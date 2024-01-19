mod util;

use aws_credential_types::Credentials;
use aws_sigv4::http_request::{sign, SignableBody, SignableRequest, SigningParams, SigningSettings};
use aws_sigv4::sign::v4;
use flate2::read::GzDecoder;
use http::{Method, Request, Uri};
use http_body_util::BodyExt;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use lambda_http::Body;
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
        .unwrap()
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
        let identity = Credentials::new(access_key_id, secret_key, Some(session_token), None, "temp").into();
        let signing_settings = SigningSettings::default();
        let signing_params = v4::SigningParams::builder()
            .identity(&identity)
            .region(region.as_str())
            .name("lambda")
            .time(SystemTime::now())
            .settings(signing_settings)
            .build()
            .unwrap();

        // Convert the HTTP request into a signable request
        let signable_request = SignableRequest::new(
            req.method().as_str(),
            req.uri().to_string(),
            req.headers().iter().map(|(k, v)| (k.as_str(), v.to_str().unwrap())),
            SignableBody::Bytes(req.body().as_bytes()),
        )
        .unwrap();

        // Sign and then apply the signature to the request
        let (signing_instructions, _signature) = sign(signable_request, &SigningParams::from(signing_params))
            .unwrap()
            .into_parts();
        // aws-sigv4 still on http 0.4. This is a workaround until it is updated to http 1.0
        util::apply_to_request_http0x(signing_instructions, req);
    }
}

#[ignore]
#[tokio::test]
async fn test_http_basic_request() {
    let conf = TestConfig::default();
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build::<_, Body>(get_https_connector());
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
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build::<_, Body>(get_https_connector());
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
    let body_bytes = body.collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice::<serde_json::Value>(&body_bytes).unwrap();

    assert_eq!(200, parts.status.as_u16());
    assert!(body["headers"]["Foo"][0].is_string());
    assert_eq!(Some("Bar"), body["headers"]["Foo"][0].as_str());
}

#[ignore]
#[tokio::test]
async fn test_http_query_params() {
    let conf = TestConfig::default();
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build::<_, Body>(get_https_connector());
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
    let body_bytes = body.collect().await.unwrap().to_bytes();
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
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build::<_, Body>(get_https_connector());
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
    let body_bytes = body.collect().await.unwrap().to_bytes();
    let body = decode_reader(&body_bytes).unwrap();

    assert_eq!(200, parts.status.as_u16());
    assert!(body.contains("<html>"));
}
