#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;

mod context;
mod message;
mod user;

use message::{LogoutRequest, TokenRequest};
use user::{logout, token};

use enzyme::{
    macros::route,
    router::Router,
    server::{Config, Server},
};
use http::method::Method;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let example = |cx: TokenContext, req: LogoutRequest| async { Ok(message::LogoutResponse) };

    let router = {
        let mut router = Router::new();

        router.add(Method::POST, route!(/"token")).mount(token);
        router.add(Method::POST, route!(/"logout")).mount(logout);
        router.add(Method::GET, route!(/"example")).mount(example);
        router
    };

    let config = Config::new("127.0.0.1:4000")?;
    Ok(Server::new(router).run(config))
}
