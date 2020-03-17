use crate::{error::Error, req::Req};
use std::{future::Future, pin::Pin};

/// A trait for things that can be used as routes.  
pub trait Endpoint<Body, Res>: 'static + Copy {
    type Fut: Future<Output = Result<Res, Error>> + Send + Sync;
    fn call(&self, req: Req<Body>) -> Self::Fut;
}

/// A blanket impl over async functions.  
impl<Body, Res, F, G> Endpoint<Body, Res> for F
where
    F: Fn(Req<Body>) -> G + Copy + 'static,
    G: Future<Output = Result<Res, Error>> + Send + Sync + 'static,
{
    type Fut = Pin<Box<dyn Future<Output = Result<Res, Error>> + Send + Sync>>;

    fn call(&self, req: Req<Body>) -> Self::Fut {
        let fut = (self)(req);
        Box::pin(async move { fut.await })
    }
}
