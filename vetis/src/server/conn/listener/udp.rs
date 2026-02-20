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
use http::header;
use http_body_util::{BodyExt, Full};

use log::{debug, error, info};
use rt_gate::{spawn_server, spawn_worker, GateTask};

use crate::{
    config::server::ListenerConfig,
    errors::{StartError::Tls, VetisError},
    server::{
        conn::listener::{Listener, ListenerResult},
        http::static_response,
        tls::TlsFactory,
    },
    VetisRwLock, VetisVirtualHosts,
};

/// UDP listener
pub struct UdpListener {
    config: ListenerConfig,
    task: Option<GateTask>,
    virtual_hosts: VetisVirtualHosts,
}

impl Listener for UdpListener {
    /// Create a new listener
    ///
    /// # Arguments
    ///
    /// * `config` - A `ListenerConfig` instance containing the listener configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - A new `UdpListener` instance.
    fn new(config: ListenerConfig) -> Self {
        Self { config, task: None, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())) }
    }

    /// Allow set virtual hosts
    ///
    /// # Arguments
    ///
    /// * `virtual_hosts` - A `VetisVirtualHosts` instance containing the virtual hosts.
    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts) {
        self.virtual_hosts = virtual_hosts;
    }

    /// Listen for incoming connections
    ///
    /// # Returns
    ///
    /// * `ListenerResult<'_, ()>` - A `ListenerResult` instance containing the result of the listener.
    fn listen(&mut self) -> ListenerResult<'_, ()> {
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

    /// Stop the listener
    ///
    /// # Returns
    ///
    /// * `ListenerResult<'_, ()>` - A `ListenerResult` instance containing the result of the listener.
    fn stop(&mut self) -> ListenerResult<'_, ()> {
        Box::pin(async move {
            if let Some(mut task) = self.task.take() {
                task.cancel().await;
            }
            Ok(())
        })
    }
}

impl UdpListener {
    async fn handle_connections(
        &mut self,
        endpoint: quinn::Endpoint,
        virtual_hosts: VetisVirtualHosts,
    ) -> Result<GateTask, VetisError> {
        let port = self.config.port();
        let task = spawn_server(async move {
            while let Some(new_conn) = endpoint
                .accept()
                .await
            {
                let virtual_hosts = virtual_hosts.clone();
                let addr = new_conn.remote_address();
                spawn_worker(async move {
                    match new_conn.await {
                        Ok(conn) => {
                            let mut h3_conn: Connection<QuinnConnection, Bytes> = match
                                Connection::new(QuinnConnection::new(conn)).await {
                                    Ok(conn) => conn,
                                    Err(err) => {
                                        error!("Cannot create connection: {:?}", err);
                                        return;
                                    }
                                };

                            loop {
                                match h3_conn
                                    .accept()
                                    .await
                                {
                                    Ok(Some(resolver)) => {
                                        let result = handle_http_request(
                                            port,
                                            resolver,
                                            virtual_hosts.clone(),
                                            addr,
                                        );

                                        if let Err(err) = result {
                                            error!("Error handling HTTP request: {:?}", err);
                                        }
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

fn handle_http_request(
    port: u16,
    resolver: RequestResolver<QuinnConnection, Bytes>,
    virtual_hosts: VetisVirtualHosts,
    client_addr: SocketAddr,
) -> Result<(), VetisError> {
    let virtual_hosts = virtual_hosts.clone();
    spawn_worker(async move {
        let result = resolver
            .resolve_request()
            .await;
        if let Ok((req, mut stream)) = result {
            let (parts, _) = req.into_parts();

            let method = parts.method.clone();

            let uri = parts.uri.clone();

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
                debug!("Serving request for host: {}", host);
                let virtual_host = virtual_hosts
                    .read()
                    .await;

                let virtual_host = virtual_host.get(&(host.host().into(), port));

                let response = if let Some(virtual_host) = virtual_host {
                    let request = crate::Request::from_quic(request);

                    let vetis_response = virtual_host
                        .route(request)
                        .await;

                    let response = if let Err(err) = vetis_response {
                        error!("Error executing request: {:?}", err);
                        static_response(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                            None,
                            "Internal server error".to_string(),
                        )
                    } else {
                        let mut response = vetis_response
                            .unwrap()
                            .into_inner();

                        let default_headers = virtual_host
                            .config()
                            .default_headers();

                        if let Some(default_headers) = default_headers {
                            for (key, value) in default_headers {
                                let header_name =
                                    http::header::HeaderName::from_bytes(key.as_bytes());
                                if header_name.is_err() {
                                    error!("Invalid header name: {}", key);
                                    continue;
                                }
                                let header_name = header_name.unwrap();

                                let header_value =
                                    http::header::HeaderValue::from_str(value.as_str());
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

                        response
                    };

                    // TODO: Log request and its response status code (move it to oneshot channel?)
                    info!("{} {} {} {}", client_addr, method, uri, response.status());

                    Ok::<_, VetisError>(response)
                } else {
                    error!("Virtual host not found: {}", host);
                    let response = static_response(
                        http::StatusCode::NOT_FOUND,
                        None,
                        "Virtual host not found".to_string(),
                    );
                    Ok(response)
                };

                response
            } else {
                error!("Host not found in request");
                let response = static_response(
                    http::StatusCode::BAD_REQUEST,
                    None,
                    "Host not found in request".to_string(),
                );
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
                        debug!("Successfully respond to connection");
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
