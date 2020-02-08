use crate::{config::Config, router::Router};

use async_std::io::{self, Read, Write};
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task::{self, Context, Poll};

use http_types::Error;
use std::{pin::Pin, sync::Arc};

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config,
        }
    }

    pub fn run(self, router: Arc<Router>) -> Result<(), ()> {
        let _ = task::block_on(async {
            use async_std::net::ToSocketAddrs;
            let listener = TcpListener::bind(self.config.addr()).await.unwrap();
            let addr = format!("http://{}", listener.local_addr().unwrap());
            println!("listening on {}", addr);

            let mut incoming = listener.incoming();

            while let Some(stream) = incoming.next().await {
                let router = router.clone();
                let stream = stream.unwrap();
                let addr = addr.clone();
                task::spawn(async {
                    if let Err(err) = accept(addr, stream, router).await {
                        eprintln!("{}", err);
                    }
                });
            }
            Ok(()).map_err(|_: ()| ())
        });
        Ok(())
    }
}

async fn accept(addr: String, stream: TcpStream, router: Arc<Router>) -> Result<(), Error> {
    println!("starting new connection from {}", stream.peer_addr()?);

    // TODO: Delete this line when we implement `Clone` for `TcpStream`.
    let stream = Stream(Arc::new(stream));

    let router = router.clone();
    async_h1::server::accept(&addr, stream, |req| async {
        let response = router.clone().lookup(req).await.await;
        Ok(response)
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
