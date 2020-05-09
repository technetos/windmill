use crate::{error::Error, Params};
use std::{future::Future, pin::Pin};

/// A trait for things that can be used as routes.  
pub trait Endpoint: 'static + Copy {
    type Fut: Future<Output = Result<http_types::Response, Error>> + Send + Sync + 'static;
    fn call(&self, req: http_types::Request, params: Params) -> Self::Fut;
}

/// A blanket impl over the generated hidden functions for endpoints.  
impl<F, G> Endpoint for F
where
    F: Fn(http_types::Request, Params) -> G + Copy + 'static,
    G: Future<Output = Result<http_types::Response, Error>> + Send + Sync + 'static,
{
    type Fut = Pin<Box<dyn Future<Output = Result<http_types::Response, Error>> + Send + Sync>>;

    fn call(&self, req: http_types::Request, params: Params) -> Self::Fut {
        let fut = (self)(req, params);
        Box::pin(async move { fut.await })
    }
}
