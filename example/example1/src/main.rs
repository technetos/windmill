#![feature(proc_macro_hygiene)]

mod service;
use service::{auth_service, service};

use windmill::*;

use http_types::{Method, StatusCode};
use serde::Deserialize;

#[derive(Deserialize)]
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

    Ok((id, String::new()))
}

async fn hello(req: Req<Vec<String>>) -> Result<String, Error> {
    let name = req.params().get("name").ok_or_else(|| Error {
        code: StatusCode::InternalServerError,
        msg: serde_json::json!("param does not exist"),
    })?;

    dbg!(req.body());

    Ok(format!("Greetings {}!", name))
}
