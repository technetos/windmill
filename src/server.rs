use crate::router::Router;

use async_std::prelude::*;
use async_std::net::{self, TcpStream};
use async_std::task::{self, Context, Poll};
use async_std::io::{self, Read, Write};
use async_h1::Exception;

use http_types::{Request, Response};
use std::future::Future;
use std::{
    pin::Pin,
    sync::Arc,
};

pub struct Config {
    sock_addr: SocketAddr,
}

//impl Config {
//    pub fn new(addr: &str) -> Result<Self, Exception> {
//        Ok(Self {
//            sock_addr: addr.parse()?,
//        })
//    }
//
//    async fn into_socket_addr(self) -> SocketAddr {
//        self.sock_addr
//    }
//}

pub struct Server {
    router: Arc<Router>,
}

impl Server {
    pub fn new(router: Router) -> Self {
        Self {
            router: Arc::new(router),
        }
    }

    pub fn run(self, config: Config) -> Result<(), Exception> {
        task::block_on(async {
            let listener = net::TcpListener::bind("127.0.0.1:4000").await?;
            let addr = format!("http://{}", listener.local_addr()?);
            println!("listening on {}", addr);
            let mut incoming = listener.incoming();

            while let Some(stream) = incoming.next().await {
                let stream = stream?;
                let addr = addr.clone();
                task::spawn(async {
                    if let Err(err) = accept(addr, stream, self.router.clone()).await {
                        eprintln!("{}", err);
                    }
                });
            }
            Ok(())
        });
        Ok(())
    }
}

async fn accept(
    addr: String,
    stream: TcpStream,
    router: Arc<Router>,
) -> Result<(), Exception> {
    // println!("starting new connection from {}", stream.peer_addr()?);

    // TODO: Delete this line when we implement `Clone` for `TcpStream`.
    let stream = Stream(Arc::new(stream));

    async_h1::server::accept(&addr, stream.clone(), stream, |req| {
        async {
            let response = router.clone().lookup(&mut req).await?;
            Ok(response)
            //            let resp = Response::new(StatusCode::Ok)
            //                .set_header("Content-Type", "text/plain")?
            //                .set_body_string("Hello".into())?;
            // To try chunked encoding, replace `set_body_string` with the following method call
            // .set_body(io::Cursor::new(vec![
            //     0x48u8, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21,
            // ]));
            //            Ok(resp)
        }
    })
    .await
}

#[derive(Clone)]
struct Stream(Arc<TcpStream>);

impl Read for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_read(cx, buf)
    }
}

impl Write for Stream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.0).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.0).poll_close(cx)
    }
}
