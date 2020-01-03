use crate::context::Context;
use crate::params::Params;
use crate::result::WebResult;
use http_types::{headers, Method, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::{collections::HashMap, pin::Pin};

pub trait Endpoint: Send + Sync + Copy + 'static {
    fn call<Req: for<'de> Deserialize<'de> + Send, Res: Serialize>(
        &self,
        req: Req,
        params: Params,
    ) -> BoxFuture<'static, Res>;
}

pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub(crate) type DynEndpoint =
    dyn (Fn(Request, Params) -> BoxFuture<'static, Response>) + 'static + Send + Sync;

#[derive(Debug)]
pub struct StaticSegment {
    pub value: &'static str,
    pub position: usize,
}

#[derive(Debug)]
pub struct DynamicSegment {
    pub name: &'static str,
    pub position: usize,
}

pub struct Route {
    pub static_segments: Vec<StaticSegment>,
    pub dynamic_segments: Vec<DynamicSegment>,
    pub handler: Option<Box<DynEndpoint>>,
}

use async_std::prelude::*;
impl Route {
    fn mount<Req, Res>(&mut self, endpoint: impl Endpoint)
    where
        Req: for<'de> Deserialize<'de> + Default + Send,
        Res: Serialize + Send,
    {
        self.handler = Some(Box::new(move |mut req, params| {
            let fut = async move {
                let has_body = req
                    .header(&headers::CONTENT_LENGTH)
                    .map(|values| values.first().map(|value| value.as_str() == "0"))
                    .flatten()
                    .unwrap_or_else(|| false);

                let mut body = vec![];
                req.read_to_end(&mut body).await.unwrap();

                // Parse the body as json if the request has a body
                let decoded_req = if has_body {
                    match serde_json::from_slice(&body) {
                        Ok(req) => req,
                        Err(e) => {
                            return error_response(format!("{}", e), StatusCode::BadRequest);
                        }
                    }
                } else {
                    Req::default()
                };

                // Await the evaluation of the endpoint handler
                let res: Res = endpoint.call(decoded_req, params).await;
                success_response(res)
            };
            Box::pin(fut)
        }));
    }
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
        self.table
            .entry(method)
            .or_insert_with(|| vec![route()])
            .push(route());
    }

    pub(crate) fn lookup(
        self: Arc<Self>,
        req: Request,
    ) -> Box<dyn Future<Output = Response> + Unpin> {
        let method = req.method();
        let raw_route = RawRoute::from_path(req.url().path().into());
        let maybe_route = if let Some(routes) = self.table.get(method) {
            routes
                .iter()
                .filter(|route| paths_match(route, &raw_route))
                .nth(0)
        } else {
            return Box::new(Box::pin(not_found()));
        };

        if let Some(route) = maybe_route {
            let params = route.dynamic_segments.iter().fold(
                HashMap::new(),
                |mut params, dynamic_segment| {
                    params.insert(
                        dynamic_segment.name,
                        raw_route.raw_segments[dynamic_segment.position]
                            .value
                            .into(),
                    );
                    params
                },
            );

            return Box::new((route.handler.as_ref().unwrap())(req, params));
        }

        Box::new(Box::pin(not_found()))
    }
}

fn paths_match(route: &Route, raw_route: &RawRoute) -> bool {
    if raw_route.raw_segments.len() == route.static_segments.len() + route.dynamic_segments.len() {
        let static_matches = route
            .static_segments
            .iter()
            .fold(true, |is_match, static_segment| {
                is_match && (&raw_route.raw_segments[static_segment.position] == static_segment)
            });

        let dynamic_matches =
            route
                .dynamic_segments
                .iter()
                .fold(true, |is_match, dynamic_segment| {
                    is_match
                        && (&raw_route.raw_segments[dynamic_segment.position] == dynamic_segment)
                });

        static_matches && dynamic_matches
    } else {
        false
    }
}

async fn not_found() -> Response {
    use serde_json::json;

    error_response(json!("not found"), StatusCode::NotFound)
}

#[derive(Debug)]
pub(crate) struct RawSegment<'s> {
    value: &'s str,
    position: usize,
}

#[derive(Debug)]
pub(crate) struct RawRoute<'s> {
    pub raw_segments: Vec<RawSegment<'s>>,
}

impl<'s> RawRoute<'s> {
    pub(crate) fn from_path(path: &'s str) -> Self {
        Self {
            raw_segments: path
                .split("/")
                .skip(1)
                .enumerate()
                .map(|(i, segment)| RawSegment {
                    value: segment,
                    position: i,
                })
                .collect(),
        }
    }
}

impl<'s> PartialEq<RawSegment<'s>> for StaticSegment {
    fn eq(&self, other: &RawSegment) -> bool {
        self.position == other.position && self.value == other.value
    }
}

impl<'s> PartialEq<RawSegment<'s>> for DynamicSegment {
    fn eq(&self, other: &RawSegment) -> bool {
        self.position == other.position
    }
}

impl<'s> PartialEq<StaticSegment> for RawSegment<'s> {
    fn eq(&self, other: &StaticSegment) -> bool {
        other == self
    }
}

impl<'s> PartialEq<DynamicSegment> for RawSegment<'s> {
    fn eq(&self, other: &DynamicSegment) -> bool {
        other == self
    }
}

pub(crate) fn error_response(msg: impl Serialize, code: StatusCode) -> Response {
    let mut res = Response::new(code);
    res.set_body(serde_json::to_vec(&msg).unwrap());
    res
}

fn success_response(msg: impl Serialize) -> Response {
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(serde_json::to_vec(&msg).unwrap());
    res
}
