use http::StatusCode;
use serde::Serialize;

pub struct WebError {
    pub code: StatusCode,
    pub msg: serde_json::Value,
}
