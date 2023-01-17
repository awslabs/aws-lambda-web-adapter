use hyper::Client;
use hyper_tls::HttpsConnector;
use std::env;

#[ignore]
#[tokio::test]
async fn test_oci_rest_basic_request() {
    let oci_rest_endpoint = env::var("OCI_REST_ENDPOINT").unwrap();

    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let response = client.get((&oci_rest_endpoint).parse().unwrap()).await.unwrap();

    assert_eq!(200, response.status());
}
