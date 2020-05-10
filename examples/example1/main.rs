#![feature(proc_macro_hygiene)]

use windmill::*;

use http_types::{headers::HeaderName, Method, StatusCode};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
struct ExampleRequest {
    foo: String,
}

fn main() {
    let mut router = Router::new();
    let config = Config::new("127.0.0.1:4000");

    #[rustfmt::skip]
    router.add(Method::Get, route!(/"example"/id), ___example_route);
    router.add(Method::Get, route!(/"hello"/name), ___hello);

    if let Err(e) = Server::new(config).run(router) {
        println!("{}", e);
    }
}

#[endpoint]
async fn example_route(
    _auth: Auth,
    id: Id,
    body: Body<ExampleRequest>,
) -> Result<http_types::Response, Error> {
    let body = body.inner.ok_or_else(|| Error {
        code: StatusCode::BadRequest,
        msg: serde_json::json!("body required"),
    })?;

    dbg!(&body);

    dbg!(id.id);

    Ok(http_types::Response::new(StatusCode::Ok))
}

#[endpoint]
async fn hello() -> Result<http_types::Response, Error> {
    Ok(http_types::Response::new(StatusCode::Ok))
}

struct Auth {
    user_id: u64,
    token: String,
}

fn parse_header(req: &http_types::Request) -> Result<String, Error> {
    use std::str::FromStr;
    let header_name = HeaderName::from_str("authorization").map_err(|_header_name| Error {
        code: StatusCode::InternalServerError,
        msg: json!("bad header name"),
    })?;

    let header = req.header(&header_name).ok_or_else(|| Error {
        code: StatusCode::BadRequest,
        msg: json!("authorization required"),
    })?;

    let header = header.first().as_ref().unwrap().to_string();

    Ok(header)
}

impl Props for Auth {
    type Fut = PropsFuture<Self>;
    fn call(req: http_types::Request, params: Params) -> Self::Fut {
        Box::pin(async move {
            let header = parse_header(&req)?;

            Ok((
                req,
                params,
                Self {
                    user_id: 1,
                    token: header,
                },
            ))
        })
    }
}

struct Id {
    id: u64,
}

impl Props for Id {
    type Fut = PropsFuture<Self>;
    fn call(req: http_types::Request, params: Params) -> Self::Fut {
        Box::pin(async move {
            use std::str::FromStr;
            let id = u64::from_str(params.get("id").ok_or_else(|| Error {
                code: StatusCode::InternalServerError,
                msg: serde_json::json!("param does not exist"),
            })?)
            .map_err(|e| Error {
                code: StatusCode::BadRequest,
                msg: serde_json::json!(format!("{}", e)),
            })?;

            Ok((req, params, Self { id }))
        })
    }
}

struct Body<T> {
    inner: Option<T>,
}

impl<T: for<'de> Deserialize<'de> + std::fmt::Debug> Props for Body<T> {
    type Fut = PropsFuture<Self>;
    fn call(mut req: http_types::Request, params: Params) -> Self::Fut {
        Box::pin(async move {
            let body: Option<T> =
                serde_json::from_slice(&read_body(&mut req).await).unwrap_or_else(|_| None);

            Ok((req, params, Body { inner: body }))
        })
    }
}
