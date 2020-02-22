use crate::{params::Params, req::Req, error::Error};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

/// A trait implemented by functions that can be used as routes.  
pub trait Endpoint<Body, Res>: 'static + Copy
where
    Body: for<'de> Deserialize<'de> + 'static + Send,
    Res: Serialize + 'static,
{
    type Fut: Future<Output = Result<Res, Error>> + Send + Sync;
    fn call(&self, req: Req<Body>) -> Self::Fut;
}

/// A blanket impl over async functions.  
impl<Body, Res, F, G> Endpoint<Body, Res> for F
where
    Body: for<'de> Deserialize<'de> + 'static + Send,
    Res: Serialize + 'static + Send,
    G: Future<Output = Result<Res, Error>> + 'static + Send + Sync,
    F: Fn(Req<Body>) -> G + 'static + Copy,
{
    type Fut = Pin<Box<dyn Future<Output = Result<Res, Error>> + Send + Sync>>;
    /// Invoke the endpoint passing in the request.  
    fn call(&self, req: Req<Body>) -> Self::Fut {
        let fut = (self)(req);
        Box::pin(async move { fut.await })
    }
}

pub(crate) async fn json_endpoint<Body, Res>(
    mut req: http_types::Request,
    params: Params,
    endpoint: impl Endpoint<Body, Res> + Send + Sync,
) -> http_types::Response
where
    Body: for<'de> Deserialize<'de> + 'static + Send + Sync,
    Res: Serialize + 'static + Send + Sync,
{
    use async_std::prelude::*;
    use http_types::{headers, StatusCode};

    let has_body = req
        .header(&headers::CONTENT_LENGTH)
        .map(|header_values| header_values.first().map(|value| value.as_str() != "0"))
        .flatten()
        .unwrap_or_else(|| false);

    let mut body = vec![];
    if has_body {
        let _ = req.read_to_end(&mut body).await;
    }

    let req_body: Option<Body> = serde_json::from_slice(&body).unwrap_or_else(|_| None);

    let res_body: Res = match endpoint.call(Req::new(req, req_body, params)).await {
        Ok(res_body) => res_body,
        Err(e) => {
            let mut res = http_types::Response::new(e.code());
            let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
            res.set_body(serde_json::to_vec(e.msg()).unwrap());
            return res;
        }
    };

    let res_body_bytes = match serde_json::to_vec(&res_body) {
        Ok(res_body_bytes) => res_body_bytes,
        Err(e) => {
            let mut res = http_types::Response::new(StatusCode::InternalServerError);
            let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
            res.set_body(serde_json::to_vec(&format!("{}", e)).unwrap());
            return res;
        }
    };

    let mut res = http_types::Response::new(StatusCode::Ok);
    res.set_body(res_body_bytes);
    let _ = res.insert_header(http_types::headers::CONTENT_TYPE, "application/json");
    res
}
