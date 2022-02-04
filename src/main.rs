// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use lambda_http::{
    handler,
    lambda_runtime::{self, Context},
    Body, IntoResponse, Request, RequestExt, Response,
};
use log::*;
use once_cell::sync::OnceCell;
use reqwest::{redirect, Client};
use serde_json::json;
use std::{env, mem, thread};
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;
use url::form_urlencoded::Serializer;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
static HTTP_CLIENT: OnceCell<Client> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // register as an external extension
    let aws_lambda_runtime_api = env::var("AWS_LAMBDA_RUNTIME_API").unwrap();
    let extension_next_url = format!("http://{}/2020-01-01/extension/event/next", aws_lambda_runtime_api);
    let extension_register_url = format!("http://{}/2020-01-01/extension/register", aws_lambda_runtime_api);
    let executable_name = env::current_exe().unwrap().file_name().unwrap().to_string_lossy().to_string();
    let client = reqwest::Client::new();
    let resp = client
        .post(extension_register_url)
        .header("Lambda-Extension-Name", executable_name)
        .json(&json!({"events": []}))
        .send()
        .await?;
    let extension_id = resp.headers().get("Lambda-Extension-Identifier").unwrap().clone();
    thread::spawn(move || {
        let extension_id_str = extension_id.to_str().unwrap();
        debug!("[extension] enter event loop for extension id: '{}'", extension_id_str);
        let client = reqwest::blocking::Client::new();
        client
            .get(extension_next_url)
            .header("Lambda-Extension-Identifier", extension_id_str)
            .send()
            .unwrap();
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
    lambda_runtime::run(handler(http_proxy_handler)).await?;
    Ok(())
}

async fn http_proxy_handler(event: Request, _: Context) -> Result<impl IntoResponse, Error> {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let app_host = format!("http://127.0.0.1:{}", port);
    let query_params = event.query_string_parameters();
    debug!("query_params are {:#?}", query_params);

    let (parts, body) = event.into_parts();
    let mut app_url = app_host + parts.uri.path();

    // append query parameters to app_url
    if !query_params.is_empty() {
        app_url.push('?');
        let mut serializer = Serializer::new(&mut app_url);
        for (key, _) in query_params.iter() {
            for value in query_params.get_all(key).unwrap().iter() {
                serializer.append_pair(key, value);
            }
        }
        serializer.finish();
    }
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
    let content_type;
    debug!("app response headers are {:#?}", app_response.headers());

    if app_response.headers().get(http::header::CONTENT_ENCODING).is_some() {
        debug!("body is binary");
        let content = app_response.bytes().await?;
        return Ok(Body::Binary(content.to_vec()));
    }

    if let Some(value) = app_response.headers().get(http::header::CONTENT_TYPE) {
        content_type = value.to_str().unwrap_or_default();
    } else {
        content_type = "";
    }
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
