// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

use lambda_web_adapter::{Adapter, AdapterOptions};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let options = AdapterOptions::from_env();
    lambda_web_adapter::register_default_extension();

    let mut adapter = Adapter::new(&options);
    adapter.check_init_health().await;

    lambda_http::run(adapter).await
}
