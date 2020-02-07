use crate::params::Params;
use crate::{Endpoint, HttpError, Req};
use http_types::{headers, Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::{collections::HashMap, pin::Pin};

type ResponseFuture = Pin<Box<dyn Future<Output = http_types::Response> + Send + Sync>>;

pub struct StaticSegment {
    pub value: &'static str,
    pub position: usize,
}

pub struct DynamicSegment {
    pub name: &'static str,
    pub position: usize,
}

pub struct Route {
    pub static_segments: Vec<StaticSegment>,
    pub dynamic_segments: Vec<DynamicSegment>,
    pub handler: Option<Box<dyn Fn(http_types::Request, Params) -> ResponseFuture + Send + Sync>>,
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

    pub fn add<Error, Body, Res>(
        &mut self,
        method: Method,
        mut route: Route,
        endpoint: impl Endpoint<Error, Body, Res> + Send + Sync,
    ) where
        Error: HttpError + 'static + Send + Sync,
        Body: for<'de> Deserialize<'de> + Default + 'static + Send + Sync,
        Res: Serialize + 'static + Send + Sync,
    {
        let entry = self
            .table
            .entry(method)
            .or_insert_with(|| Vec::<Route>::new());

        let handler = move |mut req: http_types::Request, params: Params| -> ResponseFuture {
            use async_std::prelude::*;

            let fut = async move {
                let has_body = req
                    .header(&headers::CONTENT_LENGTH)
                    .map(|values| values.first().map(|value| value.as_str() == "0"))
                    .flatten()
                    .unwrap_or_else(|| false);

                let req_body: Body = if has_body {
                    let mut body = vec![];
                    if let Err(_) = req.read_to_end(&mut body).await {
                        return http_types::Response::new(StatusCode::BadRequest);
                    }

                    match serde_json::from_slice(&body) {
                        Ok(res_body) => res_body,
                        Err(_) => return http_types::Response::new(StatusCode::BadRequest),
                    }
                } else {
                    Body::default()
                };

                let res_body: Res = match endpoint.call(Req::new(req, req_body, params)).await {
                    Ok(res_body) => res_body,
                    Err(e) => {
                        let mut res = http_types::Response::new(e.code());
                        res.set_body(serde_json::to_vec(e.msg()).unwrap());
                        let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
                        return res;
                    }
                };

                let res_body_bytes = match serde_json::to_vec(&res_body) {
                    Ok(res_body_bytes) => res_body_bytes,
                    Err(e) => {
                        let mut res = http_types::Response::new(StatusCode::InternalServerError);
                        let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
                        res.set_body(
                            serde_json::to_vec(&serde_json::json!({
                                "error": format!("{}", e),
                            }))
                            .unwrap(),
                        );
                        return res;
                    }
                };

                let mut res = http_types::Response::new(StatusCode::Ok);
                res.set_body(res_body_bytes);
                let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
                res
            };
            Box::pin(fut)
        };

        route.handler = Some(Box::new(handler));
        entry.push(route);
    }

    pub(crate) async fn lookup(
        self: Arc<Self>,
        req: http_types::Request,
    ) -> Box<dyn Future<Output = http_types::Response> + Unpin + Send + Sync> {
        let method = req.method();
        let raw_route = RawRoute::from_path(req.url().path().into());
        let maybe_route = if let Some(routes) = self.table.get(&method) {
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

            Box::new((route.handler.as_ref().unwrap())(req, params))
        } else {
            Box::new(Box::pin(not_found()))
        }
    }
}

fn paths_match(route: &Route, raw_route: &RawRoute) -> bool {
    if raw_route.raw_segments.len() == route.static_segments.len() + route.dynamic_segments.len() {
        let static_matches = || {
            route
                .static_segments
                .iter()
                .fold(true, |is_match, static_segment| {
                    is_match && (&raw_route.raw_segments[static_segment.position] == static_segment)
                })
        };

        let dynamic_matches = || {
            route
                .dynamic_segments
                .iter()
                .fold(true, |is_match, dynamic_segment| {
                    is_match
                        && (&raw_route.raw_segments[dynamic_segment.position] == dynamic_segment)
                })
        };

        static_matches() && dynamic_matches()
    } else {
        false
    }
}

async fn not_found() -> http_types::Response {
    http_types::Response::new(StatusCode::NotFound)
}

pub(crate) struct RawSegment<'s> {
    value: &'s str,
    position: usize,
}

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
