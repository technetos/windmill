use crate::{params::Params, result::WebResult};
use async_trait::async_trait;
use http::request::Parts;

#[async_trait]
pub trait Context: Sized {
    async fn from_parts(_: Parts, _: Params) -> WebResult<Self>;
}
