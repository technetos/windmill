use bytes::Bytes;
use futures::future;
use futures::future::Future;
use futures::future::FutureExt;
use http::{Request, Response};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

pub type Error = Box<dyn std::error::Error + Send>;

pub type BytesF = Box<dyn Future<Output = Result<Bytes, Error>> + Unpin + Send + 'static>;

pub type ResponseF<T> = Box<dyn Future<Output = Result<T, Error>> + Send + Unpin>;

pub struct Action {
    f: Box<dyn Fn(TcpStream) -> BytesF>,
}

impl Action {
    pub fn new(f: impl Fn(TcpStream) -> BytesF + 'static) -> Self {
        Self { f: Box::new(f) }
    }
}

pub struct Context;

pub struct Endpoint;

impl Endpoint {
    pub fn new<Req: 'static, Res: 'static>(f: fn(Req) -> ResponseF<Res>) -> Action
    where
        Req: for<'de> Deserialize<'de>,
        Res: Serialize,
    {
        Action::new(move |stream: TcpStream| -> BytesF {
            let mut request_bytes = Vec::new();
            let mut stream = BufReader::new(stream);

            loop {
                request_bytes.clear();

                // really b'\n' should be b"\r\n" but i dont know if that works
                let bytes_read = stream.read_until(b'\n', &mut request_bytes).unwrap();
                if bytes_read == 0 {
                    break;
                }
            }

            match serde_json::from_slice(&request_bytes) {
                Ok(req) => {
                    let res = f(req);
                    Box::new(res.then(|_| {
                        let mut bytes = Bytes::new();
                        bytes.extend_from_slice(b"test response bytes string");
                        futures::future::ok(bytes)
                    }))
                }
                Err(e) => {
                    // Respond with error
                    dbg!(e);
                    panic!();
                }
            }
        })
    }
}

#[test]
fn test() {
    use crate::Endpoint;

    mod messages {
        use serde::{Deserialize, Serialize};

        #[derive(Deserialize)]
        pub struct TestRequest;

        #[derive(Serialize)]
        pub struct TestResponse;

        #[derive(Deserialize)]
        pub struct LogoutRequest;

        #[derive(Serialize)]
        pub struct LogoutResponse;
    }

    struct TestEndpoint;

    use messages::{LogoutRequest, LogoutResponse, TestRequest, TestResponse};

    impl TestEndpoint {
        pub fn get_token(req: TestRequest) -> ResponseF<TestResponse> {
            Box::new(future::ready(Ok(TestResponse)))
        }

        pub fn logout(req: LogoutRequest) -> ResponseF<LogoutResponse> {
            Box::new(future::ready(Ok(LogoutResponse)))
        }
    }

    // Wiring
    let test_service = Endpoint::new(TestEndpoint::get_token);
    let logout_service = Endpoint::new(TestEndpoint::logout);
}
