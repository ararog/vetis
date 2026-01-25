#[cfg(all(
    any(feature = "http2", feature = "http3"),
    not(any(feature = "tokio-rust-tls", feature = "smol-rust-tls"))
))]
compile_error!("http2 and http3 requires tokio-rust-tls or smol-rust-tls!");

#[cfg(all(feature = "tokio-rt", feature = "smol-rt"))]
compile_error!("Only one runtime feature can be enabled at a time.");

use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;

#[cfg(any(feature = "http1", feature = "http2"))]
use hyper::body::Incoming;
use log::{error, info};

#[cfg(feature = "smol-rt")]
use async_signal::Signals;
#[cfg(feature = "smol-rt")]
use futures_lite::prelude::*;
#[cfg(feature = "smol-rt")]
use signal_hook::low_level;

#[cfg(feature = "smol-rt")]
use smol::sync::RwLock;

#[cfg(feature = "tokio-rt")]
use tokio::sync::RwLock;

pub(crate) type VetisRwLock<T> = RwLock<T>;

pub(crate) type VetisVirtualHosts =
    Arc<VetisRwLock<HashMap<String, Box<dyn VirtualHost + Send + Sync + 'static>>>>;

use crate::server::{config::ServerConfig, errors::VetisError, virtual_host::VirtualHost, Server};

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
    virtual_hosts: VetisVirtualHosts,

    #[cfg(feature = "http1")]
    instance: Option<server::http::HttpServer>,
    #[cfg(feature = "http2")]
    instance: Option<server::http::HttpServer>,
    #[cfg(feature = "http3")]
    instance: Option<server::quic::HttpServer>,
}

impl Vetis {
    pub fn new(config: ServerConfig) -> Vetis {
        Vetis { config, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())), instance: None }
    }

    pub async fn add_virtual_host(
        &mut self,
        virtual_host: Box<dyn VirtualHost + Send + Sync + 'static>,
    ) {
        self.virtual_hosts
            .write()
            .await
            .insert(
                virtual_host
                    .hostname()
                    .to_string(),
                virtual_host,
            );
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub async fn run(&mut self) -> Result<(), VetisError> {
        self.start().await?;

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

    pub async fn start(&mut self) -> Result<(), VetisError> {
        if self
            .virtual_hosts
            .read()
            .await
            .is_empty()
        {
            error!("You must add at least one virtual host");
            return Err(VetisError::NoVirtualHosts);
        }

        #[cfg(any(feature = "http1", feature = "http2"))]
        let mut server = server::http::HttpServer::new(self.config.clone());

        #[cfg(feature = "http3")]
        let mut server = server::quic::HttpServer::new(self.config.clone());

        server.set_virtual_hosts(
            self.virtual_hosts
                .clone(),
        );

        server
            .start()
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
