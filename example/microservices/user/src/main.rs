#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

use enzyme::{
    error::WebError,
    macros::{route, Context},
    params::Params,
    result::WebResult,
    router::Router,
    server::Server,
};
use http::{method::Method, request::Parts, status::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

fn main() {
    let token = |cx: TokenContext, req: TokenRequest| USER.token(cx, req);
    let logout = |cx: AuthContext, req: LogoutRequest| USER.logout(cx, req);

    let router = {
        let mut router = Router::new();

        router.add(Method::POST, route!(/"token" => token));
        router.add(Method::POST, route!(/"logout" => logout));
        router
    };

    Server::new(router).run()
}
