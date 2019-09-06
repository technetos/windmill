use enzyme::prelude::*;

use futures::future::{ok, Future, FutureExt, Ready};
use http::{request::Parts, status::StatusCode};
use http_service::{HttpService, Request, Response};
use http_service_hyper;
use serde_json::json;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
};

use serde::{Deserialize, Serialize};

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

pub struct Server;

impl HttpService for Server {
    type Connection = ();
    type ConnectionFuture = Ready<Result<(), std::io::Error>>;
    type ResponseFuture = Pin<Box<Future<Output = Result<Response, std::io::Error>> + Send>>;

    fn connect(&self) -> Self::ConnectionFuture {
        ok(())
    }

    fn respond(&self, _conn: &mut (), req: Request) -> Self::ResponseFuture {
        let handler = Endpoint::new(test_route, auth_context);

        async move { Ok((handler)(req).await) }.boxed()
    }
}

fn main() {
    let s = Server;

    http_service_hyper::run(
        s,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
    );
}
