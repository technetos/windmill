#![feature(proc_macro_hygiene)]
use enzyme::{
    error::WebError,
    macros::{route, Context},
    result::WebResult,
    router::Router,
    server::Server,
};
use http::{method::Method, request::Parts, status::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Default)]
struct TestRequest {
    pub foo: String,
}

#[derive(Serialize)]
struct TestResponse {
    success: bool,
}

#[derive(Context)]
struct AuthContext {
    auth_token: String,
}

async fn auth_context(parts: Parts) -> WebResult<AuthContext> {
    match parts.headers.get("authorization") {
        Some(auth_token) => Ok(AuthContext {
            auth_token: auth_token.to_str().unwrap().to_string(),
        }),
        None => Err(WebError {
            msg: json!("Unauthorized"),
            code: StatusCode::UNAUTHORIZED,
        }),
    }
}

async fn test_route(cx: AuthContext, req: TestRequest) -> WebResult<TestResponse> {
    Ok(TestResponse { success: true })
}

fn main() {
    let router = {
        let mut router = Router::new();

        router.add(Method::GET, route!(/"users"/user_id/"me" => test_route));
        router.add(Method::GET, route!(/"users"/user_id/"me2" => test_route));
        router.add(Method::GET, route!(/"users"/user_id/"me3" => test_route));
        router.add(Method::GET, route!(/"users"/user_id/"me4" => test_route));
        router.add(Method::GET, route!(/"users"/user_id/"me5" => test_route));
        router.add(Method::GET, route!(/"users"/user_id/"me6" => test_route));
        router
    };

    Server::new(router).run()
}
