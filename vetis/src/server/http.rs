use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use futures_lite::stream;
use futures_util::TryStreamExt;
use http_body_util::{BodyExt, Either, Full, StreamBody};
use hyper::body::Frame;

use crate::{
    config::{Protocol, ServerConfig},
    errors::VetisError,
    server::{
        conn::listener::{Listener, ServerListener},
        Server,
    },
    VetisBodyExt, VetisResponseBody, VetisRwLock, VetisVirtualHosts,
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
) -> http::Response<VetisResponseBody> {
    http::Response::builder()
        .status(status)
        .body(VetisResponseBody::body_from_text(&body))
        .unwrap()
}
