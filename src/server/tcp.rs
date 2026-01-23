use std::sync::Arc;

use crate::RequestType;
use crate::ResponseType;

#[cfg(all(feature = "smol-rt", feature = "http2"))]
use crate::rt::smol::SmolExecutor;
#[cfg(all(feature = "tokio-rt", feature = "http2"))]
use hyper_util::rt::TokioExecutor;
#[cfg(feature = "smol-rt")]
use smol::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "tokio-rt")]
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{server::errors::VetisError, server::Server};
use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;
use hyper::body::Incoming;
use log::error;
use rt_gate::{spawn_server, spawn_worker, GateTask};

use hyper::service::Service;

#[cfg(feature = "http1")]
use hyper::server::conn::http1;
#[cfg(feature = "http2")]
use hyper::server::conn::http2;

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

#[cfg(feature = "tokio-rt")]
type EasyTcpListener = TcpListener;
#[cfg(feature = "tokio-rt")]
type EasyIo<T> = TokioIo<T>;
#[cfg(feature = "tokio-rt")]
type EasyTlsAcceptor = TlsAcceptor;
#[cfg(all(feature = "tokio-rt", feature = "http2"))]
type EasyExecutor = TokioExecutor;

#[cfg(feature = "smol-rt")]
type EasyTcpListener = TcpListener;
#[cfg(feature = "smol-rt")]
type EasyIo<T> = FuturesIo<T>;
#[cfg(feature = "smol-rt")]
type EasyTlsAcceptor = TlsAcceptor;
#[cfg(all(feature = "smol-rt", feature = "http2"))]
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

                if let Some(acceptor) = &tls_acceptor {
                    let tls_stream = acceptor
                        .accept(stream)
                        .await;

                    if let Err(e) = tls_stream {
                        error!("Cannot accept connection: {:?}", e);
                        continue;
                    }

                    let io = EasyIo::new(tls_stream.unwrap());
                    let handler = handler.clone();
                    let request_handler = ServerHandler::new(handler);
                    let _ = request_handler.handle(io);
                } else {
                    let io = EasyIo::new(stream);
                    let handler = handler.clone();
                    let request_handler = ServerHandler::new(handler);
                    let _ = request_handler.handle(io);
                }
            }
        });

        Ok(task)
    }
}

struct ServerHandler<S> {
    handler: Arc<S>,
}

impl<S> ServerHandler<S>
where
    S: Service<RequestType, Response = ResponseType, Error = VetisError> + Send + Sync + 'static,
    S::Future: Send,
{
    pub fn new(handler: Arc<S>) -> Self {
        Self { handler }
    }

    pub fn handle<T>(&self, io: EasyIo<T>) -> Result<(), VetisError>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let handler = self.handler.clone();

        // TODO: Inspect request by checking HOST header to find virtual host, then path
        spawn_worker(async move {
            #[cfg(feature = "http1")]
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, handler.clone())
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
            #[cfg(feature = "http2")]
            if let Err(err) = http2::Builder::new(EasyExecutor::new())
                .serve_connection(io, handler.clone())
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });

        Ok(())
    }
}
