use crate::{params::Params, result::WebResult};
use async_trait::async_trait;
use http_types::Request;
use std::future::Future;

pub trait Context: Sized {
    fn from_parts(
        _: &Request,
        _: Params,
    ) -> Box<Future<Output = WebResult<Self>> + Unpin + Send + Sync>;
}
