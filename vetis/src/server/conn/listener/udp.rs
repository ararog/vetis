use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use ::http::{Request, Response};
use bytes::Bytes;
use h3::server::{Connection, RequestResolver};
use h3_quinn::{
    quinn::{self, crypto::rustls::QuicServerConfig},
    Connection as QuinnConnection,
};
use http_body_util::{BodyExt, Full};

use log::{error, info};
use rt_gate::{spawn_server, spawn_worker, GateTask};

use crate::{
    config::ListenerConfig,
    errors::{StartError::Tls, VetisError},
    server::{
        conn::listener::{Listener, ListenerResult},
        tls::TlsFactory,
    },
    VetisRwLock, VetisVirtualHosts,
};

pub struct UdpListener<'a> {
    config: ListenerConfig<'a>,
    task: Option<GateTask>,
    virtual_hosts: VetisVirtualHosts<'a>,
}

impl<'a> Listener<'a> for UdpListener<'a> {
    fn new(config: ListenerConfig<'a>) -> Self {
        Self { config, task: None, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())) }
    }

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts<'a>) {
        self.virtual_hosts = virtual_hosts;
    }

    fn listen(&mut self) -> ListenerResult<'a, ()> {
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
        };
        Box::pin(future)
    }

    fn stop(&mut self) -> ListenerResult<'a, ()> {
        Box::pin(async move {
            if let Some(mut task) = self.task.take() {
                task.cancel().await;
            }
            Ok(())
        })
    }
}

impl<'a> UdpListener<'a> {
    async fn handle_connections(
        &mut self,
        endpoint: quinn::Endpoint,
        virtual_hosts: VetisVirtualHosts<'a>,
    ) -> Result<GateTask, VetisError> {
        let port = self.config.port();
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

                            loop {
                                match h3_conn
                                    .accept()
                                    .await
                                {
                                    Ok(Some(resolver)) => {
                                        handle_http_request(port, resolver, virtual_hosts.clone());
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

fn handle_http_request<'a>(
    port: u16,
    resolver: RequestResolver<QuinnConnection, Bytes>,
    virtual_hosts: VetisVirtualHosts<'a>,
) -> Result<(), VetisError> {
    let virtual_hosts = virtual_hosts.clone();
    spawn_worker(async move {
        let result = resolver
            .resolve_request()
            .await;
        if let Ok((req, mut stream)) = result {
            let (parts, _) = req.into_parts();

            /*
            let body = if parts.method == http::Method::POST
                || parts.method == http::Method::PUT
                || parts.method == http::Method::PATCH
            {
                let body = Full::new(Bytes::new());

                let mut data = Vec::new();
                while let Ok(Some(chunk)) = stream
                    .recv_data()
                    .await
                {
                    data.extend_from_slice(&[1, 2, 4]);
                }
                body
            } else {
                Full::new(Bytes::new())
            };
            */

            let body = Full::new(Bytes::new());

            let request = Request::from_parts(parts, body);

            let host = request
                .uri()
                .authority();

            let virtual_hosts = virtual_hosts.clone();
            let response = if let Some(host) = host {
                info!("Serving request for host: {}", host);
                let virtual_host = virtual_hosts
                    .read()
                    .await;

                let virtual_host = virtual_host.get(&(
                    host.host()
                        .to_string(),
                    port,
                ));

                let response = if let Some(virtual_host) = virtual_host {
                    let request = crate::Request::from_quic(request);

                    let vetis_response = virtual_host
                        .route(request)
                        .await;

                    let response = if let Err(err) = vetis_response {
                        error!("Error executing request: {:?}", err);
                        Response::builder()
                            .status(500)
                            .body(Full::new(Bytes::from_static(b"Internal server error")))
                            .unwrap()
                    } else {
                        let response = vetis_response
                            .unwrap()
                            .into_inner();

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

                                let header_value = header::HeaderValue::from_str(value.as_str());
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
                    };

                    Ok::<_, VetisError>(response)
                } else {
                    error!("Virtual host not found: {}", host.to_string());
                    let response = Response::builder()
                        .status(404)
                        .body(Full::new(Bytes::from_static(b"Virtual host not found")))
                        .unwrap();
                    Ok(response)
                };

                response
            } else {
                error!("Host not found in request");
                let response = Response::builder()
                    .status(400)
                    .body(Full::new(Bytes::from_static(b"Host not found in request")))
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

                let _ = stream
                    .finish()
                    .await;
            } else {
                error!("HttpServer - Error serving connection: {:?}", response.err());
            }
        }
    });

    Ok(())
}
