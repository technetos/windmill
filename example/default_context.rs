#![feature(proc_macro_hygiene)]
use enzyme::{
    context::{default_context, Context},
    macros::route,
    result::WebResult,
    router::Router,
    server::Server,
};

use http::method::Method;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default)]
struct TestRequest {
    pub foo: String,
}

#[derive(Serialize)]
struct TestResponse {
    success: bool,
}

async fn test_route(cx: Context, req: TestRequest) -> WebResult<TestResponse> {
    println!("req.foo: {}", &req.foo);
    Ok(TestResponse { success: true })
}

fn main() {
    let router = {
        let mut router = Router::new();
        router.add(
            Method::GET,
            route!(/"users"/user_id: i32/"me" => default_context => test_route),
        );
        router.add(
            Method::GET,
            route!(/"users"/"me"/user_id: i32 => default_context => test_route),
        );
        router.add(
            Method::POST,
            route!(/"info"/node_id: u64 => default_context => test_route),
        );
        router
    };

    Server::new(router).run()
}
