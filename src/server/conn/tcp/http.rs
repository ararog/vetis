use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::service_fn,
};

use log::{error, info};
use rt_gate::{spawn_server, spawn_worker, GateTask};

use ::http::Response;

#[cfg(feature = "smol-rt")]
use peekable::futures::AsyncPeekable;

#[cfg(feature = "tokio-rt")]
use peekable::tokio::AsyncPeekable;

#[cfg(feature = "http1")]
use hyper::server::conn::http1;
#[cfg(feature = "http2")]
use hyper::server::conn::http2;

#[cfg(all(feature = "smol-rt", feature = "http2"))]
use crate::rt::smol::SmolExecutor;
#[cfg(all(feature = "tokio-rt", feature = "http2"))]
use hyper_util::rt::TokioExecutor;

#[cfg(feature = "smol-rt")]
use smol::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "tokio-rt")]
use tokio::io::{AsyncRead, AsyncWrite};

#[cfg(all(feature = "tokio-rt", any(feature = "http1", feature = "http2")))]
use hyper_util::rt::TokioIo;
#[cfg(feature = "tokio-rt")]
use tokio::net::TcpListener;
#[cfg(feature = "tokio-rt")]
use tokio_rustls::TlsAcceptor;

#[cfg(feature = "smol-rt")]
use futures_rustls::TlsAcceptor;
#[cfg(feature = "smol-rt")]
use smol::net::TcpListener;
#[cfg(all(feature = "smol-rt", any(feature = "http1", feature = "http2")))]
use smol_hyper::rt::FuturesIo;

use crate::{
    config::ServerConfig,
    errors::{StartError, VetisError},
    server::{conn::tcp::TcpServer, tls::TlsFactory, virtual_host::VirtualHost, Server},
    VetisRwLock, VetisVirtualHosts,
};

#[cfg(feature = "tokio-rt")]
type VetisTcpListener = TcpListener;
#[cfg(feature = "tokio-rt")]
type VetisTlsAcceptor = TlsAcceptor;
#[cfg(feature = "tokio-rt")]
type VetisIo<T> = TokioIo<T>;
#[cfg(all(feature = "tokio-rt", feature = "http2"))]
type VetisExecutor = TokioExecutor;

#[cfg(feature = "smol-rt")]
type VetisTcpListener = TcpListener;
#[cfg(feature = "smol-rt")]
type VetisTlsAcceptor = TlsAcceptor;
#[cfg(feature = "smol-rt")]
type VetisIo<T> = FuturesIo<T>;
#[cfg(all(feature = "smol-rt", feature = "http2"))]
type VetisExecutor = SmolExecutor;

pub struct HttpServer {
    config: ServerConfig,
    task: Option<GateTask>,
    virtual_hosts: VetisVirtualHosts,
}

impl HttpServer {
    pub fn new(config: ServerConfig) -> Self {
        Self { config, task: None, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())) }
    }
}

impl Server<Incoming, Full<Bytes>> for HttpServer {
    fn port(&self) -> u16 {
        self.config.port()
    }

    fn set_virtual_hosts(
        &mut self,
        virtual_hosts: Arc<VetisRwLock<HashMap<String, Box<dyn VirtualHost>>>>,
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

        let task = self
            .handle_connections(
                listener,
                self.virtual_hosts
                    .clone(),
            )
            .await?;

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

impl TcpServer for HttpServer {
    async fn handle_connections(
        &mut self,
        listener: VetisTcpListener,
        virtual_host: VetisVirtualHosts,
    ) -> Result<GateTask, VetisError> {
        let alpn = vec![
            #[cfg(feature = "http1")]
            b"http/1.1".to_vec(),
            #[cfg(feature = "http2")]
            b"h2".to_vec(),
            #[cfg(feature = "http3")]
            b"h3".to_vec(),
        ];
        let tls_config = TlsFactory::create_tls_config(virtual_host.clone(), alpn).await?;
        if tls_config.is_none() {
            return Err(VetisError::Start(StartError::Tls(
                "Failed to create TLS acceptor".to_string(),
            )));
        }

        let tls_config = tls_config.unwrap();
        let tls_acceptor = VetisTlsAcceptor::from(Arc::new(tls_config));
        let task = spawn_server(async move {
            loop {
                let result = listener
                    .accept()
                    .await;

                if let Err(err) = result {
                    error!("Cannot accept connection: {:?}", err);
                    continue;
                }

                let (stream, _) = result.unwrap();
                if let Err(e) = stream.set_nodelay(true) {
                    error!("Cannot set TCP_NODELAY: {}", e);
                    continue;
                }

                let mut peekable = AsyncPeekable::from(stream);

                let mut peeked = [0; 16];
                peekable
                    .peek_exact(&mut peeked)
                    .await
                    .unwrap();

                let is_tls = peeked.starts_with(&[0x16, 0x03]);

                if is_tls {
                    let tls_stream = tls_acceptor
                        .accept(peekable)
                        .await;

                    if let Err(e) = tls_stream {
                        error!("Cannot accept connection: {:?}", e);
                        continue;
                    }

                    let tls_stream = tls_stream.unwrap();
                    let io = VetisIo::new(tls_stream);
                    let request_handler = ServerHandler {};
                    let _ = request_handler.handle(io, virtual_host.clone());
                } else {
                    let io = VetisIo::new(peekable);
                    let request_handler = ServerHandler {};
                    let _ = request_handler.handle(io, virtual_host.clone());
                }
            }
        });

        Ok(task)
    }
}

struct ServerHandler {}

impl ServerHandler {
    pub fn handle<T>(
        &self,
        io: VetisIo<T>,
        virtual_hosts: VetisVirtualHosts,
    ) -> Result<(), VetisError>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let virtual_hosts = virtual_hosts.clone();

        let service_fn = service_fn(move |req| {
            let value = virtual_hosts.clone();
            async move {
                let host = req
                    .uri()
                    .authority();
                if let Some(host) = host {
                    info!("Serving request for host: {}", host);
                    let virtual_hosts = value.read().await;

                    let virtual_host = virtual_hosts.get(&host.to_string());

                    if let Some(virtual_host) = virtual_host {
                        (virtual_host)
                            .execute(req)
                            .await
                    } else {
                        error!("Virtual host not found for host: {}", host);
                        let response = Response::builder()
                            .status(404)
                            .body(Full::new(Bytes::from_static(b"Virtual host not found")))
                            .unwrap();
                        Ok(response)
                    }
                } else {
                    error!("Host header not found in request");
                    let response = Response::builder()
                        .status(400)
                        .body(Full::new(Bytes::from_static(b"Host header not found in request")))
                        .unwrap();
                    Ok(response)
                }
            }
        });

        // TODO: Inspect request by checking HOST header to find virtual host, then path
        spawn_worker(async move {
            #[cfg(feature = "http1")]
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn)
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
            #[cfg(feature = "http2")]
            if let Err(err) = http2::Builder::new(VetisExecutor::new())
                .serve_connection(io, service_fn)
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });

        Ok(())
    }
}
