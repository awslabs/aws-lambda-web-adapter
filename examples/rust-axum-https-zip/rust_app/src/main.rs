use axum::{routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let config = RustlsConfig::from_pem_file("./certs/cert.pem", "./certs/key.pem")
        .await
        .unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8443));
    println!("listening on {addr}");
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
