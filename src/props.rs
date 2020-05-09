use crate::{error::Error, params::Params};
use std::{future::Future, pin::Pin};

pub type PropsFuture<T> = Pin<
    Box<
        dyn Future<Output = Result<(http_types::Request, Params, T), Error>>
            + Send
            + Sync
            + 'static,
    >,
>;

/// A trait implemented by functions that can be used as props.  
pub trait Props: Sized {
    type Fut: Future<Output = Result<(http_types::Request, Params, Self), Error>>
        + Unpin
        + Send
        + Sync;

    fn call(_: http_types::Request, _: Params) -> Self::Fut;
}
