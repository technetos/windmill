use crate::{
    endpoint::Endpoint,
    params::Params,
    route::{RawRoute, ResponseFuture, Route},
    service::Service,
};
use http_types::{mime, Method, Mime, StatusCode};
use std::{collections::HashMap, future::Future, sync::Arc};

/// ## Router
///
/// This is the router provided by windmill for routing requests.  
///
/// A route is composed of 4 parts:
///
/// + Methods
///
/// Methods in rust use `http_types::Method` variants like `Get` and `Post`.
///
/// + Routes
///
/// Routes are generated using the `route!` macro.  See the module level documentation.  
///
/// + Endpoints
///
/// Endpoints are functions that match the signature or similar of the function below:
/// ```rust
///
/// async fn example_route(req: Req<ExampleRequest>) -> Result<String, Error> {
///     ...
///     ...
///     Ok(String::from("Hello!"))
/// }
///
/// ```
///
/// + Services
///
/// See the main document on Services for more general information.  
///
/// A router for dispatching requests to endpoints.  
pub struct Router {
    table: HashMap<Method, Vec<Route>>,
}

impl Router {
    /// Create a new Router.  
    ///
    /// ## Examples
    /// ```
    /// let mut router = Router::new();
    /// ```
    pub fn new() -> Self {
        Router {
            table: HashMap::new(),
        }
    }

    /// A route in the router is composed of an `http-types::Method`, a
    /// [`Route`](struct.Route.html), an endpoint and a service.  
    ///
    /// ## Examples
    /// ```
    /// async fn example(req: Req<String>) -> Result<String, Error> {
    ///     Ok(String::from("greetings"))
    /// }
    ///
    /// async fn example2(req: Req<u64>) -> Result<(), Error> {
    ///     Ok(())
    /// }
    ///
    /// let mut router = Router::new();
    ///
    /// router.add(Method::Get, route!(/"example"), example, service);
    /// router.add(Method::Get, route!(/"example2"), example2, service);
    /// ```
    ///
    /// ## Precedence and ambiguity
    ///
    /// When adding routes to the router the order they are added in is sometimes important.  
    /// ```
    /// router.add(Method::Get, route!(/a/b/c, example, service);
    /// router.add(Method::Get, route!(/"a"/b/c, example2, service);
    /// ```
    /// In the example above the second route will never get run because the first route matches
    /// the literal "a" as a dynamic segment.  To solve this simply insert _more specific_
    /// routes before _less specific_ routes as seen in the example below.  
    /// ```
    /// router.add(Method::Get, route!(/"a"/b/c, example2, service);
    /// router.add(Method::Get, route!(/a/b/c, example, service);
    /// ```
    pub fn add(&mut self, method: Method, mut route: Route, endpoint: impl Endpoint + Send + Sync) {
        let entry = self
            .table
            .entry(method)
            .or_insert_with(|| Vec::<Route>::new());

        // haha type erasure is awesome
        let handler = move |req: http_types::Request, params: Params| -> ResponseFuture {
            Box::pin(async move {
                match endpoint.call(req, params).await {
                    Ok(res) => res,
                    Err(e) => {
                        let mut res = response(e.code(), mime::JSON);
                        let bytes = serde_json::to_vec(e.msg()).unwrap();
                        res.set_body(bytes);
                        res
                    }
                }
            })
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

        match self
            .table
            .get(&method)
            .map(|routes| routes.iter().find(|route| paths_match(route, &raw_route)))
        {
            Some(Some(route)) => {
                let mut params = HashMap::new();

                route.dynamic_segments.iter().for_each(|dynamic_segment| {
                    params.insert(
                        dynamic_segment.name,
                        raw_route.raw_segments[dynamic_segment.position]
                            .value
                            .into(),
                    );
                });

                Box::new((route.handler.as_ref().unwrap())(req, params))
            }
            _ => Box::new(Box::pin(not_found())),
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

fn response(code: StatusCode, mime: Mime) -> http_types::Response {
    let mut res = http_types::Response::new(code);
    let _ = res.set_content_type(mime);
    res
}
