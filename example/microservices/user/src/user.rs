use crate::{
    message::{LogoutRequest, LogoutResponse, TokenRequest, TokenResponse},
    context::{TokenContext, AuthContext},
};
use enzyme::{error::WebError, result::WebResult};
use http::status::StatusCode;
use serde_json::json;

pub(crate) struct User;

impl User {
    pub async fn token(&self, _cx: TokenContext, req: TokenRequest) -> WebResult<TokenResponse<'_>> {
        dbg!(req.username);
        Ok(TokenResponse { user_token: "12345" })
    }

    pub async fn logout(&self, cx: AuthContext, req: LogoutRequest) -> WebResult<LogoutResponse> {
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
