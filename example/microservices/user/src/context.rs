use async_trait::async_trait;
use enzyme::{context::Context, error::WebError, params::Params, result::WebResult};
use http::{request::Parts, status::StatusCode};
use serde_json::json;

pub(crate) struct TokenContext;

#[async_trait]
impl Context for TokenContext {
    async fn from_parts(_parts: Parts, _params: Params) -> WebResult<TokenContext> {
        Ok(TokenContext)
    }
}

pub(crate) struct AuthContext {
    pub(crate) user_token: String,
}

#[async_trait]
impl Context for AuthContext {
    async fn from_parts(parts: Parts, _params: Params) -> WebResult<AuthContext> {
        match parts.headers.get("authorization") {
            Some(user_token) => Ok(AuthContext {
                user_token: user_token.to_str().unwrap().to_string(),
            }),
            None => Err(WebError {
                msg: json!("Unauthorized"),
                code: StatusCode::UNAUTHORIZED,
            }),
        }
    }
}
