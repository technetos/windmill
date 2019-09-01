use http::request::Parts;
use crate::WebResult;

pub struct Context {
    parts: Parts,
}

pub async fn default_context(parts: Parts) -> WebResult<Context> {
    Ok(Context { parts })
}
