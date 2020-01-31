use crate::{
    context::{AuthContext, TokenContext},
    message::{LogoutRequest, LogoutResponse, TokenRequest, TokenResponse},
};
use enzyme::{error::WebError, result::WebResult};
use http::status::StatusCode;
use serde_json::json;

pub async fn token(_: TokenRequest) -> TokenResponse<'static> {
    TokenResponse {
        user_token: "12345",
    }
}

pub async fn logout(_: LogoutRequest) -> LogoutResponse {
    if &cx.user_token == "12345" {
        Ok(LogoutResponse)
    } else {
        Err(WebError {
            msg: json!("Unauthorized"),
            code: StatusCode::UNAUTHORIZED,
        })
    }
}
