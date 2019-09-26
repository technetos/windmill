use crate::result::WebResult;
use futures::future::{Future, FutureExt};
use http::request::Parts;
use std::pin::Pin;

pub trait FromParts: Sized {
    fn from_parts(_: Parts) -> Pin<Box<dyn Future<Output = WebResult<Self>> + Send>>;
}

pub struct Context {
    pub parts: Parts,
}

pub async fn default_context(parts: Parts) -> WebResult<Context> {
    Ok(Context { parts })
}

impl FromParts for Context {
    fn from_parts(parts: Parts) -> Pin<Box<dyn Future<Output = WebResult<Self>> + Send>> {
        async move { default_context(parts).await }.boxed()
    }
}
