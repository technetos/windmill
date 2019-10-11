#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

use enzyme::{
    error::WebError,
    macros::{route, Context},
    params::Params,
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
struct TestResponse<'s> {
    success: &'s str,
}

#[derive(Context)]
struct AuthContext {
    auth_token: String,
}

async fn auth_context(parts: Parts, params: Params) -> WebResult<AuthContext> {
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

struct TestEndpoint {
    msg: String,
}

impl TestEndpoint {
    pub async fn test(
        &self,
        cx: AuthContext,
        req: TestRequest,
    ) -> WebResult<TestResponse<'_>> {
        Ok(TestResponse { success: &self.msg })
    }
}

lazy_static! {
    static ref TEST_SERVICE: TestEndpoint = TestEndpoint {
        msg: "Hello".into(),
    };
}

fn main() {
    let test_route = |cx: AuthContext, req: TestRequest| TEST_SERVICE.test(cx, req);

    let router = {
        let mut router = Router::new();

        router.add(Method::GET, route!(/"users"/user_id/"me" => test_route));
        router
    };

    Server::new(router).run()
}
