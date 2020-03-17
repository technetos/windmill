use serde::Serialize;
use http_types::{Method, Request, url::Url};
use enzyme::{Res, Error};
use crate::messages::*;

pub struct UserClient;

impl UserClient {
    pub async fn user_by_id<T>(method: Method, body: T) -> Result<Res<UserByIdRes>, Error>
    where
        T: Serialize + Send + 'static,
    {
        use std::str::FromStr;
        let mut req = Request::new(method, Url::from_str(&format!("{}/v1/user/id", "http://127.0.0.1:4500")).unwrap());
        let res_body_bytes = serde_json::to_vec(&body).unwrap();
        req.set_body(res_body_bytes);

        Ok(enzyme::send_request(req, "http://127.0.0.1:4500").await)
    }
}

pub struct Clients {
    pub user: UserClient,
}

pub const CLIENTS: Clients = Clients {
    user: UserClient,
};
