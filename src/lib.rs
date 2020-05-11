//! # Windmill
//! A simple to use async web server framework.  
//!
//! ## Async functions
//! In rust async functions are functions that start with the keyword `async`
//! and return a `Future` that eventually yeilds the specified return type.  Functions that return
//! futures can be awaited upon with the `.await` syntax.  
//!
//! ## Endpoint
//! Endpoints are async functions that match the signature or similar of the function below:
//! ```
//! # pub use windmill::*;
//! # pub use http_types::{Response};
//! #[endpoint]
//! async fn example_route() -> Result<Response, Error> {
//!     // ...
//!     // ...
//!     Ok(Response::from("Hello!"))
//! }
//! ```
//! ## Props
//! Props are asynchronously constructed components that are passed into endpoints as function
//! arguments.  
//!
//! We can pass in a prop to the `example_route` above by modifying it to take an argument:
//! ```
//! # pub use windmill::*;
//! # pub use http_types::{Response};
//! # use serde::Deserialize;
//! # struct Body<T> {
//! #     inner: Option<T>,
//! # }
//!
//! # impl<T: for<'de> Deserialize<'de>> Props for Body<T> {
//! #     type Fut = PropsFuture<Self>;
//! #     fn call(mut req: http_types::Request, params: Params) -> Self::Fut {
//! #         Box::pin(async move {
//! #             let body: Option<T> =
//! #                 serde_json::from_slice(&read_body(&mut req).await).unwrap_or_else(|_| None);
//! #             Ok((req, params, Body { inner: body }))
//! #         })
//! #     }
//! # }
//!
//! #[endpoint]
//! async fn example_route(body: Body<String>) -> Result<Response, Error> {
//!     // ...
//!     // ...
//!     Ok(Response::from("Hello!"))
//! }
//! ```
//! ### Create your own props
//! In the example above the `Body` props could be implemented as follows:
//! ```
//! # pub use windmill::*;
//! # use serde::Deserialize;
//! struct Body<T> {
//!     inner: Option<T>,
//! }
//!
//! impl<T: for<'de> Deserialize<'de>> Props for Body<T> {
//!     type Fut = PropsFuture<Self>;
//!
//!     fn call(mut req: http_types::Request, params: Params) -> Self::Fut {
//!         Box::pin(async move {
//!             let body: Option<T> =
//!                 serde_json::from_slice(&read_body(&mut req).await).unwrap_or_else(|_| None);
//!
//!             Ok((req, params, Body { inner: body }))
//!         })
//!     }
//! }
//! ```
//! Props have access to the raw request and the params for the route.  Anything data stored
//! within the request or the params can be made available to an endpoint via a props.  
//!
//! Before `example_route` is invoked, an instance of the `Body` props is constructed using the
//! `call` method above.  Constructing an instance of the `Body` props parses the body from the
//! request and returns `Self`, this instance is then passed in as an argument to `example_route`.
//!
//! In this example we have made the parsed JSON body available to the endpoint through the `body`
//! argument.
//!
//! # Examples
//!
//! ```no_run
//! # #![feature(proc_macro_hygiene)]
//! # use windmill::*;
//! # use http_types::{Method, Response, StatusCode};
//! # #[endpoint] async fn example_route() -> Result<Response, Error> {  Ok(Response::new(StatusCode::Ok)) }
//! fn main() {
//!     let mut router = Router::new();
//!     let config = Config::new("127.0.0.1:4000");
//!
//!     router.add(Method::Get, route!(/"example"), ___example_route);
//!
//!     if let Err(e) = Server::new(config).run(router) {
//!         println!("{}", e);
//!     }
//! }
//! ```

mod config;
mod endpoint;
mod error;
mod props;
mod route;
mod router;
mod server;
mod util;

mod codegen {
    pub use codegen::endpoint;
    pub use codegen::route;
}

mod params {
    pub type Params = std::collections::HashMap<&'static str, String>;
}

pub use crate::{
    codegen::{endpoint, route},
    config::Config,
    endpoint::Endpoint,
    error::Error,
    params::Params,
    props::{Props, PropsFuture},
    route::{DynamicSegment, Route, StaticSegment},
    router::Router,
    server::Server,
    util::read_body,
};
