// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

use lambda_web_adapter::{Adapter, AdapterOptions};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).without_time().init();

    let options = AdapterOptions::from_env();
    lambda_web_adapter::register_default_extension();

    let mut adapter = Adapter::new(&options);
    adapter.check_init_health().await;

    lambda_http::run(adapter).await
}
