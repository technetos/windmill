use crate::{context::Context, params::Params, result::WebResult};

use async_std::prelude::*;
use http_types::{headers, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::{error::Error, pin::Pin, str::FromStr};

pub(crate) type AsyncResponse =
    Pin<Box<dyn Future<Output = Result<Response, std::io::Error>> + Send + Sync>>;

pub fn service_endpoint<Req, Res, Ctx, F>(
    f: impl Fn(Ctx, Req) -> F + Send + Copy + 'static,
) -> Pin<Box<Fn(Request, Params) -> AsyncResponse>>
where
    Req: for<'de> Deserialize<'de> + Send + 'static + Default,
    Res: Serialize + Send + 'static,
    Ctx: Context + Send + 'static,
    F: Future<Output = WebResult<Res>> + Send + 'static + Sync,
{
    Box::pin(move |req: Request, params: Params| {
        Box::pin(async {
            let has_body = req
                .header(&headers::CONTENT_LENGTH)
                .map(|values| values.first().map(|value| value.as_str() == "0"))
                .flatten()
                .unwrap_or_else(|| false);

            let mut body = vec![];
            req.read_to_end(&mut body).await?;

            // Await the evaluation of the context
            let context = match Ctx::from_parts(&req, params).await {
                Ok(ctx) => ctx,
                Err(e) => return error_response(e.msg, e.code),
            };

            // Parse the body as json if the request has a body
            let req = if has_body {
                match serde_json::from_slice(&body) {
                    Ok(req) => req,
                    Err(e) => {
                        return error_response(format!("{}", e), StatusCode::BadRequest);
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
        })
    })
}

pub(crate) fn error_response(
    msg: impl Serialize,
    code: StatusCode,
) -> Result<Response, std::io::Error> {
    let mut res = Response::new(code);
    res.set_body(serde_json::to_vec(&msg)?);
    Ok(res)
}

fn success_response(msg: impl Serialize) -> Result<Response, std::io::Error> {
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(serde_json::to_vec(&msg)?);
    Ok(res)
}
