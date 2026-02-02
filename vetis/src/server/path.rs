use std::{future::Future, pin::Pin};

#[cfg(any(feature = "static-files", feature = "reverse-proxy"))]
use crate::errors::VirtualHostError;

#[cfg(feature = "static-files")]
use std::fs;

#[cfg(feature = "reverse-proxy")]
use deboa::{client::conn::pool::HttpConnectionPool, request::DeboaRequest, Client};
#[cfg(feature = "reverse-proxy")]
use std::sync::OnceLock;

use std::sync::Arc;

use crate::{errors::VetisError, server::virtual_host::BoxedHandlerClosure, Request, Response};

#[cfg(feature = "reverse-proxy")]
static CLIENT: OnceLock<Client> = OnceLock::new();

pub trait Path {
    fn uri(&self) -> &str;
    fn handle(
        &self,
        request: Request,
        uri: Arc<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>;
}

pub enum HostPath {
    Handler(HandlerPath),
    #[cfg(feature = "reverse-proxy")]
    Proxy(ProxyPath),
    #[cfg(feature = "static-files")]
    Static(StaticPath),
}

impl Path for HostPath {
    fn uri(&self) -> &str {
        match self {
            HostPath::Handler(handler) => handler.uri(),
            #[cfg(feature = "reverse-proxy")]
            HostPath::Proxy(proxy) => proxy.uri(),
            #[cfg(feature = "static-files")]
            HostPath::Static(static_path) => static_path.uri(),
        }
    }

    fn handle(
        &self,
        request: Request,
        uri: Arc<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        match self {
            HostPath::Handler(handler) => handler.handle(request, uri),
            #[cfg(feature = "reverse-proxy")]
            HostPath::Proxy(proxy) => proxy.handle(request, uri),
            #[cfg(feature = "static-files")]
            HostPath::Static(static_path) => static_path.handle(request, uri),
        }
    }
}

pub struct HandlerPath {
    uri: Arc<str>,
    handler: BoxedHandlerClosure,
}

impl HandlerPath {
    pub fn new_host_path(uri: &str, handler: BoxedHandlerClosure) -> HostPath {
        HostPath::Handler(Self { uri: Arc::from(uri), handler })
    }
}

impl Path for HandlerPath {
    fn uri(&self) -> &str {
        self.uri.as_ref()
    }

    fn handle(
        &self,
        request: Request,
        _uri: Arc<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        (self.handler)(request)
    }
}

#[cfg(feature = "static-files")]
pub struct StaticPathBuilder {
    uri: Arc<str>,
    extensions: Arc<str>,
    directory: Arc<str>,
}

#[cfg(feature = "static-files")]
impl StaticPathBuilder {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Arc::from(uri);
        self
    }

    pub fn extensions(mut self, extensions: &str) -> Self {
        self.extensions = Arc::from(extensions);
        self
    }

    pub fn directory(mut self, directory: &str) -> Self {
        self.directory = Arc::from(directory);
        self
    }

    pub fn build(self) -> Result<HostPath, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::VirtualHost(VirtualHostError::InvalidPath(
                "URI cannot be empty".to_string(),
            )));
        }
        if self
            .extensions
            .is_empty()
        {
            return Err(VetisError::VirtualHost(VirtualHostError::InvalidPath(
                "Extensions cannot be empty".to_string(),
            )));
        }
        if self
            .directory
            .is_empty()
        {
            return Err(VetisError::VirtualHost(VirtualHostError::InvalidPath(
                "Directory cannot be empty".to_string(),
            )));
        }

        Ok(HostPath::Static(StaticPath {
            uri: self.uri,
            extensions: self.extensions,
            directory: self.directory,
        }))
    }
}

#[cfg(feature = "static-files")]
pub struct StaticPath {
    uri: Arc<str>,
    extensions: Arc<str>,
    directory: Arc<str>,
}

#[cfg(feature = "static-files")]
impl StaticPath {
    pub fn extensions(&self) -> &str {
        &self.extensions
    }

    pub fn directory(&self) -> &str {
        &self.directory
    }

    pub fn builder() -> StaticPathBuilder {
        StaticPathBuilder {
            uri: Arc::from(""),
            extensions: Arc::from(""),
            directory: Arc::from(""),
        }
    }
}

#[cfg(feature = "static-files")]
impl Path for StaticPath {
    fn uri(&self) -> &str {
        self.uri.as_ref()
    }

    fn handle(
        &self,
        _request: Request,
        uri: Arc<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        Box::pin(async move {
            let ext_regex = regex::Regex::new(&self.extensions);
            if let Ok(ext_regex) = ext_regex {
                if !ext_regex.is_match(uri.as_ref()) {
                    return Ok(Response::builder()
                        .status(http::StatusCode::BAD_REQUEST)
                        .text("Invalid file extension"));
                }
            }

            let result = fs::read(format!("{}/{}", self.directory, uri));
            if let Ok(data) = result {
                use bytes::Bytes;
                use http_body_util::Full;

                return Ok(Response::builder()
                    .status(http::StatusCode::OK)
                    .body(http_body_util::Either::Right(Full::new(Bytes::from(data)))));
            }

            // TODO: return 404
            Ok(Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .text("Not found"))
        })
    }
}

#[cfg(feature = "reverse-proxy")]
pub struct ProxyPathBuilder {
    uri: Arc<str>,
    target: Arc<str>,
}

#[cfg(feature = "reverse-proxy")]
impl ProxyPathBuilder {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Arc::from(uri);
        self
    }

    pub fn target(mut self, target: &str) -> Self {
        self.target = Arc::from(target);
        self
    }

    pub fn build(self) -> Result<HostPath, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::VirtualHost(VirtualHostError::InvalidPath(
                "URI cannot be empty".to_string(),
            )));
        }
        if self
            .target
            .is_empty()
        {
            return Err(VetisError::VirtualHost(VirtualHostError::InvalidPath(
                "Target cannot be empty".to_string(),
            )));
        }

        Ok(HostPath::Proxy(ProxyPath { uri: self.uri, target: self.target }))
    }
}

#[cfg(feature = "reverse-proxy")]
pub struct ProxyPath {
    uri: Arc<str>,
    target: Arc<str>,
}

#[cfg(feature = "reverse-proxy")]
impl ProxyPath {
    pub fn builder() -> ProxyPathBuilder {
        ProxyPathBuilder { uri: "".into(), target: "".into() }
    }

    pub fn target(&self) -> &str {
        self.target.as_ref()
    }
}

#[cfg(feature = "reverse-proxy")]
impl Path for ProxyPath {
    fn uri(&self) -> &str {
        self.uri.as_ref()
    }

    fn handle(
        &self,
        request: Request,
        uri: Arc<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        let (request_parts, request_body) = request.into_http_parts();

        let target_path = request_parts
            .uri
            .path()
            .strip_prefix(uri.as_ref())
            .unwrap_or("");

        let target_path = Arc::<str>::from(target_path);

        let target = self.target();

        Box::pin(async move {
            let target_url = format!("{}{}", target, target_path);
            let deboa_request = DeboaRequest::at(target_url, request_parts.method)
                .map_err(|e| VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))?
                .headers(request_parts.headers)
                .build()
                .map_err(|e| VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))?;

            let client = CLIENT.get_or_init(|| {
                Client::builder()
                    .pool(HttpConnectionPool::default())
                    .build()
            });

            let response = client
                .execute(deboa_request)
                .await
                .map_err(|e| VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))?;

            let (response_parts, response_body) = response.into_parts();

            let vetis_response = Response::builder()
                .status(response_parts.status)
                .headers(response_parts.headers)
                .body(response_body);

            Ok::<Response, VetisError>(vetis_response)
        })
    }
}
