//! # Enzyme
//!
//! A simple to use async web server framework.  
//!
//! Currently all requests and response bodies are JSON only.  Making this pluggable is a future
//! goal.  
//!
//! __Basic Usage__
//!
//! ```no_run
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
//! async fn example(req: Req<String>) -> Result<String, Error> {
//!     Ok(String::from("foo"))
//! }
//! ```

mod config;
mod endpoint;
mod req;
mod router;
mod server;
mod error;

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
    router::{DynamicSegment, Route, Router, StaticSegment},
    server::Server,
    error::Error,
    codegen::route,
};
