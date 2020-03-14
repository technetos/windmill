#![feature(proc_macro_hygiene)]

use enzyme::*;

use http_types::{headers, Method, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

    router.add(Method::Get, route!(/"images"/image_id), example_route, service);

    if let Err(e) = Server::new(config).run(router) {
        println!("{}", e);
    }
}

async fn example_route(req: Req<ExampleRequest>) -> Result<(String, String), Error> {
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

    Ok((req.params().get("image_id").unwrap().into(), String::new()))
}

pub async fn service<Body, Res>(
    mut req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res> + Send + Sync,
) -> http_types::Response
where
    Body: for<'de> Deserialize<'de> + 'static + Send + Sync,
    Res: Serialize + 'static + Send + Sync,
{
    use async_std::prelude::*;

    let has_body = req
        .header(&headers::CONTENT_LENGTH)
        .map(|header_values| header_values.first().map(|value| value.as_str() != "0"))
        .flatten()
        .unwrap_or_else(|| false);

    let mut body = vec![];
    if has_body {
        let _ = req.read_to_end(&mut body).await;
    }

    let req_body: Option<Body> = serde_json::from_slice(&body).unwrap_or_else(|_| None);

    let res_body: Res = match endpoint.call(Req::new(req, req_body, params)).await {
        Ok(res_body) => res_body,
        Err(e) => {
            let mut res = http_types::Response::new(e.code());
            let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
            res.set_body(serde_json::to_vec(e.msg()).unwrap());
            return res;
        }
    };

    let res_body_bytes = match serde_json::to_vec(&res_body) {
        Ok(res_body_bytes) => res_body_bytes,
        Err(e) => {
            let mut res = http_types::Response::new(StatusCode::InternalServerError);
            let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
            res.set_body(serde_json::to_vec(&format!("{}", e)).unwrap());
            return res;
        }
    };

    let mut res = http_types::Response::new(StatusCode::Ok);
    res.set_body(res_body_bytes);
    let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
    res
}
