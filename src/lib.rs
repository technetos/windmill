use futures::future;
use futures::future::BoxFuture;
use futures::future::Future;
use serde::Deserialize;

pub struct Error {
    msg: String,
}

pub struct Context {}

pub struct Request<T> {
    context: Context,
    data: T,
}

pub trait IntoRequest<T>: Sized {
    fn to_req<
        I: Into<Request<T>>,
        F: Future<Output = Result<Self, Error>>>(
            raw: I
        ) -> F;
}

impl<T> IntoRequest<T> for Request<T>
where
    T: for<'de> Deserialize<'de>,
{
    fn to_req<
        I: Into<Request<T>>,
        F: Future<Output = Result<Request<T>, Error>>>(
            raw: I,
        ) -> F {
            // Returns F, a future whos output is either a
            // Request<T> or an Error

        }
}

#[test]
fn test() {}
