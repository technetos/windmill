#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

use http_types::{Method, Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use windmill::*;

lazy_static! {
    static ref ENV_VARS: Result<EnvVars, envy::Error> = envy::from_env::<EnvVars>();
}

fn main() {
    // read variables to memoize
    let _ = ENV_VARS.clone();

    let mut router = Router::new();
    let config = Config::new("127.0.0.1:4000");

    #[rustfmt::skip]
        router.add(Method::Get, route!(/"main"), ___my_main_handler);

    if let Err(e) = Server::new(config).run(router) {
        println!("{}", e);
    }
}

#[endpoint]
async fn my_main_handler(env: EnvVarsProps) -> Result<http_types::Response, Error> {
    println!("env vars: {:?}", env);
    let mut response = http_types::Response::new(http_types::StatusCode::Ok);
    let env_var_json = serde_json::to_string(env.env_vars).map_err(|e| Error {
        code: StatusCode::InternalServerError,
        msg: serde_json::json!(&format!("{}", e)),
    })?;
    let body = http_types::Body::from(env_var_json);
    response.set_body(body);
    Ok(response)
}

#[derive(Clone, Deserialize, Debug, Serialize)]
struct EnvVars {
    foo: u16,
    bar: bool,
}

#[derive(Debug)]
struct EnvVarsProps {
    env_vars: &'static EnvVars,
}

impl Props for EnvVarsProps {
    type Fut = PropsFuture<Self>;
    fn call(req: Request, params: Params) -> Self::Fut {
        Box::pin(async move {
            let props = Self {
                env_vars: ENV_VARS.as_ref().map_err(|e| Error {
                    code: StatusCode::InternalServerError,
                    msg: serde_json::json!(&format!("{}", e)),
                })?,
            };
            Ok((req, params, props))
        })
    }
}
