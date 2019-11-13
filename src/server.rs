use crate::router::Router;

use http_service::{HttpService, Request, Response};
use http_service_hyper;
use std::future::Future;
use std::{
    net::{AddrParseError, SocketAddr},
    pin::Pin,
    sync::Arc,
};

pub struct Config {
    sock_addr: SocketAddr,
}

impl Config {
    pub fn new(addr: &str) -> Result<Self, AddrParseError> {
        Ok(Self {
            sock_addr: addr.parse()?,
        })
    }

    fn into_socket_addr(self) -> SocketAddr {
        self.sock_addr
    }
}

pub struct Server {
    router: Arc<Router>,
}

impl HttpService for Server {
    type Connection = ();
    type ConnectionFuture = crate::ready::Ready<Result<(), std::io::Error>>;
    type ResponseFuture = Pin<Box<dyn Future<Output = Result<Response, std::io::Error>> + Send>>;

    fn connect(&self) -> Self::ConnectionFuture {
        crate::ready::ready(Ok(()))
    }

    fn respond(&self, _conn: &mut (), req: Request) -> Self::ResponseFuture {
        self.router.clone().lookup(req)
    }
}

impl Server {
    pub fn new(router: Router) -> Self {
        Self {
            router: Arc::new(router),
        }
    }

    pub fn run(self, config: Config) {
        http_service_hyper::run(self, config.into_socket_addr());
    }
}
