pub mod endpoint;
pub mod error;
pub mod context;

pub type WebResult<T> = Result<T, error::WebError>;

pub mod prelude {
    pub use super::WebResult;
    pub use super::endpoint::Endpoint;
    pub use super::error::WebError;
}
