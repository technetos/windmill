use futures::future;
use futures::future::Future;
use futures::future::FutureExt;
use serde::{Deserialize, Serialize};
use http::{Request, Response};
use std::{net::TcpStream, io::{BufRead, BufReader}};
use bytes::Bytes;

pub type BytesF = Box<Future<Output = Result<Bytes, Error>> + Unpin + Send + 'static>;

pub struct Action {
    f: Box<Fn(TcpStream) -> BytesF>,
}

impl Action {
    pub fn new(f: impl Fn(TcpStream) -> BytesF + 'static) -> Self {
        Self { f: Box::new(f) }
    }
}

pub type Error = Box<std::error::Error>;

pub struct Context;

pub trait Endpoint<Req, Res> {
    fn run(&self, req: Req) -> Box<Future<Output = Result<Res, Error>> + Send + Unpin>
    where
        Req: for<'de> Deserialize<'de>,
        Res: Serialize;
}

pub struct EndpointHandler;

impl EndpointHandler {
    pub fn handle<Req, Res>(endpoint: impl Endpoint<Req, Res>) -> Action
    where
        Req: for<'de> Deserialize<'de>,
        Res: Serialize
    {
        Action::new(|mut stream: TcpStream| -> BytesF {
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

            let req: Req = serde_json::from_slice(&request_bytes).unwrap();
            let res = endpoint.run(req);
            Box::new(res.then(|_| {
                let mut bytes = Bytes::new();
                bytes.extend_from_slice(b"test response bytes string");
                Ok(bytes)
            }))
        })
    }
}

#[test]
fn test() {
    use crate::{Endpoint, EndpointHandler, Error};

    mod messages {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize)]
        pub struct TestRequest;

        #[derive(Serialize)]
        pub struct TestResponse;
    }

    struct TestEndpoint;

    use messages::{TestRequest, TestResponse};

    impl Endpoint<TestRequest, TestResponse> for TestEndpoint {
        fn run(&self, req: TestRequest) -> Box<Future<Output = Result<TestResponse, Error>> + Unpin> {
            Box::new(future::ready(Ok(TestResponse)))
        }
    }

    let handler = EndpointHandler::handle(TestEndpoint);
}
