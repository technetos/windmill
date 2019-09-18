#![feature(proc_macro_hygiene)]
use enzyme::context::default_context;
use enzyme::macros::route;
use enzyme::router::Router;
use http::method::Method;
use enzyme::prelude::*;

use futures::future::{ok, Future, FutureExt, Ready};
use http_service::{HttpService, Request, Response};
use http_service_hyper;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
};

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
    Ok(TestResponse { success: true })
}

pub struct Server {
    router: Router,
}

impl HttpService for Server {
    type Connection = ();
    type ConnectionFuture = Ready<Result<(), std::io::Error>>;
    type ResponseFuture = Pin<Box<Future<Output = Result<Response, std::io::Error>> + Send>>;

    fn connect(&self) -> Self::ConnectionFuture {
        ok(())
    }

    fn respond(&self, _conn: &mut (), req: Request) -> Self::ResponseFuture {
        let test_endpoint = Endpoint::new(test_route, default_context);

        async move { Ok((test_endpoint)(req).await) }.boxed()
    }
}

fn main() {
    let mut router = Router::new();
    router.add(Method::POST, route!(/"users"/user_id: i32/"me" => default_context => test_route));

    let s = Server { router };

    http_service_hyper::run(
        s,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
    );
}
