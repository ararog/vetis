use std::sync::Arc;

#[cfg(feature = "smol-rt")]
use crate::rt::smol::SmolExecutor;
#[cfg(feature = "tokio-rt")]
use hyper_util::rt::{TokioExecutor, TokioIo};

use crate::{server::errors::VetisError, server::Server};
use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;
use hyper::body::Incoming;
use rt_gate::{spawn_server, spawn_worker, GateTask};

use hyper::service::Service;

#[cfg(feature = "http1")]
use hyper::server::conn::http1;
#[cfg(feature = "http2")]
use hyper::server::conn::http2;

#[cfg(feature = "tokio-rt")]
use ::tokio::net::TcpListener;
#[cfg(feature = "tokio-rt")]
use ::tokio::net::TcpStream;
#[cfg(feature = "smol-rt")]
use smol_hyper::rt::FuturesIo;
#[cfg(feature = "tokio-rt")]
use tokio_rustls::{server::TlsStream, TlsAcceptor};

#[cfg(feature = "smol-rt")]
use ::smol::net::TcpListener;
#[cfg(feature = "smol-rt")]
use ::smol::net::TcpStream;
#[cfg(feature = "smol-rt")]
use futures_rustls::{server::TlsStream, TlsAcceptor};

#[cfg(feature = "tokio-rt")]
type EasyTcpListener = TcpListener;
#[cfg(feature = "tokio-rt")]
type EasyIo<T> = TokioIo<T>;
#[cfg(feature = "tokio-rt")]
type EasyTlsAcceptor = TlsAcceptor;
#[cfg(feature = "tokio-rt")]
type EasyExecutor = TokioExecutor;

#[cfg(feature = "smol-rt")]
type EasyTcpListener = TcpListener;
#[cfg(feature = "smol-rt")]
type EasyIo<T> = FuturesIo<T>;
#[cfg(feature = "smol-rt")]
type EasyTlsAcceptor = TlsAcceptor;
#[cfg(feature = "smol-rt")]
type EasyExecutor = SmolExecutor;

pub trait TcpServer: Server<Incoming, Full<Bytes>> {
    fn handle_connections<S>(
        &mut self,
        listener: EasyTcpListener,
        tls_acceptor: Option<EasyTlsAcceptor>,
        handler: Arc<S>,
    ) -> Result<GateTask, VetisError>
    where
        S: Service<Request<Incoming>, Response = Response<Full<Bytes>>, Error = VetisError>
            + Send
            + Sync
            + 'static,
        S::Future: Send,
    {
        let task = spawn_server(async move {
            loop {
                let (stream, _) = listener
                    .accept()
                    .await
                    .expect("HttpServer - Failed to accept connection");

                if let Some(acceptor) = &tls_acceptor {
                    let tls_stream: TlsStream<TcpStream> = acceptor
                        .accept(stream)
                        .await
                        .expect("HttpServer - Failed to accept TLS connection");

                    let io = EasyIo::new(tls_stream);
                    let handler = handler.clone();
                    spawn_worker(async move {
                        #[cfg(feature = "http1")]
                        {
                            let result = http1::Builder::new()
                                .serve_connection(io, handler.clone())
                                .await;
                            if let Err(err) = result {
                                eprintln!("HttpServer - Error serving connection: {}", err);
                            }
                        }
                        #[cfg(feature = "http2")]
                        {
                            let result = http2::Builder::new(EasyExecutor::new())
                                .serve_connection(io, handler.clone())
                                .await;
                            if let Err(err) = result {
                                eprintln!("HttpServer - Error serving connection: {}", err);
                            }
                        }
                    });
                } else {
                    let io = EasyIo::new(stream);
                    let handler = handler.clone();
                    spawn_worker(async move {
                        #[cfg(feature = "http1")]
                        if let Err(err) = http1::Builder::new()
                            .serve_connection(io, handler.clone())
                            .await
                        {
                            eprintln!("HttpServer - Error serving connection: {}", err);
                        }
                        #[cfg(feature = "http2")]
                        if let Err(err) = http2::Builder::new(EasyExecutor::new())
                            .serve_connection(io, handler.clone())
                            .await
                        {
                            eprintln!("HttpServer - Error serving connection: {}", err);
                        }
                    });
                }
            }
        });

        Ok(task)
    }
}
