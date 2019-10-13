#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

mod message;
mod service;
mod user;

use message::{LogoutRequest, TokenRequest};
use service::USER_SERVICE;
use user::{AuthContext, TokenContext};

use enzyme::{macros::route, router::Router, server::Server};
use http::method::Method;

fn main() {
    let token = |cx: TokenContext, req: TokenRequest| USER_SERVICE.token(cx, req);
    let logout = |cx: AuthContext, req: LogoutRequest| USER_SERVICE.logout(cx, req);

    let router = {
        let mut router = Router::new();

        router.add(Method::POST, route!(/"token" => token));
        router.add(Method::POST, route!(/"logout" => logout));
        router
    };

    Server::new(router).run()
}
