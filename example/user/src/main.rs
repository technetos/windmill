#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

mod context;
mod message;
mod service;
mod user;

use context::{AuthContext, TokenContext};
use message::{LogoutRequest, TokenRequest};
use service::USER_SERVICE;

use enzyme::{
    macros::route,
    router::Router,
    server::{Config, Server},
};
use http::method::Method;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = |cx: TokenContext, req: TokenRequest| USER_SERVICE.token(cx, req);
    let logout = |cx: AuthContext, req: LogoutRequest| USER_SERVICE.logout(cx, req);

    let router = {
        let mut router = Router::new();

        router.add(Method::POST, route!(/"token" => token));
        router.add(Method::POST, route!(/"logout" => logout));
        router
    };

    let config = Config::new("127.0.0.1:3000")?;
    Ok(Server::new(router).run(config))
}
