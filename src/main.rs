use turbo::prelude::*;

use futures::future::{ok, Future, FutureExt, Ready};
use http_service::{HttpService, Request, Response};
use http_service_hyper;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TestRequest {
    pub foo: String,
}

#[derive(Serialize)]
struct TestResponse;

async fn test_route(req: TestRequest) -> WebResult<TestResponse> {
    println!("{}", req.foo);
    Ok(TestResponse)
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
        let handler = Endpoint::new(test_route);

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
