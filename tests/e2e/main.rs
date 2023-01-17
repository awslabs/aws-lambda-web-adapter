use http::Uri;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use lambda_http::aws_lambda_events::serde_json;
use lambda_http::aws_lambda_events::serde_json::Value;
use std::env;

fn get_endpoints() -> Vec<Option<String>> {
    let configurations = [
        "OCI_REST_ENDPOINT",
        "OCI_HTTP_ENDPOINT",
        "OCI_ALB_ENDPOINT",
        // "OCI_FURL_ENDPOINT",
        "ZIP_REST_ENDPOINT",
        "ZIP_HTTP_ENDPOINT",
        "ZIP_ALB_ENDPOINT",
        // "ZIP_FURL_ENDPOINT",
    ];

    configurations.iter().map(|e| env::var(e).ok()).collect()
}

#[ignore]
#[tokio::test]
async fn test_http_basic_request() {
    for endpoint in get_endpoints().iter() {
        if let Some(endpoint) = endpoint {
            let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
            let response = client.get(endpoint.parse().unwrap()).await.unwrap();

            assert_eq!(200, response.status());
        }
    }
}

#[ignore]
#[tokio::test]
async fn test_http_headers() {
    for endpoint in get_endpoints().iter() {
        if let Some(endpoint) = endpoint {
            let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
            let uri = endpoint.to_string() + "get";
            let req = Request::builder()
                .method(Method::GET)
                .uri(uri)
                .header("Foo", "Bar")
                .body(Body::empty())
                .unwrap();
            let resp = client.request(req).await.unwrap();
            let (parts, body) = resp.into_parts();
            let body_bytes = hyper::body::to_bytes(body).await.unwrap();
            let body = serde_json::from_slice::<Value>(&*body_bytes).unwrap();

            assert_eq!(200, parts.status.as_u16());
            assert!(body["headers"]["Foo"][0].is_string());
            assert_eq!(Some("Bar"), body["headers"]["Foo"][0].as_str());
        }
    }
}

#[ignore]
#[tokio::test]
async fn test_http_query_params() {
    for endpoint in get_endpoints().iter() {
        if let Some(endpoint) = endpoint {
            let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
            let parts = endpoint.parse::<Uri>().unwrap().into_parts();
            let uri = Uri::builder()
                .scheme(parts.scheme.unwrap())
                .authority(parts.authority.unwrap())
                .path_and_query("/get?foo=bar&fizz=buzz")
                .build()
                .unwrap();
            let req = Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .unwrap();
            let resp = client.request(req).await.unwrap();
            let (parts, body) = resp.into_parts();
            let body_bytes = hyper::body::to_bytes(body).await.unwrap();
            let body = serde_json::from_slice::<Value>(&*body_bytes).unwrap();

            assert_eq!(200, parts.status.as_u16());
            assert!(body["args"]["fizz"][0].is_string());
            assert_eq!(Some("buzz"), body["args"]["fizz"][0].as_str());
            assert_eq!(Some("bar"), body["args"]["foo"][0].as_str());
        }
    }
}
