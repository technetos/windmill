#![feature(proc_macro_hygiene)]

use enzyme::codegen::route;
use enzyme::router::DynamicSegment;
use enzyme::req::Req;
use enzyme::router::Route;
use enzyme::router::Router;
use enzyme::router::StaticSegment;
use enzyme::Error;
use enzyme::server::Server;
use http_types::{Method, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default, Debug)]
struct ExampleRequest {
    image_orientation: String,
}

#[derive(Serialize)]
struct ExampleResponse {
    foo: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::new();

    router.add(Method::Get, route!(/"images"/image_id), example_route);
    Server::new().run(std::sync::Arc::new(router));

    Ok(())
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

    println!("image_id: {}", &image_id);

    let query_params = req.url().query();

    dbg!(&query_params);

    Ok(ExampleResponse {
        foo: format!("{}", &image_id),
    })
}
