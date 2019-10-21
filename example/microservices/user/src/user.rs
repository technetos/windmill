use crate::{
    context::{AuthContext, TokenContext},
    message::{LogoutRequest, LogoutResponse, TokenRequest, TokenResponse},
};
use enzyme::{error::WebError, result::WebResult};
use http::status::StatusCode;
use serde_json::json;

pub(crate) struct User;

impl User {
    pub async fn token(&self, _: TokenContext, req: TokenRequest) -> WebResult<TokenResponse<'_>> {
        dbg!(req.username);
        Ok(TokenResponse {
            user_token: "12345",
        })
    }

    pub async fn logout(&self, cx: AuthContext, _: LogoutRequest) -> WebResult<LogoutResponse> {
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
