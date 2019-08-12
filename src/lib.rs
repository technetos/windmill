use futures::future;
use futures::future::Future;
use serde::{Deserialize, Serialize};

pub struct Error {
    msg: String,
}

pub struct Context {}

pub struct Request<T> {
    context: Context,
    data: T,
}

/// A trait representing an async conversion into T
pub trait AsyncInto<T> {
    fn async_into<'a>(self) -> Box<dyn Future<Output = Result<T, Error>> + Unpin + 'a>;
}

pub trait IntoRequest<'a, T: 'a>: AsyncInto<Request<T>> {
    fn into_request(self) -> Box<dyn Future<Output = Result<Request<T>, Error>> + Unpin + 'a>;
}

impl<'a, T: 'a> IntoRequest<'a, T> for T
where
    T: for<'de> Deserialize<'de> + Serialize + AsyncInto<Request<T>>,
{
    fn into_request(self) -> Box<dyn Future<Output = Result<Request<T>, Error>> + Unpin + 'a> {
        Box::new(self.async_into())
    }
}

#[test]
fn test() {
    struct GetUserId;

    impl AsyncInto<Request<GetUserId>> for GetUserId {
        fn async_into<'a>(self) -> Box<dyn Future<Output = Result<Request<GetUserId>, Error>> + Unpin + 'a> {
            Box::new(future::ready(Ok(Request {
                context: Context {},
                data: self
            })))
        }
    }
}
