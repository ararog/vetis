use std::{future::Future, sync::Arc};

use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;
use hyper::body::Incoming;

#[cfg(feature = "http2")]
use crate::server::Server;
use crate::server::{config::ServerConfig, errors::VetisError};

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
        Vetis {
            config,
            instance: None,
        }
    }

    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub async fn start<F, Fut>(&mut self, handler: F) -> Result<(), VetisError>
    where
        F: Fn(RequestType) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ResponseType, VetisError>> + Send + 'static,
    {
        let mut server = server::http::HttpServer::new(self.config.clone());
        server.start(handler).await?;
        self.instance = Some(server);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), VetisError> {
        if let Some(instance) = &mut self.instance {
            instance.stop().await?;
        } else {
            return Err(VetisError::NoInstances);
        }
        Ok(())
    }
}
