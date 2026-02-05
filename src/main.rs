// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lambda_web_adapter::{Adapter, AdapterOptions, Error};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

fn main() -> Result<(), Error> {
    // Apply runtime proxy configuration BEFORE starting tokio runtime
    // This must happen before any threads are spawned to avoid unsafe env::set_var
    Adapter::apply_runtime_proxy_config();

    // Start tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}

async fn async_main() -> Result<(), Error> {
    // setup tracing subscriber for logging
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).without_time().init();

    // get configuration options from environment variables
    let options = AdapterOptions::default();

    // create an adapter
    let mut adapter = Adapter::new(&options)?;
    // register the adapter as an extension
    adapter.register_default_extension();
    // check if the web application is ready
    adapter.check_init_health().await;
    // start lambda runtime after the web application is ready
    adapter.run().await?;

    Ok(())
}
