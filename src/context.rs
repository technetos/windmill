use crate::result::WebResult;
use futures::future::{Future, FutureExt};
use http::request::Parts;
use std::pin::Pin;

pub trait Context: Sized {
    fn from_parts(_: Parts) -> Pin<Box<dyn Future<Output = WebResult<Self>> + Send>>;
}
