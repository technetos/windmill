use crate::{params::Params, result::WebResult};
use http::request::Parts;
use async_trait::async_trait;

#[async_trait]
pub trait Context: Sized {
    async fn from_parts<'a>(_: Parts, _: Params) -> WebResult<Self>;
}
