#![feature(proc_macro_hygiene)]
use enzyme::{
    macros::route,
    result::WebResult,
    router::Router,
    server::Server,
    error::WebError,
};

use http::{method::Method, request::Parts, status::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

struct CustomContext {
    auth_token: String,
}

async fn auth_context(parts: Parts) -> WebResult<CustomContext> {
    match parts.headers.get("authorization") {
        Some(auth_token) => Ok(CustomContext {
            auth_token: auth_token.to_str().unwrap().to_string(),
        }),
        None => Err(WebError {
            msg: json!("Unauthorized"),
            code: StatusCode::UNAUTHORIZED,
        }),
    }
}

#[derive(Deserialize, Default)]
struct TestRequest {
    pub foo: String,
}

#[derive(Serialize)]
struct TestResponse {
    success: bool,
}

async fn test_route(cx: CustomContext, req: TestRequest) -> WebResult<TestResponse> {
    Ok(TestResponse { success: true })
}

fn main() {
    let router = {
        let mut router = Router::new();
        router.add(
            Method::GET,
            route!(/"users"/user_id: i32/"me" => auth_context => test_route),
        );
        router.add(
            Method::GET,
            route!(/"users"/"me"/user_id: i32 => auth_context => test_route),
        );
        router.add(
            Method::POST,
            route!(/"info"/node_id: u64 => auth_context => test_route),
        );
        router
    };

    Server::new(router).run()
}
