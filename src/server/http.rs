use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use std::collections::HashMap;
use std::future::Future;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

#[cfg(feature = "tokio-rt")]
use tokio::net::TcpListener;

#[cfg(feature = "smol-rt")]
use smol::net::TcpListener;

#[cfg(feature = "tokio-rt")]
type VetisTcpListener = TcpListener;

#[cfg(feature = "smol-rt")]
type VetisTcpListener = TcpListener;

use rt_gate::GateTask;

use crate::server::virtual_host::VirtualHost;
use crate::server::{config::ServerConfig, errors::VetisError, tcp::TcpServer, Server};
use crate::VetisRwLock;

pub struct HttpServer {
    config: ServerConfig,
    task: Option<GateTask>,
    virtual_hosts: Arc<VetisRwLock<HashMap<String, Box<dyn VirtualHost + Send + Sync + 'static>>>>,
}

impl HttpServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            task: None,
            virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new().into())),
        }
    }
}

impl TcpServer for HttpServer {}

impl Server<Incoming, Full<Bytes>> for HttpServer {
    fn port(&self) -> u16 {
        self.config.port()
    }

    fn set_virtual_hosts(
        &mut self,
        virtual_hosts: Arc<
            VetisRwLock<HashMap<String, Box<dyn VirtualHost + Send + Sync + 'static>>>,
        >,
    ) {
        self.virtual_hosts = virtual_hosts;
    }

    async fn start(&mut self) -> Result<(), VetisError> {
        let addr = if let Ok(ip) = self
            .config
            .interface()
            .parse::<Ipv4Addr>()
        {
            SocketAddr::from((ip, self.config.port()))
        } else {
            let addr = self
                .config
                .interface()
                .parse::<Ipv6Addr>();
            if let Ok(addr) = addr {
                SocketAddr::from((addr, self.config.port()))
            } else {
                SocketAddr::from(([0, 0, 0, 0], self.config.port()))
            }
        };

        let listener = VetisTcpListener::bind(addr)
            .await
            .map_err(|e| VetisError::Bind(e.to_string()))?;

        let task = self.handle_connections(
            listener,
            self.virtual_hosts
                .clone(),
        )?;

        self.task = Some(task);

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), VetisError> {
        if let Some(mut task) = self.task.take() {
            task.cancel().await;
        }
        Ok(())
    }
}
