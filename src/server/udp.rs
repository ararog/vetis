use std::sync::Arc;

use log::{error, info};
use rt_gate::{spawn_server, spawn_worker, GateTask};

use crate::server::{errors::VetisError, Server};
use bytes::Bytes;
use h3::server::{Connection, RequestResolver};
use h3_quinn::Connection as QuinnConnection;
use http::{Request, Response};
use http_body_util::{BodyExt, Full};
use hyper::service::Service;

pub trait UdpServer: Server<Full<Bytes>, Full<Bytes>> {
    fn handle_connections<S>(
        &mut self,
        endpoint: quinn::Endpoint,
        handler: Arc<S>,
    ) -> Result<GateTask, VetisError>
    where
        S: Service<Request<Full<Bytes>>, Response = Response<Full<Bytes>>, Error = VetisError>
            + Send
            + Sync
            + 'static,
        S::Future: Send,
    {
        let task = spawn_server(async move {
            while let Some(new_conn) = endpoint
                .accept()
                .await
            {
                let handler = handler.clone();
                spawn_worker(async move {
                    match new_conn.await {
                        Ok(conn) => {
                            let mut h3_conn: Connection<QuinnConnection, Bytes> =
                                Connection::new(QuinnConnection::new(conn))
                                    .await
                                    .unwrap();
                            let request_handler = ServerHandler::new(handler);
                            loop {
                                match h3_conn
                                    .accept()
                                    .await
                                {
                                    Ok(Some(resolver)) => {
                                        let _ = request_handler.handle(resolver);
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

struct ServerHandler<S> {
    handler: Arc<S>,
}

impl<S> ServerHandler<S>
where
    S: Service<Request<Full<Bytes>>, Response = Response<Full<Bytes>>, Error = VetisError>
        + Send
        + Sync
        + 'static,
    S::Future: Send,
{
    pub fn new(handler: Arc<S>) -> Self {
        Self { handler }
    }

    pub fn handle(
        &self,
        resolver: RequestResolver<QuinnConnection, Bytes>,
    ) -> Result<(), VetisError> {
        let handler = self.handler.clone();
        spawn_worker(async move {
            let result = resolver
                .resolve_request()
                .await;
            if let Ok((req, mut stream)) = result {
                let (parts, _) = req.into_parts();

                let request = http::Request::from_parts(parts, Full::new(Bytes::new()));

                let response = handler
                    .call(request)
                    .await
                    .map_err(|e| VetisError::Handler(e.to_string()));

                if let Ok(response) = response {
                    let (parts, body) = response.into_parts();

                    let mut resp = http::Response::builder()
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
