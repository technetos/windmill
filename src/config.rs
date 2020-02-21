/// Configuration for the server.  
pub struct Config {
    addr: String,
}

impl Config {
    /// Create a new instance of `Config` with the address the server should bind to.  
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    /// Get the address that the server is running on.  
    pub fn addr(&self) -> &str {
        &self.addr
    }
}
