pub mod context;
pub mod endpoint;
pub mod error;
pub mod router;

pub mod macros {
    pub use enzyme_macro::route;
}

pub type WebResult<T> = Result<T, error::WebError>;

pub mod prelude {
    pub use super::context::Context;
    pub use super::endpoint::Endpoint;
    pub use super::error::WebError;
    pub use super::WebResult;
}
