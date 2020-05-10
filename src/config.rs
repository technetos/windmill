/// Configuration for the server.  
pub struct Config {
    addr: String,
}

impl Config {
    /// Create a new instance of `Config` with the address the server should bind to.  
    /// ```
    /// # use windmill::Config;
    /// let config = Config::new("127.0.0.1:4000");
    /// ```
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }

    /// Get the address that the server is running on.  
    /// ```
    /// # use windmill::Config;
    /// let config = Config::new("127.0.0.1:4000");
    /// assert_eq!(config.addr(), "127.0.0.1:4000");
    /// ```
    pub fn addr(&self) -> &str {
        &self.addr
    }
}
