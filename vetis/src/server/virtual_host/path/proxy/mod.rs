use crate::{
    config::server::virtual_host::path::proxy::ProxyPathConfig,
    errors::{VetisError, VirtualHostError},
    server::{
        http::{Request, Response},
        virtual_host::path::{HostPath, Path},
    },
};
use deboa::{client::conn::pool::HttpConnectionPool, request::DeboaRequest, Client};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
};

static CLIENT: OnceLock<Client> = OnceLock::new();

/// Proxy path
pub struct ProxyPath {
    config: ProxyPathConfig,
}

impl ProxyPath {
    /// Create a new proxy path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The proxy path configuration
    ///
    /// # Returns
    ///
    /// * `ProxyPath` - The proxy path
    pub fn new(config: ProxyPathConfig) -> ProxyPath {
        ProxyPath { config }
    }
}

impl From<ProxyPath> for HostPath {
    /// Convert proxy path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The proxy path to convert
    ///
    /// # Returns
    ///
    /// * `HostPath` - The host path
    fn from(value: ProxyPath) -> Self {
        HostPath::Proxy(value)
    }
}

impl Path for ProxyPath {
    /// Get the URI of the proxy path
    ///
    /// # Returns
    ///
    /// * `&str` - The URI of the proxy path
    fn uri(&self) -> &str {
        self.config.uri()
    }

    /// Handle proxy request
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle
    /// * `uri` - The URI of the request
    ///
    /// # Returns
    ///
    /// * `Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>` - The future that will resolve to the response
    fn handle(
        &self,
        request: Request,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        let (request_parts, _request_body) = request.into_http_parts();

        let target = self.config.target();

        Box::pin(async move {
            let target_url = format!("{}{}", target, uri);
            let deboa_request = match DeboaRequest::at(target_url, request_parts.method) {
                Ok(request) => request,
                Err(e) => {
                    return Err(VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))
                }
            };

            let deboa_request = match deboa_request
                .headers(request_parts.headers)
                // TODO: Set body
                .build()
            {
                Ok(request) => request,
                Err(e) => {
                    return Err(VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))
                }
            };

            let client = CLIENT.get_or_init(|| {
                Client::builder()
                    .pool(HttpConnectionPool::default())
                    .build()
            });

            // TODO: Check errors and handle them properly by returning a proper response 500, 503 or 504
            let response = client
                .execute(deboa_request)
                .await;

            let response = match response {
                Ok(response) => response,
                Err(e) => {
                    return Err(VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))
                }
            };

            let (response_parts, response_body) = response.into_parts();

            let vetis_response = Response::builder()
                .status(response_parts.status)
                .headers(response_parts.headers)
                .body(response_body);

            Ok::<Response, VetisError>(vetis_response)
        })
    }
}
