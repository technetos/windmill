//! # Windmill
//!
//! A simple to use async web server framework.  
//!
//! A core concept in windmill is automatic deserialization and serialization of user defined
//! request and response types.  Currently all requests and response bodies are JSON only.  Making
//! this pluggable is a future goal.  
//!
//! The `Content-Length` header is required in any requests containing a body that
//! you wish to be automatically deserialized.  A `Content-Length` of 0 will prevent
//! deserialization of the body entirely.   
//!
//! # Basic Usage
//!
//! ```
//!
//! #[derive(Deserialize)]
//! struct ExampleBody<'s> {
//!     name: &'s str,
//! }
//!
//! #[derive(Serialize)]
//! struct ExampleResponse {
//!     greeting: String,
//! }
//!
//! async fn example(req: Req<ExampleBody>) -> Result<ExampleResponse, Error> {
//!     let body = req.body().ok_or_else(|| Error {
//!         code: StatusCode::BadRequest,
//!         msg: json!("body required"),
//!     })?;
//!
//!     Ok(ExampleResponse {
//!         greeting: format!("Greetings {}!", body.name),
//!     })
//! }
//!
//! fn main() {
//!     let mut router = Router::new();
//!     let config = Config::new("127.0.0.1:4000");
//!
//!     router.add(Method::Get, route!(/"example"), example, service);
//!
//!     if let Err(e) = Server::new(config).run(router) {
//!         println!("{}", e);
//!     }
//! }
//!
//! ```
//! # Windmill
//!
//! ## Async functions
//! In rust async functions are functions that start with the keyword `async`
//! and return a `Future` that eventually yeilds the specified return type.
//!
//! ## Traits
//!
//! ### Service
//!
//! #### Base service function
//! #### Composite service functions
//!
//! ### Endpoint
//!

mod config;
mod endpoint;
mod error;
mod req;
mod route;
mod router;
mod server;
mod service;
mod state;
mod util;

mod codegen {
    pub use codegen::endpoint;
    pub use codegen::route;
}

mod params {
    pub type Params = std::collections::HashMap<&'static str, String>;
}

mod context {
    pub type Context =
        std::collections::HashMap<&'static str, Box<dyn std::any::Any + Send + Sync>>;
}

pub use crate::{
    codegen::{endpoint, route},
    config::Config,
    context::Context,
    endpoint::Endpoint,
    error::Error,
    params::Params,
    req::Req,
    route::{DynamicSegment, Route, StaticSegment},
    router::Router,
    server::Server,
    service::{Service, ServiceFuture},
    state::State,
    util::read_body,
};
