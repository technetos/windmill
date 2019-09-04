pub mod context;
pub mod endpoint;
pub mod error;

pub type WebResult<T> = Result<T, error::WebError>;

pub mod prelude {
    pub use super::context::Context;
    pub use super::endpoint::Endpoint;
    pub use super::error::WebError;
    pub use super::WebResult;
}
