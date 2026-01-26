use std::{
    collections::HashMap,
    future::Future,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use bytes::Bytes;
use h3::server::{Connection, RequestResolver};
use h3_quinn::{
    quinn::{self, crypto::rustls::QuicServerConfig},
    Connection as QuinnConnection,
};
use http::{Request, Response};
use http_body_util::{BodyExt, Full};

use log::{error, info};
use rt_gate::{spawn_server, spawn_worker, GateTask};

use crate::{
    config::ServerConfig,
    errors::{StartError::Tls, VetisError},
    server::{
        conn::udp::UdpServer,
        tls::{self, TlsFactory},
        virtual_host::{self, VirtualHost},
        Server,
    },
    VetisRwLock, VetisVirtualHosts,
};

pub struct HttpServer {
    task: Option<GateTask>,
    config: ServerConfig,
    virtual_hosts: VetisVirtualHosts,
}

impl HttpServer {
    pub fn new(config: ServerConfig) -> Self {
        Self { task: None, config, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())) }
    }
}

impl Server<Full<Bytes>, Full<Bytes>> for HttpServer {
    fn port(&self) -> u16 {
        self.config.port()
    }

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts) {
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

        let tls_config = TlsFactory::create_tls_config(
            self.virtual_hosts
                .clone(),
            vec![b"h3".to_vec()],
        )
        .await?;

        if let Some(tls_config) = tls_config {
            let quic_config = QuicServerConfig::try_from(tls_config)
                .map_err(|e| VetisError::Start(Tls(e.to_string())))?;

            let server_config = quinn::ServerConfig::with_crypto(Arc::new(quic_config));

            let endpoint = quinn::Endpoint::server(server_config, addr)
                .map_err(|e| VetisError::Bind(e.to_string()))?;

            let server_task = self
                .handle_connections(
                    endpoint,
                    self.virtual_hosts
                        .clone(),
                )
                .await?;

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

impl UdpServer for HttpServer {
    async fn handle_connections(
        &mut self,
        endpoint: quinn::Endpoint,
        virtual_hosts: VetisVirtualHosts,
    ) -> Result<GateTask, VetisError> {
        let task = spawn_server(async move {
            while let Some(new_conn) = endpoint
                .accept()
                .await
            {
                let virtual_hosts = virtual_hosts.clone();
                spawn_worker(async move {
                    match new_conn.await {
                        Ok(conn) => {
                            let mut h3_conn: Connection<QuinnConnection, Bytes> =
                                Connection::new(QuinnConnection::new(conn))
                                    .await
                                    .unwrap();
                            let request_handler = ServerHandler {};
                            loop {
                                match h3_conn
                                    .accept()
                                    .await
                                {
                                    Ok(Some(resolver)) => {
                                        let _ =
                                            request_handler.handle(resolver, virtual_hosts.clone());
                                    }
                                    Ok(None) => {
                                        break;
                                    }
                                    Err(err) => {
                                        error!("Cannot accept connection: {:?}", err);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error!("Accepting connection failed: {:?}", err);
                        }
                    }
                });
            }

            endpoint
                .wait_idle()
                .await;
        });

        Ok(task)
    }
}

struct ServerHandler {}

impl ServerHandler {
    pub fn handle(
        &self,
        resolver: RequestResolver<QuinnConnection, Bytes>,
        virtual_hosts: VetisVirtualHosts,
    ) -> Result<(), VetisError> {
        let virtual_hosts = virtual_hosts.clone();
        spawn_worker(async move {
            let result = resolver
                .resolve_request()
                .await;
            if let Ok((req, mut stream)) = result {
                let (parts, _) = req.into_parts();

                let request = Request::from_parts(parts, Full::new(Bytes::new()));

                let host = request
                    .headers()
                    .get(::http::header::HOST);

                let virtual_hosts = virtual_hosts.clone();
                let response = if let Some(host) = host {
                    info!(
                        "Serving request for host: {}",
                        host.to_str()
                            .unwrap()
                    );
                    let virtual_host = virtual_hosts
                        .read()
                        .await;

                    let virtual_host = virtual_host.get(
                        host.to_str()
                            .unwrap(),
                    );

                    let response = if let Some(virtual_host) = virtual_host {
                        (virtual_host)
                            .execute(request)
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
                    };

                    response
                } else {
                    error!("Host header not found in request");
                    let response = Response::builder()
                        .status(400)
                        .body(Full::new(Bytes::from_static(b"Host header not found in request")))
                        .unwrap();
                    Ok(response)
                };

                if let Ok(response) = response {
                    let (parts, body) = response.into_parts();

                    let mut resp = Response::builder()
                        .status(parts.status)
                        .version(parts.version)
                        .extension(parts.extensions)
                        .body(())
                        .unwrap();

                    resp.headers_mut()
                        .extend(parts.headers);

                    match stream
                        .send_response(resp)
                        .await
                    {
                        Ok(_) => {
                            info!("Successfully respond to connection");
                        }
                        Err(err) => {
                            error!("Unable to send response to connection peer: {:?}", err);
                        }
                    }

                    let collected = body.collect().await;

                    let buf = Bytes::from(
                        collected
                            .expect("HttpServer - Failed to collect response")
                            .to_bytes()
                            .to_vec(),
                    );

                    let _ = stream
                        .send_data(buf)
                        .await;
                } else {
                    error!("HttpServer - Error serving connection: {:?}", response.err());
                }

                let _ = stream
                    .finish()
                    .await;
            }
        });

        Ok(())
    }
}
