use http_service::{HttpService, Request, Response};
use futures::future::{ok, Future, FutureExt, Ready};
use crate::router::Router;
use http_service_hyper;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    pin::Pin,
};

pub struct Server {
    router: Router,
}

impl Server {
    pub fn new(router: Router) -> Self {
        Self {
            router
        }
    }

}

impl HttpService for Server {
    type Connection = ();
    type ConnectionFuture = Ready<Result<(), std::io::Error>>;
    type ResponseFuture = Pin<Box<Future<Output = Result<Response, std::io::Error>> + Send>>;

    fn connect(&self) -> Self::ConnectionFuture {
        ok(())
    }

    fn respond(&self, _conn: &mut (), req: Request) -> Self::ResponseFuture {
        let method = req.method();
        let routes = self.router.lookup(method);

        for route in routes {
                return async move { Ok((route.handler)(req).await) }.boxed();
            }
        }
    }
}

fn paths_match(route: &Route, incoming_path: &str) -> bool {
    route.static_segments == RawRoute::from(incoming_path).static_segments
}

        
