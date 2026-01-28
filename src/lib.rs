#[cfg(all(
    any(feature = "http2", feature = "http3"),
    not(any(feature = "tokio-rust-tls", feature = "smol-rust-tls"))
))]
compile_error!("http2 and http3 requires tokio-rust-tls or smol-rust-tls!");

#[cfg(all(feature = "tokio-rt", feature = "smol-rt"))]
compile_error!("Only one runtime feature can be enabled at a time.");

use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
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

pub(crate) type VetisVirtualHosts = Arc<VetisRwLock<HashMap<(String, u16), Box<dyn VirtualHost>>>>;

use crate::{
    config::ServerConfig,
    errors::VetisError,
    server::{virtual_host::VirtualHost, Server},
};

pub mod config;
pub mod errors;
mod rt;
pub mod server;
mod tests;

pub struct Vetis {
    config: ServerConfig,
    virtual_hosts: VetisVirtualHosts,
    instance: Option<server::http::HttpServer>,
}

impl Vetis {
    pub fn new(config: ServerConfig) -> Vetis {
        Vetis { config, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())), instance: None }
    }

    pub async fn add_virtual_host<V>(&mut self, virtual_host: V)
    where
        V: VirtualHost,
    {
        let key = (virtual_host.hostname(), virtual_host.port());

        self.virtual_hosts
            .write()
            .await
            .insert(key, Box::new(virtual_host));
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub async fn run(&mut self) -> Result<(), VetisError> {
        self.start().await?;

        for listener in self
            .config
            .listeners()
        {
            info!("Server listening on port {}:{}", listener.interface(), listener.port());
        }

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

        let mut server = server::http::HttpServer::new(self.config.clone());

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

pub struct Request {
    pub(crate) inner_http: Option<http::Request<Incoming>>,
    pub(crate) inner_quic: Option<http::Request<Full<Bytes>>>,
}

impl Request {
    pub fn from_http(req: http::Request<Incoming>) -> Self {
        Self { inner_http: Some(req), inner_quic: None }
    }

    pub fn from_quic(req: http::Request<Full<Bytes>>) -> Self {
        Self { inner_http: None, inner_quic: Some(req) }
    }

    pub fn uri(&self) -> &http::Uri {
        match &self.inner_http {
            Some(req) => req.uri(),
            None => match &self.inner_quic {
                Some(req) => req.uri(),
                None => panic!("No request"),
            },
        }
    }

    pub fn headers(&self) -> &http::HeaderMap {
        match &self.inner_http {
            Some(req) => req.headers(),
            None => match &self.inner_quic {
                Some(req) => req.headers(),
                None => panic!("No request"),
            },
        }
    }

    pub fn method(&self) -> &http::Method {
        match &self.inner_http {
            Some(req) => req.method(),
            None => match &self.inner_quic {
                Some(req) => req.method(),
                None => panic!("No request"),
            },
        }
    }
}

pub struct ResponseBuilder {
    status: http::StatusCode,
    version: http::Version,
    headers: http::HeaderMap,
}

impl ResponseBuilder {
    pub fn status(mut self, status: http::StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn version(mut self, version: http::Version) -> Self {
        self.version = version;
        self
    }

    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    pub fn body(self, body: Full<Bytes>) -> Response {
        let response = http::Response::builder()
            .status(self.status)
            .version(self.version);

        Response {
            inner: response
                .body(body)
                .unwrap(),
        }
    }
}

pub struct Response {
    pub(crate) inner: http::Response<Full<Bytes>>,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder {
            status: http::StatusCode::OK,
            version: http::Version::HTTP_11,
            headers: http::HeaderMap::new(),
        }
    }

    pub fn into_inner(self) -> http::Response<Full<Bytes>> {
        self.inner
    }
}
