use crate::WebResult;
use http::request::Parts;

pub struct Context {
    pub parts: Parts,
}

pub async fn default_context(parts: Parts) -> WebResult<Context> {
    Ok(Context { parts })
}
