// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lambda_web_adapter::{Adapter, AdapterOptions, Error};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // setup tracing subscriber for logging
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).without_time().init();

    // get configuration options from environment variables
    let options = AdapterOptions::default();

    // create an adapter
    let mut adapter = Adapter::new(&options);
    // register the adapter as an extension
    adapter.register_default_extension();
    // check if the web application is ready
    adapter.check_init_health().await;
    // start lambda runtime after the web application is ready
    adapter.run().await.expect("lambda runtime failed");

    Ok(())
}
