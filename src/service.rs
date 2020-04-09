use crate::{endpoint::Endpoint, params::Params, error::Error};
use std::{future::Future, pin::Pin};

/// A trait implemented by functions that can be used as services.  
pub trait Service<Body, Res, E>: 'static {
    type Fut: Future<Output = http_types::Response> + Send + Sync + 'static;

    fn call(&self, _: http_types::Request, _: Params, _: E) -> Self::Fut
    where
        E: Endpoint<Body, Res>;
}

/// A blanket impl over async functions.  
impl<Body, Res, F, G, E> Service<Body, Res, E> for F
where
    F: Fn(http_types::Request, Params, E) -> G + 'static,
    G: Future<Output = Result<http_types::Response, Error>> + Send + Sync + 'static,
    E: Endpoint<Body, Res>,
{
    type Fut = Pin<Box<dyn Future<Output = http_types::Response> + Send + Sync>>;
    fn call(&self, req: http_types::Request, params: Params, endpoint: E) -> Self::Fut {
        let fut = (self)(req, params, endpoint);
        Box::pin(async move { match fut.await {
            Ok(res) => res,
            Err(e) => {
                let mut res = http_types::Response::new(e.code);
                let bytes = serde_json::to_vec(&e.msg).unwrap();
                res.set_body(bytes);
                res
            }
        }})
    }
}
