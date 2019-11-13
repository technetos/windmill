pub mod context;
pub mod endpoint;
pub mod error;
pub mod router;
pub mod server;

pub mod macros {
    pub use enzyme_macro::route;
}

pub mod result {
    pub type WebResult<T> = Result<T, crate::error::WebError>;
}

pub mod params {
    pub type Params = std::collections::HashMap<&'static str, String>;
}

// Not accessible from outside this crate.  Here only while we still depend on hyper/http-service.
#[rustfmt::skip]
mod ready {
    pub struct Ready<T>(Option<T>);

    impl<T> Unpin for Ready<T> {}

    impl<T> std::future::Future for Ready<T> {
        type Output = T;

        #[inline]
        fn poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<T> {
            std::task::Poll::Ready(self.0.take().unwrap())
        }
    }

    pub(crate) fn ready<T>(t: T) -> Ready<T> {
        Ready(Some(t))
    }
}
