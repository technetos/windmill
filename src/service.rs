use crate::{context::Context, endpoint::Endpoint, error::Error, params::Params, state::State};
use std::{future::Future, pin::Pin};

/// A trait implemented by functions that can be used as services.  
pub trait Service: Sized {
    type Fut: Future<Output = Result<Self, Error>> + Unpin + Send + Sync + 'static;

    fn call(_: std::sync::Arc<http_types::Request>, _: std::sync::Arc<Params>) -> Self::Fut;
}

///// A blanket impl over async functions.  
//impl<Body, Res, F, G, E> Service<Body, Res, E> for F
//where
//    F: Fn(http_types::Request, Params) -> G + 'static,
//    G: Future<Output = Result<http_types::Response, Error>> + Send + Sync + 'static,
//    E: Endpoint<Body, Res>,
//{
//    type Fut = Pin<Box<dyn Future<Output = http_types::Response> + Send + Sync>>;
//    fn call(
//        &self,
//        req: http_types::Request,
//        params: Params,
//        endpoint: E,
//        context: Context,
//    ) -> Self::Fut {
//        let fut = (self)(req, params, endpoint, context);
//        Box::pin(async move {
//            match fut.await {
//                Ok(res) => res,
//                Err(e) => {
//                    let mut res = http_types::Response::new(e.code);
//                    let bytes = serde_json::to_vec(&e.msg).unwrap();
//                    res.set_body(bytes);
//                    res
//                }
//            }
//        })
//    }
//}
