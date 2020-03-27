#![feature(proc_macro_hygiene)]

use enzyme::*;

use http_types::{mime, Method, Mime, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct ExampleRequest {
    foo: String,
}

fn main() {
    let mut router = Router::new();
    let config = Config::new("127.0.0.1:4000");

    #[rustfmt::skip]
    router.add(Method::Get, route!(/"example"/id), example_route, auth_service);
    router.add(Method::Get, route!(/"greeting"/name), hello, service);

    if let Err(e) = Server::new(config).run(router) {
        println!("{}", e);
    }
}

async fn example_route(req: Req<ExampleRequest>) -> Result<(u64, String), Error> {
    use std::str::FromStr;
    let id = u64::from_str(req.params().get("id").ok_or_else(|| Error {
        code: StatusCode::InternalServerError,
        msg: serde_json::json!("param does not exist"),
    })?)
    .map_err(|e| Error {
        code: StatusCode::BadRequest,
        msg: serde_json::json!(format!("{}", e)),
    })?;

    let body = req.body().ok_or_else(|| Error {
        code: StatusCode::BadRequest,
        msg: serde_json::json!("body required"),
    })?;

    dbg!(body);

    Ok((id, String::new()))
}

async fn hello(req: Req<ExampleRequest>) -> Result<String, Error> {
    let name = req.params().get("name").ok_or_else(|| Error {
        code: StatusCode::InternalServerError,
        msg: serde_json::json!("param does not exist"),
    })?;

    Ok(format!("Greetings {}!", name))
}

pub async fn service<Body, Res>(
    mut req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res>,
) -> http_types::Response
where
    Body: for<'de> Deserialize<'de>,
    Res: Serialize,
{
    let body = serde_json::from_slice(&read_body(&mut req).await).unwrap_or_else(|_| None);

    let maybe_bytes = endpoint
        .call(Req::new(req, body, params))
        .await
        .map(|body| serde_json::to_vec(&body));

    match maybe_bytes {
        Ok(Ok(bytes)) => {
            let mut res = response(StatusCode::Ok, mime::JSON);
            res.set_body(bytes);
            res
        }
        Ok(Err(e)) => {
            let mut res = response(StatusCode::InternalServerError, mime::JSON);
            res.set_body(serde_json::to_vec(&format!("{}", e)).unwrap());
            res
        }
        Err(e) => {
            let mut res = response(e.code(), mime::JSON);
            res.set_body(serde_json::to_vec(e.msg()).unwrap());
            res
        }
    }
}

pub async fn auth_service<Body, Res>(
    mut req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res>,
) -> http_types::Response
where
    Body: for<'de> Deserialize<'de>,
    Res: Serialize,
{
    use std::str::FromStr;
    if req.header(&http_types::headers::HeaderName::from_str("authorization").unwrap()).is_none() {
        return http_types::Response::new(StatusCode::BadRequest);
    }
    service(req, params, endpoint).await    
}

fn response(code: StatusCode, mime: Mime) -> http_types::Response {
    let mut res = http_types::Response::new(code);
    let _ = res.set_content_type(mime);
    res
}
