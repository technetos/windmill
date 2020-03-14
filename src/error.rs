/// A HTTP error.  
pub struct Error {
    pub code: http_types::StatusCode,
    pub msg: serde_json::Value,
}

impl Error {
    /// The HTTP error code.  
    pub fn code(&self) -> http_types::StatusCode {
        self.code
    }

    /// The message describing what went wrong.  
    pub fn msg(&self) -> &serde_json::Value {
        &self.msg
    }
}
