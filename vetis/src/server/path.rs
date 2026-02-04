use std::{future::Future, pin::Pin};

#[cfg(feature = "reverse-proxy")]
use crate::config::ProxyPathConfig;
#[cfg(feature = "reverse-proxy")]
use deboa::{client::conn::pool::HttpConnectionPool, request::DeboaRequest, Client};
#[cfg(feature = "reverse-proxy")]
use std::sync::OnceLock;

#[cfg(feature = "static-files")]
use crate::config::StaticPathConfig;
#[cfg(feature = "static-files")]
use bytes::Bytes;
#[cfg(feature = "static-files")]
use http_body_util::Full;
#[cfg(feature = "static-files")]
use serde::Deserialize;
#[cfg(feature = "static-files")]
use std::{fs, path::PathBuf};

use std::sync::Arc;

use crate::{
    errors::{VetisError, VirtualHostError},
    server::virtual_host::BoxedHandlerClosure,
    Request, Response,
};

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

pub struct HandlerPathBuilder {
    uri: Arc<str>,
    handler: Option<BoxedHandlerClosure>,
}

impl HandlerPathBuilder {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Arc::from(uri);
        self
    }

    pub fn handler(mut self, handler: BoxedHandlerClosure) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn build(self) -> Result<HostPath, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::VirtualHost(VirtualHostError::InvalidPath(
                "URI cannot be empty".to_string(),
            )));
        }

        if self
            .handler
            .is_none()
        {
            return Err(VetisError::VirtualHost(VirtualHostError::InvalidPath(
                "Handler cannot be empty".to_string(),
            )));
        }

        Ok(HostPath::Handler(HandlerPath {
            uri: self.uri,
            handler: self
                .handler
                .unwrap(),
        }))
    }
}

pub struct HandlerPath {
    uri: Arc<str>,
    handler: BoxedHandlerClosure,
}

impl HandlerPath {
    pub fn builder() -> HandlerPathBuilder {
        HandlerPathBuilder { uri: Arc::from(""), handler: None }
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
#[derive(Deserialize)]
pub struct StaticPath {
    config: StaticPathConfig,
}

#[cfg(feature = "static-files")]
impl StaticPath {
    pub fn new(config: StaticPathConfig) -> StaticPath {
        StaticPath { config }
    }

    fn serve_file(&self, file: PathBuf) -> Result<Response, VetisError> {
        let result = fs::read(file);
        if let Ok(data) = result {
            return Ok(Response::builder()
                .status(http::StatusCode::OK)
                .body(http_body_util::Either::Right(Full::new(Bytes::from(data)))));
        }

        self.serve_status_page(404)
    }

    fn serve_index_file(&self, directory: PathBuf) -> Result<Response, VetisError> {
        if let Some(index_files) = self
            .config
            .index_files()
        {
            if let Some(index_file) = index_files
                .iter()
                .find(|index_file| {
                    directory
                        .join(index_file)
                        .exists()
                })
            {
                return self.serve_file(directory.join(index_file));
            }
        }

        self.serve_status_page(404)
    }

    fn serve_status_page(&self, status: u16) -> Result<Response, VetisError> {
        let not_found_response = Response::builder()
            .status(http::StatusCode::from_u16(status).unwrap())
            .text("Not found");

        if let Some(status_pages) = &self
            .config
            .status_pages()
        {
            if let Some(page) = status_pages.get(&status) {
                let file = PathBuf::from(
                    self.config
                        .directory(),
                )
                .join(page);
                if file.exists() {
                    return self.serve_file(file);
                }
            }
        }
        Ok(not_found_response)
    }
}

#[cfg(feature = "static-files")]
impl From<StaticPath> for HostPath {
    fn from(value: StaticPath) -> Self {
        HostPath::Static(value)
    }
}

#[cfg(feature = "static-files")]
impl Path for StaticPath {
    fn uri(&self) -> &str {
        self.config.uri()
    }

    fn handle(
        &self,
        _request: Request,
        uri: Arc<str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        Box::pin(async move {
            let ext_regex = regex::Regex::new(
                self.config
                    .extensions(),
            );

            let directory = PathBuf::from(
                self.config
                    .directory(),
            );

            let uri = uri
                .strip_prefix("/")
                .unwrap_or(&uri);
            let file = directory.join(uri);
            if file.is_file() && !file.exists() {
                // check file by mimetype
                if let Ok(ext_regex) = ext_regex {
                    if !ext_regex.is_match(uri.as_ref()) {
                        return self.serve_index_file(directory);
                    }
                }
            } else if file.is_dir() {
                return self.serve_index_file(file);
            }

            self.serve_file(file)
        })
    }
}

#[cfg(feature = "reverse-proxy")]
pub struct ProxyPath {
    config: ProxyPathConfig,
}

#[cfg(feature = "reverse-proxy")]
impl ProxyPath {
    pub fn new(config: ProxyPathConfig) -> ProxyPath {
        ProxyPath { config }
    }
}

#[cfg(feature = "reverse-proxy")]
impl From<ProxyPath> for HostPath {
    fn from(value: ProxyPath) -> Self {
        HostPath::Proxy(value)
    }
}

#[cfg(feature = "reverse-proxy")]
impl Path for ProxyPath {
    fn uri(&self) -> &str {
        self.config.uri()
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

        let target = self.config.target();

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
