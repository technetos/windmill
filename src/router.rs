use crate::{
    endpoint::{json_endpoint, Endpoint},
    params::Params,
    error::Error,
    route::{Route, RawRoute, ResponseFuture, DynamicSegment, StaticSegment},
};
use http_types::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

/// A router for routing requests to endpoints.  
pub struct Router {
    table: HashMap<Method, Vec<Route>>,
}

impl Router {
    /// Create a new Router.  See the [`add`](struct.Router.html#method.add) method for a more
    /// useful example.  
    /// ```
    /// let mut router = Router::new();
    /// ```
    pub fn new() -> Self {
        Router {
            table: HashMap::new(),
        }
    }

    /// Add routes to the router using the `add` method.  A route in the router is composed of a
    /// `http-types::Method`, a [`Route`](struct.Route.html) and a function.  
    /// ```
    /// async fn example(req: Req<String>) -> Result<String, Error> {
    ///   Ok(String::from("greetings"))
    /// }
    ///
    /// let mut router = Router::new();
    ///
    /// router.add(Method::Get, route!(/"example"), example);
    /// ```
    pub fn add<Body, Res>(
        &mut self,
        method: Method,
        mut route: Route,
        endpoint: impl Endpoint<Body, Res> + Send + Sync,
    ) where
        Body: for<'de> Deserialize<'de> + 'static + Send + Sync,
        Res: Serialize + 'static + Send + Sync,
    {
        let entry = self
            .table
            .entry(method)
            .or_insert_with(|| Vec::<Route>::new());

        let handler = move |req: http_types::Request, params: Params| -> ResponseFuture {
            Box::pin(json_endpoint(req, params, endpoint))
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
            routes.iter().find(|route| paths_match(route, &raw_route))
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
