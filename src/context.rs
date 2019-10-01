use crate::{params::Params, result::WebResult};
use futures::future::Future;
use http::request::Parts;
use std::pin::Pin;

pub trait Context: Sized {
    fn from_parts(_: Parts, _: Params) -> Pin<Box<dyn Future<Output = WebResult<Self>> + Send>>;
}
