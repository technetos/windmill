pub mod config;
pub mod req;
pub mod router;
pub mod server;
pub mod endpoint;

pub mod codegen {
    pub use enzyme_macro::route;
}

pub mod params {
    pub type Params = std::collections::HashMap<&'static str, String>;
}

pub trait HttpError {
    fn code(&self) -> http_types::StatusCode;
    fn msg(&self) -> &serde_json::Value;
}

pub struct Error {
    pub code: http_types::StatusCode,
    pub msg: serde_json::Value,
}

impl HttpError for Error {
    fn code(&self) -> http_types::StatusCode {
        self.code
    }

    fn msg(&self) -> &serde_json::Value {
        &self.msg
    }
}

