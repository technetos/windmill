use crate::endpoint::AsyncResponse;
use futures::future::{Future, FutureExt};
use http_service::{Request, Response};
use std::{error::Error, pin::Pin};

pub(crate) type RouteFn = Box<Fn(Request) -> AsyncResponse>;

pub struct Route {
    pub path: String,
    pub handler: RouteFn,
}

impl Route {
    pub fn new(path: impl Into<String>, handler: RouteFn) -> Self {
        let path = path.into();

        Self { path, handler }
    }
}

//pub fn router(routes: Vec<Route>) -> impl Fn(Request) -> AsyncResponse {
//
//    move |req: Request| {
//        let fut = async move {
//
//        };
//
//        fut.boxed()
//    }
//}
