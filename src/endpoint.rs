use crate::error::WebError;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use futures::future::{Future, FutureExt};
use http::request::Parts;
use http::StatusCode;
use http_service::{Request, Response};
use std::error::Error;
use std::pin::Pin;

pub(crate) type AsyncResponse = Pin<Box<dyn Future<Output = Response> + Send>>;

pub struct Endpoint;

impl Endpoint {
    pub fn new<Req, Res, F>(f: fn(Req) -> F) -> impl Fn(Request) -> AsyncResponse
    where
        Req: for<'de> Deserialize<'de> + Send + 'static,
        Res: Serialize + 'static,
        F: Future<Output = Result<Res, WebError>> + Send + 'static,
    {
        move |req: Request| {
            let fut = async move {
                let body = req.into_body();

                // Wait to receive the entire body
                match body.into_vec().await {
                    // Convert the body into json from bytes
                    Ok(body_bytes) => match serde_json::from_slice(&body_bytes) {
                        Ok(req) => {
                            // Await the evaluation of the endpoint handler
                            match f(req).await {
                                Ok(res) => success_response(res),
                                Err(e) => error_response(e.msg, e.code),
                            }
                        }
                        // There was an error converting the json from bytes
                        Err(e) => error_response(e.description(), StatusCode::BAD_REQUEST),
                    },
                    // There was an error receiving the body
                    Err(e) => error_response(e.description(), StatusCode::INTERNAL_SERVER_ERROR),
                }
            };

            fut.boxed()
        }
    }

    pub fn new_with_policy<Req, Res, Policy, F, P>(
        f: fn(Policy, Req) -> F,
        p: fn(Parts) -> P,
    ) -> impl Fn(Request) -> AsyncResponse
    where
        Req: for<'de> Deserialize<'de> + Send + 'static,
        Res: Serialize + 'static,
        Policy: Send + 'static,
        F: Future<Output = Result<Res, WebError>> + Send + 'static,
        P: Future<Output = Result<Policy, WebError>> + Send + 'static,
    {
        move |req: Request| {
            let fut = async move {
                let (parts, body) = req.into_parts();

                // Await the evaluation of the policy handler
                let policy = match p(parts).await {
                    Ok(policy) => policy,
                    Err(e) => return error_response(e.msg, e.code),
                };

                // Wait to receive the entire body
                match body.into_vec().await {
                    // Convert the body into json from bytes
                    Ok(body_bytes) => match serde_json::from_slice(&body_bytes) {
                        Ok(req) => {
                            // Await the evaluation of the endpoint handler
                            match f(policy, req).await {
                                Ok(res) => success_response(res),
                                Err(e) => error_response(e.msg, e.code),
                            }
                        }
                        // There was an error converting the json from bytes
                        Err(e) => error_response(e.description(), StatusCode::BAD_REQUEST),
                    },
                    // There was an error receiving the body
                    Err(e) => error_response(e.description(), StatusCode::INTERNAL_SERVER_ERROR),
                }
            };

            fut.boxed()
        }
    }
}

fn error_response(msg: impl Serialize, code: http::StatusCode) -> Response {
    let json_vec = serde_json::to_vec(&msg).unwrap();
    let mut res = Response::new(json_vec.into());
    *res.status_mut() = code;
    res
}

fn success_response(msg: impl Serialize) -> Response {
    let json_vec = serde_json::to_vec(&msg).unwrap();
    let mut res = Response::new(json_vec.into());
    *res.status_mut() = StatusCode::OK;
    res
}
