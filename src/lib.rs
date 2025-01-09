mod logger;

use logger::LoggerLayer;

use http_body_util::Full;
use hyper::server::conn::http1::{self};
use hyper::{
    body::{Bytes, Incoming},
    Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

pub struct EchoServer {
    listener: TcpListener,
    logging_enabled: bool,
}

impl EchoServer {
    pub async fn new(logging_enabled: bool, port: u16) -> Result<Self, std::io::Error> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            logging_enabled,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let io = TokioIo::new(stream);
            let svc = tower::ServiceBuilder::new()
                .layer(LoggerLayer::new(self.logging_enabled))
                .service_fn(echo);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, hyper_util::service::TowerToHyperService::new(svc))
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

async fn echo(_request: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::from(Bytes::from("hello"))))
}

