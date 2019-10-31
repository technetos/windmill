use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default)]
pub struct TokenRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse<'s> {
    pub user_token: &'s str,
}

#[derive(Deserialize, Default)]
pub struct LogoutRequest;

#[derive(Serialize)]
pub struct LogoutResponse;
