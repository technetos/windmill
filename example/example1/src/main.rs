#![feature(proc_macro_hygiene)]

use windmill::*;

use http_types::{Method, StatusCode};
use serde::Deserialize;
use std::pin::Pin;

#[derive(Deserialize)]
struct ExampleRequest {
    foo: String,
}

fn main() {
    let mut router = Router::new();
    let config = Config::new("127.0.0.1:4000");

    #[rustfmt::skip]
    router.add(Method::Get, route!(/"example"/id), ___example_route); 

    if let Err(e) = Server::new(config).run(router) {
        println!("{}", e);
    }
}

use http_types::{headers::HeaderName, mime, Mime,}; 
use serde_json::json;
use std::future::Future;

struct Auth {
    user_id: u64,
    token: String,
}

type ResponseFuture<T> = Pin<Box<dyn Future<Output = Result<T, Error>> + Send + Sync>>;

impl Service for Auth {
    type Fut = ResponseFuture<Self>;
    fn call(req: std::sync::Arc<http_types::Request>, params: std::sync::Arc<Params>) -> Self::Fut {
        Box::pin(async move {
            use std::str::FromStr;
            let header_name = HeaderName::from_str("authorization").map_err(|header_name| Error {
                code: StatusCode::InternalServerError,
                msg: json!("bad header name"),
            })?;

            let header = req.header(&header_name).ok_or_else(|| Error {
                code: StatusCode::BadRequest,
                msg: json!("authorization required"),
            })?;

            let header = header.first().unwrap();

            Ok(Self {
                user_id: 1,
                token: String::new(),
            })
        })
    }
}

struct Body<T> {
    t: T,
}

impl Service for Body<ExampleRequest> {
    type Fut = ResponseFuture<Self>;
    fn call(req: std::sync::Arc<http_types::Request>, params: std::sync::Arc<Params>) -> Self::Fut {
        Box::pin(async {
            Ok(Body {
                t: ExampleRequest {
                    foo: String::new(),
                }
            })
        })
    }
}

#[endpoint]
async fn example_route(auth: Auth, body: Body<ExampleRequest>) -> Result<http_types::Response, Error> {
//    use std::str::FromStr;
//    let id = u64::from_str(req.params().get("id").ok_or_else(|| Error {
//        code: StatusCode::InternalServerError,
//        msg: serde_json::json!("param does not exist"),
//    })?)
//    .map_err(|e| Error {
//        code: StatusCode::BadRequest,
//        msg: serde_json::json!(format!("{}", e)),
//    })?;
//
//    //    let body = req.body().ok_or_else(|| Error {
//    //        code: StatusCode::BadRequest,
//    //        msg: serde_json::json!("body required"),
//    //    })?;
//
//    let auth = req
//        .context()
//        .get("auth")
//        .ok_or_else(|| Error {
//            code: StatusCode::InternalServerError,
//            msg: serde_json::json!("context value auth does not exist"),
//        })?
//        .downcast_ref::<http_types::headers::HeaderValue>()
//        .ok_or_else(|| Error {
//            code: StatusCode::InternalServerError,
//            msg: serde_json::json!("invalid cast to HeaderValue"),
//        })?;
//
    Ok(http_types::Response::new(StatusCode::Ok))
}
