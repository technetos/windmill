#![feature(proc_macro_hygiene)]
use enzyme::{
    context::Context,
    macros::{route, Context},
    result::WebResult,
    router::Router,
    server::Server,
};

use http::method::Method;
use http::request::Parts;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default)]
struct TestRequest {
    pub foo: String,
}

#[derive(Serialize)]
struct TestResponse {
    success: bool,
}

#[derive(Context)]
struct SimpleContext;

async fn simple_context(_parts: Parts) -> WebResult<SimpleContext> {
    Ok(SimpleContext)
}

async fn test_route(cx: SimpleContext, req: TestRequest) -> WebResult<TestResponse> {
    println!("req.foo: {}", &req.foo);
    Ok(TestResponse { success: true })
}

fn main() {
    let router = {
        let mut router = Router::new();
        router.add(
            Method::GET,
            route!(/"users"/user_id: i32/"me" => test_route),
        );
        router
    };

    Server::new(router).run()
}
