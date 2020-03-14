use crate::{error::Error, params::Params, req::Req};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

/// A trait implemented by functions that can be used as routes.  
pub trait Endpoint<Body, Res>: 'static + Copy {
    type Fut: Future<Output = Result<Res, Error>> + Send + Sync;

    fn call(&self, req: Req<Body>) -> Self::Fut
    where
        Body: for<'de> Deserialize<'de> + 'static + Send;
}

/// A blanket impl over async functions.  
impl<Body, Res, F, G> Endpoint<Body, Res> for F
where
    Body: for<'de> Deserialize<'de> + 'static + Send,
    F: Fn(Req<Body>) -> G + 'static + Copy,
    G: Future<Output = Result<Res, Error>> + 'static + Send + Sync,
{
    type Fut = Pin<Box<dyn Future<Output = Result<Res, Error>> + Send + Sync>>;
    
    fn call(&self, req: Req<Body>) -> Self::Fut {
        let fut = (self)(req);
        Box::pin(async move { fut.await })
    }
}

/// A trait implemented by functions that can be used as services.  
pub trait Service<Body, Res, E>: 'static + Copy {
    type Fut: Future<Output = http_types::Response> + Send + Sync;

    fn call(&self, _: http_types::Request, _: Params, _: E) -> Self::Fut
    where
        E: Endpoint<Body, Res>;
}

/// A blanket impl over async functions.  
impl<Body, Res, F, G, E> Service<Body, Res, E> for F
where
    G: Future<Output = http_types::Response> + Send + Sync + 'static,
    E: Endpoint<Body, Res>,
    F: Fn(http_types::Request, Params, E) -> G + Copy + 'static,
{
    type Fut = Pin<Box<dyn Future<Output = http_types::Response> + Send + Sync>>;
    fn call(&self, req: http_types::Request, params: Params, endpoint: E) -> Self::Fut {
        let fut = (self)(req, params, endpoint);
        Box::pin(async move { fut.await })
    }
}
