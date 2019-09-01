use crate::error::WebError;

use futures::future::{Future, FutureExt};
use http::{method::Method, request::Parts, StatusCode};
use http_service::{Request, Response};
use serde::{Deserialize, Serialize};
use std::{error::Error, pin::Pin};

pub(crate) type AsyncResponse = Pin<Box<dyn Future<Output = Response> + Send>>;

pub struct Endpoint;

impl Endpoint {
    pub fn new<Req, Res, Context, F, C>(
        f: fn(Context, Req) -> F,
        c: fn(Parts) -> C,
    ) -> impl Fn(Request) -> AsyncResponse
    where
        Req: for<'de> Deserialize<'de> + Send + 'static + Default,
        Res: Serialize + 'static,
        Context: Send + 'static,
        F: Future<Output = Result<Res, WebError>> + Send + 'static,
        C: Future<Output = Result<Context, WebError>> + Send + 'static,
    {
        move |req: Request| {
            let fut = async move {
                let (parts, body) = req.into_parts();

                let has_body = parts.method != Method::GET;

                // Await the evaluation of the context
                let context = match c(parts).await {
                    Ok(cx) => cx,
                    Err(e) => return error_response(e.msg, e.code),
                };

                // Wait to receive the body bytes
                let body_bytes = match body.into_vec().await {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return error_response(e.description(), StatusCode::INTERNAL_SERVER_ERROR)
                    }
                };

                // Parse the body as json if the request has a body
                let req = if has_body {
                    match serde_json::from_slice(&body_bytes) {
                        Ok(req) => req,
                        Err(e) => {
                            return error_response(format!("{}", e), StatusCode::BAD_REQUEST);
                        }
                    }
                } else {
                    Req::default()
                };

                // Await the evaluation of the endpoint handler
                match f(context, req).await {
                    Ok(res) => success_response(res),
                    Err(e) => error_response(e.msg, e.code),
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
