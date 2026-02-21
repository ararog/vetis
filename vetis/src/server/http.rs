use std::{collections::HashMap, sync::Arc};

use http::HeaderMap;

use crate::{
    config::server::{Protocol, ServerConfig},
    errors::VetisError,
    server::{
        conn::listener::{Listener, ServerListener},
        Server,
    },
    VetisBody, VetisBodyExt, VetisRwLock, VetisVirtualHosts,
};

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
