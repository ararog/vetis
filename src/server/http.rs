use std::{collections::HashMap, sync::Arc};

use rt_gate::GateTask;

#[cfg(any(feature = "http1", feature = "http2"))]
use crate::server::conn::listener::tcp::TcpServerListener;
#[cfg(feature = "http3")]
use crate::server::conn::listener::udp::UdpServerListener;

use crate::{
    config::{Protocol, ServerConfig},
    errors::VetisError,
    server::{conn::listener::ServerListener, Server},
    VetisRwLock, VetisVirtualHosts,
};

pub enum HttpListener {
    #[cfg(any(feature = "http1", feature = "http2"))]
    TCP(TcpServerListener),
    #[cfg(feature = "http3")]
    UDP(UdpServerListener),
}

impl HttpListener {
    pub async fn listen(&mut self) -> Result<(), VetisError> {
        match self {
            #[cfg(any(feature = "http1", feature = "http2"))]
            HttpListener::TCP(ref mut tcp_listener) => {
                tcp_listener
                    .listen()
                    .await?
            }
            #[cfg(feature = "http3")]
            HttpListener::UDP(ref mut udp_listener) => {
                udp_listener
                    .listen()
                    .await?
            }
        }
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), VetisError> {
        match self {
            #[cfg(any(feature = "http1", feature = "http2"))]
            HttpListener::TCP(ref mut tcp_listener) => {
                tcp_listener
                    .stop()
                    .await?
            }
            #[cfg(feature = "http3")]
            HttpListener::UDP(ref mut udp_listener) => {
                udp_listener
                    .stop()
                    .await?
            }
        }
        Ok(())
    }
}

pub struct HttpServer {
    config: ServerConfig,
    task: Option<GateTask>,
    listeners: Vec<HttpListener>,
    virtual_hosts: VetisVirtualHosts,
}

impl Server for HttpServer {
    fn new(config: ServerConfig) -> Self {
        Self {
            config,
            task: None,
            listeners: Vec::new(),
            virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())),
        }
    }

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts) {
        self.virtual_hosts = virtual_hosts;
    }

    async fn start(&mut self) -> Result<(), VetisError> {
        let mut listeners: Vec<HttpListener> = self
            .config
            .listeners()
            .iter()
            .map(|listener_config| match listener_config.protocol() {
                #[cfg(feature = "http1")]
                Protocol::HTTP1 => {
                    let mut listener = TcpServerListener::new(listener_config.clone());
                    listener.set_virtual_hosts(
                        self.virtual_hosts
                            .clone(),
                    );
                    HttpListener::TCP(listener)
                }
                #[cfg(feature = "http2")]
                Protocol::HTTP2 => {
                    let mut listener = TcpServerListener::new(listener_config.clone());
                    listener.set_virtual_hosts(
                        self.virtual_hosts
                            .clone(),
                    );
                    HttpListener::TCP(listener)
                }
                #[cfg(feature = "http3")]
                Protocol::HTTP3 => {
                    let mut listener = UdpServerListener::new(listener_config.clone());
                    listener.set_virtual_hosts(
                        self.virtual_hosts
                            .clone(),
                    );
                    HttpListener::UDP(listener)
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
