use crate::endpoint::AsyncResponse;
use futures::future::{Future, FutureExt};
use http_service::{Request, Response};
use std::{error::Error, pin::Pin, collections::HashMap};
use http::method::Method;

pub(crate) type RouteFn = Box<Fn(Request) -> AsyncResponse + Send + Sync>;

pub struct StaticSegment {
    pub value: &'static str,
    pub position: usize,
}

pub struct Route {
    pub static_segments: Vec<StaticSegment>,
    pub handler: RouteFn,
}

pub struct Router {
    table: HashMap<Method, Vec<Route>>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            table: HashMap::new(),
        }
    }

    pub fn add(&mut self, method: Method, route: impl Fn() -> Route) {
        self.table.entry(method).or_insert(vec![route()]).push(route());
    }
}
