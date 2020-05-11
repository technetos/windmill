use crate::{config::Config, router::Router};
use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use http_types::Error;
use std::sync::Arc;

/// The server that listens for requests.  
pub struct Server {
    config: Config,
}

impl Server {
    /// Create a new `Server`.
    ///
    /// ## Examples
    /// ```
    /// # use windmill::{Config, Server};
    /// let config = Config::new("127.0.0.1:4000");
    /// let server = Server::new(config);
    /// ```
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Start accepting requests on the server using the provided router.  
    ///
    /// ## Examples
    /// ```no_run
    /// # use windmill::{Config, Router, Server};
    /// # use async_std::net::TcpListener;
    /// let mut router = Router::new();
    /// let config = Config::new("127.0.0.1:4000");
    ///
    /// if let Err(e) = Server::new(config).run(router) {
    ///     println!("{}", e);
    /// }
    /// ```
    pub fn run(self, router: Router) -> Result<(), Box<dyn std::error::Error>> {
        let router = Arc::new(router);
        Ok(task::block_on(async {
            let listener = TcpListener::bind(self.config.addr())
                .await
                .map_err(|e| format!("Unable to bind to tcp socket: {}", e))?;

            let addr = format!("http://{}", listener.local_addr()?);
            println!("listening on {}", addr);

            let mut incoming = listener.incoming();

            while let Some(stream) = incoming.next().await {
                let router = router.clone();
                let addr = addr.clone();
                let stream = stream?;
                task::spawn(async {
                    if let Err(err) = accept(addr, stream, router).await {
                        eprintln!("{}", err);
                    }
                });
            }
            Ok(())
        })
        .map_err(|e: Box<dyn std::error::Error>| format!("Unable to spawn blocking task: {}", e))?)
    }
}

async fn accept(addr: String, stream: TcpStream, router: Arc<Router>) -> Result<(), Error> {
    let router = router.clone();
    async_h1::accept(&addr, stream.clone(), |req| async {
        let response = router.clone().lookup(req).await.await;
        Ok(response)
    })
    .await
}
