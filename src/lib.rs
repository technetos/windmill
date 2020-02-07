pub mod req;
pub mod router;
pub mod server;

pub mod codegen {
    pub use enzyme_macro::route;
}

pub mod params {
    pub type Params = std::collections::HashMap<&'static str, String>;
}

use req::Req;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

pub trait HttpError {
    fn code(&self) -> http_types::StatusCode;
    fn msg(&self) -> &serde_json::Value;
}

pub struct Error {
    pub code: http_types::StatusCode,
    pub msg: serde_json::Value,
}

impl HttpError for Error {
    fn code(&self) -> http_types::StatusCode {
        self.code
    }

    fn msg(&self) -> &serde_json::Value {
        &self.msg
    }
}

pub trait Endpoint<Error, Body, Res>: 'static + Copy
where
    Error: HttpError + 'static,
    Body: for<'de> Deserialize<'de> + 'static + Send,
    Res: Serialize + 'static,
{
    type Fut: Future<Output = Result<Res, Error>> + Send + Sync;
    fn call(&self, req: Req<Body>) -> Self::Fut;
}

impl<Error, Body, Res, F, G> Endpoint<Error, Body, Res> for F
where
    Error: HttpError + 'static,
    Body: for<'de> Deserialize<'de> + 'static + Send,
    Res: Serialize + 'static + Send,
    G: Future<Output = Result<Res, Error>> + 'static + Send + Sync,
    F: Fn(Req<Body>) -> G + 'static + Copy,
{
    type Fut = Pin<Box<dyn Future<Output = Result<Res, Error>> + Send + Sync>>;
    fn call(&self, req: Req<Body>) -> Self::Fut {
        let fut = (self)(req);
        Box::pin(async move { fut.await })
    }
}
