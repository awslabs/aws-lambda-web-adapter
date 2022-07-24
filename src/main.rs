// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lambda_extension::{service_fn as extension_handler, Extension};
use lambda_http::{service_fn as http_handler, Body, Request, RequestExt, Response};
use log::*;
use reqwest::{redirect, Client};
use std::time::Duration;
use std::{env, future, mem};
use tokio::time::timeout;
use tokio_retry::{strategy::FixedInterval, Retry};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

struct AdapterOptions {
    host: String,
    port: String,
    readiness_check_port: String,
    readiness_check_path: String,
    base_path: Option<String>,
    async_init: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // setup config options from environment variables
    let options = &AdapterOptions {
        host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        port: env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
        readiness_check_port: env::var("READINESS_CHECK_PORT")
            .unwrap_or_else(|_| env::var("PORT").unwrap_or_else(|_| "8080".to_string())),
        readiness_check_path: env::var("READINESS_CHECK_PATH").unwrap_or_else(|_| "/".to_string()),
        base_path: env::var("REMOVE_BASE_PATH").ok(),
        async_init: env::var("ASYNC_INIT")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),
    };

    // register as an external extension
    tokio::task::spawn(async move {
        Extension::new()
            .with_events(&[])
            .with_events_processor(extension_handler(|_| async { Ok::<(), Error>(()) }))
            .run()
            .await
            .expect("extension thread error");
    });

    // check if the application is ready
    let is_ready = if options.async_init {
        timeout(Duration::from_secs_f32(9.8), check_readiness(options))
            .await
            .is_ok()
    } else {
        check_readiness(options).await
    };

    // start lambda runtime
    let http_client = &Client::builder()
        .redirect(redirect::Policy::none())
        .pool_idle_timeout(Duration::from_secs(4))
        .build()
        .unwrap();
    lambda_http::run(http_handler(|event: Request| async move {
        http_proxy_handler(event, http_client, options, is_ready).await
    }))
    .await?;
    Ok(())
}

async fn check_readiness(options: &AdapterOptions) -> bool {
    Retry::spawn(FixedInterval::from_millis(10), || {
        let readiness_check_url = format!(
            "http://{}:{}{}",
            options.host, options.readiness_check_port, options.readiness_check_path
        );
        match reqwest::blocking::get(readiness_check_url) {
            Ok(response) if { response.status().is_success() } => future::ready(Ok(())),
            _ => future::ready(Err::<(), i32>(-1)),
        }
    })
    .await
    .is_ok()
}

async fn http_proxy_handler(
    event: Request,
    http_client: &Client,
    options: &AdapterOptions,
    is_app_ready: bool,
) -> Result<Response<Body>, Error> {
    // continue checking readiness if async_init is configured and the app is not ready
    if options.async_init && !is_app_ready {
        check_readiness(options).await;
    }

    let raw_path = event.raw_http_path();
    let (parts, body) = event.into_parts();

    // construct the path and query without APIGW stage
    let mut path_and_query = match parts.uri.query() {
        Some(query) => format!("{}?{}", raw_path, query),
        None => raw_path,
    };

    // strip away Base Path if environment variable REMOVE_BASE_PATH is set.
    if options.base_path.is_some() {
        if let Some(value) = path_and_query.strip_prefix(options.base_path.as_ref().unwrap()) {
            path_and_query = value.to_string();
        };
    }
    let app_url = format!("http://{}:{}{}", options.host, options.port, path_and_query);
    debug!("app_url is {:#?}", app_url);

    let app_response = http_client
        .request(parts.method, app_url)
        .headers(parts.headers)
        .body(body.to_vec())
        .send()
        .await?;

    let mut lambda_response = Response::builder();
    let _ = mem::replace(lambda_response.headers_mut().unwrap(), app_response.headers().clone());
    Ok(lambda_response
        .status(app_response.status())
        .body(convert_body(app_response).await?)
        .expect("failed to send response"))
}

async fn convert_body(app_response: reqwest::Response) -> Result<Body, Error> {
    debug!("app response headers are {:#?}", app_response.headers());

    if app_response.headers().get(http::header::CONTENT_ENCODING).is_some() {
        let content = app_response.bytes().await?;
        return Ok(Body::Binary(content.to_vec()));
    }

    match app_response.headers().get(http::header::CONTENT_TYPE) {
        Some(value) => {
            let content_type = value.to_str().unwrap_or_default();

            if content_type.starts_with("text")
                || content_type.starts_with("application/json")
                || content_type.starts_with("application/javascript")
                || content_type.starts_with("application/xml")
            {
                Ok(Body::Text(app_response.text().await?))
            } else {
                let content = app_response.bytes().await?;
                if content.is_empty() {
                    Ok(Body::Empty)
                } else {
                    Ok(Body::Binary(content.to_vec()))
                }
            }
        }
        None => Ok(Body::Text(app_response.text().await?)),
    }
}
