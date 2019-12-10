use crate::{params::Params, result::WebResult};
use async_trait::async_trait;
use http_types::Headers;

#[async_trait]
pub trait Context: Sized {
    async fn from_parts(_: &Headers, _: Params) -> WebResult<Self>;
}
