use http_types::StatusCode;

pub struct WebError {
    pub code: StatusCode,
    pub msg: serde_json::Value,
}
