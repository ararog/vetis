use std::future::Future;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::{net::SocketAddr, sync::Arc};

use bytes::Bytes;
use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};
use http::{Request, Response};
use http_body_util::Full;
use hyper::service::service_fn;

use rt_gate::GateTask;

use crate::server::config::ServerConfig;
use crate::server::errors::{StartError::Tls, VetisError};
use crate::server::udp::UdpServer;
use crate::server::Server;

pub struct HttpServer {
    port: u16,
    task: Option<GateTask>,
    config: ServerConfig,
}

impl HttpServer {
    pub fn new(config: ServerConfig) -> Self {
        Self { port: config.port(), task: None, config }
    }
}

impl UdpServer for HttpServer {}

impl Server<Full<Bytes>, Full<Bytes>> for HttpServer {
    fn port(&self) -> u16 {
        self.port
    }

    async fn start<H, Fut>(&mut self, handler: H) -> Result<(), VetisError>
    where
        H: Fn(Request<Full<Bytes>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<Full<Bytes>>, VetisError>> + Send + 'static,
    {
        if let Some(config) = self
            .config
            .security()
        {
            if config
                .cert()
                .is_none()
                || config
                    .key()
                    .is_none()
            {
                return Err(VetisError::Start(Tls(
                    "Server certificate and key are required".to_string()
                )));
            }

            let tls_config = self.setup_tls(&config, b"h3".to_vec())?;

            let quic_config = QuicServerConfig::try_from(tls_config)
                .map_err(|e| VetisError::Start(Tls(e.to_string())))?;

            let server_config = quinn::ServerConfig::with_crypto(Arc::new(quic_config));

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

            let endpoint = quinn::Endpoint::server(server_config, addr)
                .map_err(|e| VetisError::Bind(e.to_string()))?;

            let handler = Arc::new(service_fn(handler));

            let server_task = self.handle_connections(endpoint, handler)?;

            self.task = Some(server_task);
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), VetisError> {
        if let Some(mut task) = self.task.take() {
            task.cancel().await;
        }
        Ok(())
    }
}
