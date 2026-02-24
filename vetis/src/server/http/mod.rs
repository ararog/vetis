use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use http::HeaderMap;
use http_body_util::{combinators::BoxBody, BodyExt, Either, StreamBody};
use hyper::body::{Frame, Incoming};

use futures_util::{stream, TryStreamExt};

#[cfg(feature = "smol-rt")]
use futures_lite::AsyncReadExt;
#[cfg(feature = "smol-rt")]
use smol::fs::File;

#[cfg(feature = "tokio-rt")]
use tokio::fs::File;
#[cfg(feature = "tokio-rt")]
use tokio_util::io::ReaderStream;

use crate::{
    config::server::{Protocol, ServerConfig},
    errors::VetisError,
    server::{
        conn::listener::{Listener, ServerListener},
        Server,
    },
    VetisRwLock, VetisVirtualHosts,
};

mod request;
mod response;

pub use crate::server::http::{request::Request, response::Response};

pub type VetisBody = Either<Incoming, BoxBody<Bytes, std::io::Error>>;

pub trait VetisBodyExt {
    fn body_from_text(text: &str) -> VetisBody;
    fn body_from_bytes(bytes: &[u8]) -> VetisBody;
    fn body_from_file(file: File) -> VetisBody;
}

impl VetisBodyExt for VetisBody {
    fn body_from_text(text: &str) -> VetisBody {
        Self::body_from_bytes(text.as_bytes())
    }

    fn body_from_bytes(bytes: &[u8]) -> VetisBody {
        let all_bytes = Bytes::copy_from_slice(bytes);
        let content = stream::iter(vec![Ok(all_bytes)]).map_ok(Frame::data);
        let body = StreamBody::new(content);
        Either::Right(BodyExt::boxed(body))
    }

    fn body_from_file(file: File) -> VetisBody {
        #[cfg(feature = "tokio-rt")]
        let content = ReaderStream::new(file).map_ok(Frame::data);
        #[cfg(feature = "smol-rt")]
        let content = file
            .bytes()
            .map_ok(|data| Frame::data(bytes::Bytes::copy_from_slice(&[data])));
        let body = StreamBody::new(content);
        Either::Right(BodyExt::boxed(body))
    }
}

pub struct HttpServer {
    config: ServerConfig,
    listeners: Vec<ServerListener>,
    virtual_hosts: VetisVirtualHosts,
}

impl Server for HttpServer {
    /// Create a new server instance
    ///
    /// # Arguments
    ///
    /// * `config` - A `ServerConfig` instance containing the server configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - A new `HttpServer` instance.
    fn new(config: ServerConfig) -> Self {
        Self {
            config,
            listeners: Vec::new(),
            virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())),
        }
    }

    /// Set the virtual hosts for the server.
    ///
    /// # Arguments
    ///
    /// * `virtual_hosts` - A `VetisVirtualHosts` instance containing the virtual host registry.
    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts) {
        self.virtual_hosts = virtual_hosts;
    }

    /// Start the server.
    ///
    /// # Returns
    ///
    /// * `Result<(), VetisError>` - A result containing `()` if the server started successfully, or a `VetisError` if the server failed to start.
    async fn start(&mut self) -> Result<(), VetisError> {
        let mut listeners: Vec<ServerListener> = self
            .config
            .listeners()
            .iter()
            .map(|listener_config| match listener_config.protocol() {
                #[cfg(feature = "http1")]
                Protocol::Http1 => {
                    let mut listener = ServerListener::new(listener_config.clone());
                    listener.set_virtual_hosts(
                        self.virtual_hosts
                            .clone(),
                    );
                    listener
                }
                #[cfg(feature = "http2")]
                Protocol::Http2 => {
                    let mut listener = ServerListener::new(listener_config.clone());
                    listener.set_virtual_hosts(
                        self.virtual_hosts
                            .clone(),
                    );
                    listener
                }
                #[cfg(feature = "http3")]
                Protocol::Http3 => {
                    let mut listener = ServerListener::new(listener_config.clone());
                    listener.set_virtual_hosts(
                        self.virtual_hosts
                            .clone(),
                    );
                    listener
                }
                _ => {
                    panic!("Unsupported protocol");
                }
            })
            .collect();

        for listener in listeners.iter_mut() {
            listener
                .listen()
                .await?;
        }

        self.listeners = listeners;

        Ok(())
    }

    /// Stop the server.
    ///
    /// # Returns
    ///
    /// * `Result<(), VetisError>` - A result containing `()` if the server stopped successfully, or a `VetisError` if the server failed to stop.
    async fn stop(&mut self) -> Result<(), VetisError> {
        for listener in self
            .listeners
            .iter_mut()
        {
            listener
                .stop()
                .await?;
        }
        Ok(())
    }
}

// TODO: Move to utils, try make it more flexible
pub fn static_response(
    status: http::StatusCode,
    headers: Option<HeaderMap>,
    body: String,
) -> http::Response<VetisBody> {
    let mut response = http::Response::builder()
        .status(status)
        .body(VetisBody::body_from_text(&body))
        .unwrap();

    if let Some(headers) = headers {
        *response.headers_mut() = headers;
    }

    response
}
