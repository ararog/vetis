use std::sync::Arc;
use std::{collections::HashMap, future::Future};

use crate::{server::tls::TlsFactory, VetisRwLock};
use log::info;

use hyper::service::service_fn;

#[cfg(all(feature = "smol-rt", feature = "http2"))]
use crate::rt::smol::SmolExecutor;
#[cfg(all(feature = "tokio-rt", feature = "http2"))]
use hyper_util::rt::TokioExecutor;
#[cfg(feature = "smol-rt")]
use smol::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "tokio-rt")]
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    server::errors::VetisError,
    server::{virtual_host::VirtualHost, Server},
};
use bytes::Bytes;
use http::Response;
use http_body_util::Full;
use hyper::body::Incoming;
use log::error;
use rt_gate::{spawn_server, spawn_worker, GateTask};

#[cfg(feature = "http1")]
use hyper::server::conn::http1;
#[cfg(feature = "http2")]
use hyper::server::conn::http2;

#[cfg(all(feature = "tokio-rt", any(feature = "http1", feature = "http2")))]
use hyper_util::rt::TokioIo;
#[cfg(feature = "tokio-rt")]
use tokio::net::TcpListener;

#[cfg(feature = "smol-rt")]
use smol::net::TcpListener;
#[cfg(all(feature = "smol-rt", any(feature = "http1", feature = "http2")))]
use smol_hyper::rt::FuturesIo;

#[cfg(feature = "tokio-rt")]
type VetisTcpListener = TcpListener;
#[cfg(feature = "tokio-rt")]
type VetisIo<T> = TokioIo<T>;
#[cfg(all(feature = "tokio-rt", feature = "http2"))]
type VetisExecutor = TokioExecutor;

#[cfg(feature = "smol-rt")]
type VetisTcpListener = TcpListener;
#[cfg(feature = "smol-rt")]
type VetisIo<T> = FuturesIo<T>;
#[cfg(all(feature = "smol-rt", feature = "http2"))]
type VetisExecutor = SmolExecutor;

pub trait TcpServer: Server<Incoming, Full<Bytes>> {
    fn handle_connections(
        &mut self,
        listener: VetisTcpListener,
        virtual_host: Arc<
            VetisRwLock<HashMap<String, Box<dyn VirtualHost + Send + Sync + 'static>>>,
        >,
    ) -> Result<GateTask, VetisError> {
        //let alpn = if cfg!(feature = "http1") { "http/1.1".into() } else { "h2".into() };
        //let tls_acceptor = TlsFactory::create_tls_acceptor(virtual_host.clone(), alpn).await?;
        let virtual_hosts = virtual_host.clone();
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

                // TODO: Check if connection is secure first, then handle virtual host,
                //       path and request, note that virtual host and path are not
                //       implemented yet

                let virtual_hosts = virtual_hosts.clone();
                /*
                if let Some(acceptor) = &tls_acceptor {
                    let tls_stream = acceptor
                        .accept(stream)
                        .await;

                    if let Err(e) = tls_stream {
                        error!("Cannot accept connection: {:?}", e);
                        continue;
                    }

                    let tls_stream = tls_stream.unwrap();
                    let io = VetisIo::new(tls_stream);
                    let request_handler = ServerHandler {};
                    let _ = request_handler.handle(io, virtual_host);
                } else {
                    let io = VetisIo::new(stream);
                    let request_handler = ServerHandler {};
                    let _ = request_handler.handle(io, virtual_host);
                }
                */

                let io = VetisIo::new(stream);
                let request_handler = ServerHandler {};
                let _ = request_handler.handle(io, virtual_hosts);
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
        virtual_hosts: Arc<
            VetisRwLock<HashMap<String, Box<dyn VirtualHost + Send + Sync + 'static>>>,
        >,
    ) -> Result<(), VetisError>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let virtual_hosts = virtual_hosts.clone();

        let service_fn = service_fn(move |req| {
            let value = virtual_hosts.clone();
            async move {
                let host = req
                    .headers()
                    .get(http::header::HOST);
                if let Some(host) = host {
                    info!(
                        "Serving request for host: {}",
                        host.to_str()
                            .unwrap()
                    );
                    let virtual_host = value.read().await;

                    let virtual_host = virtual_host.get(
                        host.to_str()
                            .unwrap(),
                    );

                    if let Some(virtual_host) = virtual_host {
                        (virtual_host)
                            .execute(req)
                            .await
                    } else {
                        error!(
                            "Virtual host not found for host: {}",
                            host.to_str()
                                .unwrap()
                        );
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
