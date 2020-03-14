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
//! pub async fn service<Body, Res>(
//!     mut req: http_types::Request,
//!     params: Params,
//!     endpoint: impl Endpoint<Body, Res> + Send + Sync,
//! ) -> http_types::Response
//! where
//!     Body: for<'de> Deserialize<'de> + 'static + Send + Sync,
//!     Res: Serialize + 'static + Send + Sync,
//! {
//!     use async_std::prelude::*;
//! 
//!     let has_body = req
//!         .header(&headers::CONTENT_LENGTH)
//!         .map(|header_values| header_values.first().map(|value| value.as_str() != "0"))
//!         .flatten()
//!         .unwrap_or_else(|| false);
//! 
//!     let mut body = vec![];
//!     if has_body {
//!         let _ = req.read_to_end(&mut body).await;
//!     }
//! 
//!     let req_body: Option<Body> = serde_json::from_slice(&body).unwrap_or_else(|_| None);
//! 
//!     let res_body: Res = match endpoint.call(Req::new(req, req_body, params)).await {
//!         Ok(res_body) => res_body,
//!         Err(e) => {
//!             let mut res = http_types::Response::new(e.code());
//!             let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
//!             res.set_body(serde_json::to_vec(e.msg()).unwrap());
//!             return res;
//!         }
//!     };
//! 
//!     let res_body_bytes = match serde_json::to_vec(&res_body) {
//!         Ok(res_body_bytes) => res_body_bytes,
//!         Err(e) => {
//!             let mut res = http_types::Response::new(StatusCode::InternalServerError);
//!             let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
//!             res.set_body(serde_json::to_vec(&format!("{}", e)).unwrap());
//!             return res;
//!         }
//!     };
//! 
//!     let mut res = http_types::Response::new(StatusCode::Ok);
//!     res.set_body(res_body_bytes);
//!     let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
//!     res
//! }
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

mod config;
mod endpoint;
mod error;
mod req;
mod res;
mod route;
mod router;
mod server;

mod codegen {
    pub use codegen::route;
}

mod params {
    pub type Params = std::collections::HashMap<&'static str, String>;
}

pub use crate::{
    codegen::route,
    config::Config,
    endpoint::{Endpoint, Service},
    error::Error,
    params::Params,
    req::Req,
    res::Res,
    route::{DynamicSegment, Route, StaticSegment},
    router::Router,
    server::Server,
};
