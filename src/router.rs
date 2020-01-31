use crate::context::Context;
use crate::params::Params;
use crate::result::WebResult;
use http_types::{headers, Method, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::{collections::HashMap, pin::Pin};







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
    pub handler: Option<Box<Fn(Request, Params) -> Pin<Box<Future<Output = Response>>>>>,
}

#[test]
fn test() {
    type Result<T> = std::result::Result<T, Response>;
    use serde::{Deserialize, Serialize};
    use crate::macros::route;

    let mut router = Arc::new(Router::new());

    #[derive(Deserialize, Default)]
    struct ExampleRequest {
        image_orientation: String,
    }

    #[derive(Serialize)]
    struct ExampleResponse;


    async fn example_route(req: ExampleRequest, params: Params) -> Result<ExampleResponse> {
        use std::str::FromStr;

        let id = match u64::from_str(params.get("image_id").unwrap_or_else(|| &String::new())) {
            Ok(id) => id,
            Err(_) => return Err(Response::new(StatusCode::BadRequest)),
        };

        Ok(ExampleResponse)
    }

    let get_images = route!(/"images"/image_id);

    router.add(Method::Get, get_images, example_route);




//        let req = parse_body::<GetImagesById>(req).await;
//



}

use async_std::prelude::*;

pub struct Router {
    table: HashMap<Method, Vec<Route>>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            table: HashMap::new(),
        }
    }

    pub fn add<Req, Res>(
        &mut self,
        method: Method,
        mut route: Route,
        endpoint: fn(Req, Params) -> Pin<Box<Future<Output = Res>>>,
    ) where
        Req: for<'de> Deserialize<'de> + 'static,
        Res: Serialize + 'static,
    {
        let entry = self
            .table
            .entry(method)
            .or_insert_with(|| Vec::<Route>::new());

        let handler = move |mut req: Request, params: Params| -> Pin<Box<Future<Output = Response>>> {
            let fut = async move {
                let has_body = req
                    .header(&headers::CONTENT_LENGTH)
                    .map(|values| values.first().map(|value| value.as_str() == "0"))
                    .flatten()
                    .unwrap_or_else(|| false);

                if has_body {
                    let mut body = vec![];
                    req.read_to_end(&mut body).await.unwrap();

                    let req: Req = serde_json::from_slice(&body).unwrap();

                    let res: Res = endpoint(req, params).await;
                }

                Response::new(StatusCode::NotFound)
            };
            Box::pin(fut)
        };

        route.handler = Some(Box::new(handler));
        entry.push(route);
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
        };

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

    let mut res = Response::new(StatusCode::NotFound);
    res.set_body(serde_json::to_vec(&json!("not found")).unwrap());
    res
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
