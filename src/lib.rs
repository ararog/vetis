#[cfg(all(
    any(feature = "http2", feature = "http3"),
    not(any(feature = "tokio-rust-tls", feature = "smol-rust-tls"))
))]
compile_error!("http2 and http3 requires tokio-rust-tls or smol-rust-tls!");

#[cfg(all(feature = "tokio-rt", feature = "smol-rt"))]
compile_error!("Only one runtime feature can be enabled at a time.");

use std::future::Future;

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;

#[cfg(any(feature = "http1", feature = "http2"))]
use hyper::body::Incoming;
use log::info;

#[cfg(feature = "smol-rt")]
use async_signal::Signals;
#[cfg(feature = "smol-rt")]
use futures_lite::prelude::*;
#[cfg(feature = "smol-rt")]
use signal_hook::low_level;

use crate::server::{config::ServerConfig, errors::VetisError, Server};

mod rt;
pub mod server;
mod tests;

#[cfg(any(feature = "http1", feature = "http2"))]
pub type RequestType = Request<Incoming>;

#[cfg(feature = "http3")]
pub type RequestType = Request<Full<Bytes>>;

#[cfg(any(feature = "http1", feature = "http2"))]
pub type ResponseType = Response<Full<Bytes>>;

#[cfg(feature = "http3")]
pub type ResponseType = Response<Full<Bytes>>;

pub struct Vetis {
    config: ServerConfig,
    #[cfg(feature = "http1")]
    instance: Option<server::http::HttpServer>,
    #[cfg(feature = "http2")]
    instance: Option<server::http::HttpServer>,
    #[cfg(feature = "http3")]
    instance: Option<server::quic::HttpServer>,
}

impl Vetis {
    pub fn new(config: ServerConfig) -> Vetis {
        Vetis { config, instance: None }
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub async fn run<F, Fut>(&mut self, handler: F) -> Result<(), VetisError>
    where
        F: Fn(RequestType) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ResponseType, VetisError>> + Send + 'static,
    {
        self.start(handler)
            .await?;

        info!(
            "Server listening on port {}:{}",
            self.config
                .interface(),
            self.config.port()
        );

        #[cfg(feature = "tokio-rt")]
        let _ = tokio::signal::ctrl_c().await;

        #[cfg(feature = "smol-rt")]
        {
            use async_signal::Signal;

            let mut signals = Signals::new([Signal::Quit]).unwrap();
            while let Some(signal) = signals.next().await {
                low_level::emulate_default_handler(signal.unwrap() as i32).unwrap();
            }
        }

        info!("\nStopping server...");

        self.stop().await?;

        Ok(())
    }

    pub async fn start<F, Fut>(&mut self, handler: F) -> Result<(), VetisError>
    where
        F: Fn(RequestType) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ResponseType, VetisError>> + Send + 'static,
    {
        #[cfg(any(feature = "http1", feature = "http2"))]
        let mut server = server::http::HttpServer::new(self.config.clone());

        #[cfg(feature = "http3")]
        let mut server = server::quic::HttpServer::new(self.config.clone());

        server
            .start(handler)
            .await?;
        self.instance = Some(server);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), VetisError> {
        if let Some(instance) = &mut self.instance {
            instance
                .stop()
                .await?;
        } else {
            return Err(VetisError::NoInstances);
        }
        Ok(())
    }
}
