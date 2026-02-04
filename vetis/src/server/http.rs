use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use http_body_util::{Either, Full};
use hyper::body::Incoming;

use crate::{
    config::{Protocol, ServerConfig},
    errors::VetisError,
    server::{
        conn::listener::{Listener, ServerListener},
        Server,
    },
    VetisRwLock, VetisVirtualHosts,
};

pub struct HttpServer {
    config: ServerConfig,
    listeners: Vec<ServerListener>,
    virtual_hosts: VetisVirtualHosts,
}

impl Server for HttpServer {
    fn new(config: ServerConfig) -> Self {
        Self {
            config,
            listeners: Vec::new(),
            virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())),
        }
    }

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts) {
        self.virtual_hosts = virtual_hosts;
    }

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
    body: String,
) -> http::Response<Either<Incoming, Full<Bytes>>> {
    http::Response::builder()
        .status(status)
        .body(Either::Right(Full::new(Bytes::from(body))))
        .unwrap()
}
