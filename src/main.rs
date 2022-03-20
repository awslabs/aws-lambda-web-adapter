// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lambda_extension::{service_fn as extension_handler, Extension};
use lambda_http::{service_fn as http_handler, Body, IntoResponse, Request, Response};
use log::*;
use once_cell::sync::OnceCell;
use reqwest::{redirect, Client};
use std::{env, mem};
use tokio::runtime::Handle;
use tokio_retry::{strategy::FixedInterval, Retry};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
static HTTP_CLIENT: OnceCell<Client> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // register as an external extension
    let handle = Handle::current();
    tokio::task::spawn_blocking(move || {
        handle.spawn(async {
            Extension::new()
                .with_events(&[])
                .with_events_processor(extension_handler(|_| async { Ok::<(), Error>(()) }))
                .run()
                .await
                .expect("extension thread error");
        })
    });

    // check if the application is ready every 10 milliseconds
    Retry::spawn(FixedInterval::from_millis(10), || {
        let readiness_check_url = format!(
            "http://127.0.0.1:{}{}",
            env::var("READINESS_CHECK_PORT").unwrap_or_else(|_| "8080".to_string()),
            env::var("READINESS_CHECK_PATH").unwrap_or_else(|_| "/".to_string())
        );
        reqwest::get(readiness_check_url)
    })
    .await?;

    // start lambda runtime
    HTTP_CLIENT.set(Client::builder().redirect(redirect::Policy::none()).build().unwrap()).unwrap();
    lambda_http::run(http_handler(http_proxy_handler)).await?;
    Ok(())
}

async fn http_proxy_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let app_host = format!("http://127.0.0.1:{}", port);
    let (parts, body) = event.into_parts();
    let app_url = app_host + parts.uri.path_and_query().unwrap().as_str();
    debug!("app_url is {:#?}", app_url);
    debug!("request headers are {:#?}", parts.headers);

    let app_response = HTTP_CLIENT
        .get()
        .unwrap()
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
        debug!("body is binary");
        let content = app_response.bytes().await?;
        return Ok(Body::Binary(content.to_vec()));
    }

    let content_type = if let Some(value) = app_response.headers().get(http::header::CONTENT_TYPE) {
        value.to_str().unwrap_or_default()
    } else {
        ""
    };
    debug!("content_type is {:?}", content_type);

    if content_type.starts_with("text")
        || content_type.starts_with("application/json")
        || content_type.starts_with("application/javascript")
        || content_type.starts_with("application/xml")
    {
        debug!("body is text");
        let body_text = app_response.text().await?;
        return Ok(Body::Text(body_text));
    }
    let content = app_response.bytes().await?;
    return if !content.is_empty() {
        debug!("body is binary");
        Ok(Body::Binary(content.to_vec()))
    } else {
        debug! {"body is empty"};
        Ok(Body::Empty)
    };
}
