#![feature(proc_macro_hygiene)]

use enzyme::codegen::route;
use enzyme::config::Config;
use enzyme::req::Req;
use enzyme::router::DynamicSegment;
use enzyme::router::Route;
use enzyme::router::Router;
use enzyme::router::StaticSegment;
use enzyme::server::Server;
use enzyme::Error;
use http_types::{Method, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct ExampleRequest {
    image_orientation: String,
}

#[derive(Serialize)]
struct ExampleResponse {
    foo: String,
}

fn main() {
    let mut router = Router::new();
    let config = Config::new("127.0.0.1:4000");

    router.add(Method::Post, route!(/"images"/image_id), example_route);

    if let Err(e) = Server::new(config).run(std::sync::Arc::new(router)) {
        println!("{}", e);
    }
}

async fn example_route(req: Req<ExampleRequest>) -> Result<ExampleResponse, Error> {
    use std::str::FromStr;
    let image_id = u64::from_str(req.params().get("image_id").ok_or_else(|| Error {
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

    Ok(ExampleResponse {
        foo: String::new(),
    })
}
