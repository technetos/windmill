//! # Enzyme 🧪
//!
//! A simple to use async web server framework.  
//!
//! A core concept in enzyme is automatic deserialization and serialization of user defined
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
//! fn main() {
//!     let mut router = Router::new();
//!     let config = Config::new("127.0.0.1:4000");
//!
//!     router.add(Method::Get, route!(/"example"), example);
//!
//!     if let Err(e) = Server::new(config).run(router) {
//!         println!("{}", e);
//!     }
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
//! ```

mod config;
mod endpoint;
mod req;
mod router;
mod server;
mod error;
mod route;

mod codegen {
    pub use codegen::route;
}

mod params {
    pub type Params = std::collections::HashMap<&'static str, String>;
}

pub use crate::{
    config::Config,
    endpoint::Endpoint,
    req::Req,
    router::Router,
    route::{DynamicSegment, StaticSegment, Route},
    server::Server,
    error::Error,
    codegen::route,
};
