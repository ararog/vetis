use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response};
use std::future::Future;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

#[cfg(feature = "tokio-rt")]
use tokio::net::TcpListener;
#[cfg(feature = "tokio-rt")]
use tokio_rustls::TlsAcceptor;

#[cfg(feature = "smol-rt")]
use futures_rustls::TlsAcceptor;
#[cfg(feature = "smol-rt")]
use smol::net::TcpListener;

#[cfg(feature = "tokio-rt")]
type VetisTcpListener = TcpListener;
#[cfg(feature = "tokio-rt")]
type VetisTlsAcceptor = TlsAcceptor;

#[cfg(feature = "smol-rt")]
type VetisTcpListener = TcpListener;
#[cfg(feature = "smol-rt")]
type VetisTlsAcceptor = TlsAcceptor;

use rt_gate::GateTask;

use crate::server::errors::VetisError;
use crate::server::tcp::TcpServer;
use crate::server::{config::ServerConfig, Server};

pub struct HttpServer {
    config: ServerConfig,
    task: Option<GateTask>,
}

impl HttpServer {
    pub fn new(config: ServerConfig) -> Self {
        Self { task: None, config }
    }
}

impl TcpServer for HttpServer {}

impl Server<Incoming, Full<Bytes>> for HttpServer {
    fn port(&self) -> u16 {
        self.config.port()
    }

    async fn start<F, Fut>(&mut self, handler: F) -> Result<(), VetisError>
    where
        F: Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<Full<Bytes>>, VetisError>> + Send + 'static,
    {
        // TODO: Move this into block after check if connection is secure, use SNI for this
        let tls_acceptor = if let Some(config) = self
            .config
            .security()
        {
            let alpn = if cfg!(feature = "http1") { "http/1.1".into() } else { "h2".into() };

            let tls_config = self.setup_tls(config, alpn)?;

            Some(VetisTlsAcceptor::from(Arc::new(tls_config)))
        } else {
            None
        };

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

        let handler = Arc::new(service_fn(handler));

        let task = self.handle_connections(listener, tls_acceptor, handler)?;

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
