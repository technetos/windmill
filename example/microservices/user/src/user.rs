use crate::message::{LogoutRequest, LogoutResponse, TokenRequest, TokenResponse};

use enzyme::{error::WebError, macros::Context, params::Params, result::WebResult};
use http::{request::Parts, status::StatusCode};
use serde_json::json;

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

pub(crate) struct User {
    pub(crate) database: String,
}

impl User {
    pub async fn token(&self, _cx: TokenContext, _req: TokenRequest) -> WebResult<TokenResponse<'_>> {
        Ok(TokenResponse { user_token: "12345" })
    }

    pub async fn logout(&self, cx: AuthContext, _req: LogoutRequest) -> WebResult<LogoutResponse> {
        if &cx.user_token == "12345" {
            Ok(LogoutResponse)
        } else {
            Err(WebError {
                msg: json!("Unauthorized"),
                code: StatusCode::UNAUTHORIZED,
            })
        }
    }
}
