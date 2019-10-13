use enzyme::{error::WebError, params::Params, macros::Context, result::WebResult};
use serde_json::json;
use http::{request::Parts, status::StatusCode};

#[derive(Context)]
pub(crate) struct TokenContext;

async fn token_context(_parts: Parts, _params: Params) -> WebResult<TokenContext> {
    Ok(TokenContext)
}

#[derive(Context)]
pub(crate) struct AuthContext {
    pub(crate) user_token: String,
}

async fn auth_context(parts: Parts, _params: Params) -> WebResult<AuthContext> {
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
