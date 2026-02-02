use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use http::header;
use http_body_util::{Either, Full};
use hyper::{
    body::{Bytes, Incoming},
    service::service_fn,
};

use log::{error, info};

use rt_gate::{spawn_server, spawn_worker, GateTask};

#[cfg(feature = "smol-rt")]
use peekable::future::AsyncPeekable;

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
use tokio_rustls::TlsAcceptor;

#[cfg(feature = "smol-rt")]
use futures_rustls::TlsAcceptor;
#[cfg(all(feature = "smol-rt", any(feature = "http1", feature = "http2")))]
use smol_hyper::rt::FuturesIo;

use crate::{
    config::{ListenerConfig, Protocol},
    errors::VetisError,
    server::{
        conn::listener::{Listener, ListenerResult},
        tls::TlsFactory,
    },
    VetisRwLock, VetisVirtualHosts,
};

#[cfg(feature = "tokio-rt")]
type VetisTcpListener = tokio::net::TcpListener;
#[cfg(feature = "tokio-rt")]
type VetisTlsAcceptor = TlsAcceptor;
#[cfg(feature = "tokio-rt")]
type VetisIo<T> = TokioIo<T>;
#[cfg(all(feature = "tokio-rt", feature = "http2"))]
type VetisExecutor = TokioExecutor;

#[cfg(feature = "smol-rt")]
type VetisTcpListener = smol::net::TcpListener;
#[cfg(feature = "smol-rt")]
type VetisTlsAcceptor = TlsAcceptor;
#[cfg(feature = "smol-rt")]
type VetisIo<T> = FuturesIo<T>;
#[cfg(all(feature = "smol-rt", feature = "http2"))]
type VetisExecutor = SmolExecutor;

pub struct TcpListener {
    task: Option<GateTask>,
    config: ListenerConfig,
    virtual_hosts: VetisVirtualHosts,
}

impl Listener for TcpListener {
    fn new(config: ListenerConfig) -> Self {
        Self { task: None, config, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())) }
    }

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts) {
        self.virtual_hosts = virtual_hosts;
    }

    fn listen(&mut self) -> ListenerResult<()> {
        let future = async move {
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
                    self.config
                        .protocol()
                        .clone(),
                    listener,
                    self.virtual_hosts
                        .clone(),
                )
                .await?;

            self.task = Some(task);

            Ok(())
        };

        Box::pin(future)
    }

    fn stop(&mut self) -> ListenerResult<()> {
        let future = async move {
            if let Some(mut task) = self.task.take() {
                task.cancel().await;
            }
            Ok(())
        };

        Box::pin(future)
    }
}

impl TcpListener {
    async fn handle_connections(
        &mut self,
        protocol: Protocol,
        listener: VetisTcpListener,
        virtual_hosts: VetisVirtualHosts,
    ) -> Result<GateTask, VetisError> {
        let alpn = vec![
            #[cfg(feature = "http1")]
            b"http/1.1".to_vec(),
            #[cfg(feature = "http2")]
            b"h2".to_vec(),
            #[cfg(feature = "http3")]
            b"h3".to_vec(),
        ];
        let tls_config = TlsFactory::create_tls_config(virtual_hosts.clone(), alpn).await?;
        let port = Arc::new(self.config.port());
        let tls_config = tls_config.unwrap();
        let tls_acceptor = VetisTlsAcceptor::from(Arc::new(tls_config));
        let future = async move {
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
                    match protocol {
                        #[cfg(feature = "http1")]
                        Protocol::Http1 => {
                            let _ = handle_http1_request(port.clone(), io, virtual_hosts.clone());
                        }
                        #[cfg(feature = "http2")]
                        Protocol::Http2 => {
                            let _ = handle_http2_request(port.clone(), io, virtual_hosts.clone());
                        }
                        #[cfg(feature = "http3")]
                        Protocol::Http3 => {
                            // HTTP/3 is handled by UDP listener
                        }
                    }
                } else {
                    let io = VetisIo::new(peekable);
                    match protocol {
                        #[cfg(feature = "http1")]
                        Protocol::Http1 => {
                            let _ = handle_http1_request(port.clone(), io, virtual_hosts.clone());
                        }
                        #[cfg(feature = "http2")]
                        Protocol::Http2 => {
                            let _ = handle_http2_request(port.clone(), io, virtual_hosts.clone());
                        }
                        #[cfg(feature = "http3")]
                        Protocol::Http3 => {
                            // HTTP/3 is handled by UDP listener
                        }
                    }
                }
            }
        };

        let task = spawn_server(future);

        Ok(task)
    }
}

async fn process_request(
    req: http::Request<Incoming>,
    virtual_hosts: VetisVirtualHosts,
    port: Arc<u16>,
) -> Result<http::Response<Either<Incoming, Full<Bytes>>>, VetisError> {
    let host = req
        .headers()
        .get(header::HOST);

    let host = if let Some(host) = host {
        let host_port = host.to_str();
        match host_port {
            Ok(host_port) => Some(
                host_port
                    .split_once(':')
                    .map(|(host, _)| host)
                    .unwrap_or(host_port),
            ),
            Err(_) => Some("localhost"),
        }
    } else {
        match req
            .uri()
            .authority()
        {
            Some(auth) => Some(auth.host()),
            None => Some("localhost"),
        }
    };

    if let Some(host) = host {
        info!("Serving request for host: {}", host);
        let virtual_hosts = virtual_hosts
            .read()
            .await;

        let virtual_host = virtual_hosts.get(&(host.into(), *port.clone()));

        if let Some(virtual_host) = virtual_host {
            let request = crate::Request::from_http(req);

            let vetis_response = virtual_host
                .route(request)
                .await?;

            let mut response = vetis_response.into_inner();

            let default_headers = virtual_host
                .config()
                .default_headers();
            if let Some(default_headers) = default_headers {
                for (key, value) in default_headers {
                    let header_name = header::HeaderName::from_bytes(key.as_bytes());
                    if header_name.is_err() {
                        error!("Invalid header name: {}", key);
                        continue;
                    }
                    let header_name = header_name.unwrap();

                    let header_value = header::HeaderValue::from_str(value);
                    if header_value.is_err() {
                        error!("Invalid header value: {}", value);
                        continue;
                    }
                    let header_value = header_value.unwrap();

                    response
                        .headers_mut()
                        .insert(header_name, header_value);
                }
            }

            Ok::<http::Response<Either<Incoming, Full<Bytes>>>, VetisError>(response)
        } else {
            error!("Virtual host not found: {}", host);
            let response = http::Response::builder()
                .status(404)
                .body(Either::Right(Full::new(Bytes::from_static(b"Virtual host not found"))))
                .unwrap();
            Ok(response)
        }
    } else {
        error!("Host not found in request");
        let response = http::Response::builder()
            .status(400)
            .body(Either::Right(Full::new(Bytes::from_static(b"Host not found in request"))))
            .unwrap();
        Ok(response)
    }
}

#[cfg(feature = "http1")]
fn handle_http1_request<T>(
    port: Arc<u16>,
    io: VetisIo<T>,
    virtual_hosts: VetisVirtualHosts,
) -> Result<(), VetisError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let service_fn = service_fn(move |req| {
        let value = virtual_hosts.clone();
        let port = port.clone();
        async move { process_request(req, value, port).await }
    });

    let future = async move {
        if let Err(err) = http1::Builder::new()
            .serve_connection(io, service_fn)
            .await
        {
            error!("Error serving connection: {:?}", err);
        }
    };

    spawn_worker(future);

    Ok(())
}

#[cfg(feature = "http2")]
pub fn handle_http2_request<T>(
    port: Arc<u16>,
    io: VetisIo<T>,
    virtual_hosts: VetisVirtualHosts,
) -> Result<(), VetisError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let service_fn = service_fn(move |req| {
        let value = virtual_hosts.clone();
        async move { process_request(req, value, port.clone()).await }
    });

    let future = async move {
        if let Err(err) = http2::Builder::new(VetisExecutor::new())
            .serve_connection(io, service_fn)
            .await
        {
            error!("Error serving connection: {:?}", err);
        }
    };

    spawn_worker(future);

    Ok(())
}
