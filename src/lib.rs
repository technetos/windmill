pub mod context;
pub mod endpoint;
pub mod error;
pub mod router;
pub mod server;

pub mod macros {
    pub use enzyme_macro::route;
    pub use enzyme_macro::Context;
}

pub mod result {
    pub type WebResult<T> = Result<T, crate::error::WebError>;
}
