use crate::{context::Context, params::Params, result::WebResult};

use http::StatusCode;
use http_service::{Request, Response};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::{error::Error, pin::Pin};

pub(crate) type AsyncResponse =
    Pin<Box<dyn Future<Output = Result<Response, std::io::Error>> + Send>>;

pub struct Endpoint;

impl Endpoint {
    pub fn new<Req, Res, Ctx, F>(
        f: impl Fn(Ctx, Req) -> F + Send + Sync + Copy + 'static,
    ) -> impl Fn(Request, Params) -> AsyncResponse
    where
        Req: for<'de> Deserialize<'de> + Send + 'static + Default,
        Res: Serialize + 'static,
        Ctx: Context + Send + 'static,
        F: Future<Output = WebResult<Res>> + Send + 'static,
    {
        move |req: Request, params: Params| {
            let fut = async move {
                let has_body = req
                    .headers()
                    .get("content-length")
                    .map(|content_length| content_length.as_bytes() != b"0")
                    .unwrap_or_else(|| false);

                let (parts, body) = req.into_parts();

                // Await the evaluation of the context
                let context = match Ctx::from_parts(parts, params).await {
                    Ok(ctx) => ctx,
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

            Box::pin(fut)
        }
    }
}

pub(crate) fn error_response(
    msg: impl Serialize,
    code: http::StatusCode,
) -> Result<Response, std::io::Error> {
    let mut res = into_response(msg);
    *res.status_mut() = code;
    Ok(res)
}

fn success_response(msg: impl Serialize) -> Result<Response, std::io::Error> {
    let mut res = into_response(msg);
    *res.status_mut() = StatusCode::OK;
    Ok(res)
}

fn into_response(msg: impl Serialize) -> Response {
    let json_vec = serde_json::to_vec(&msg).unwrap();
    Response::new(json_vec.into())
}
