//! Path module for handling different types of paths in the server

use std::{future::Future, pin::Pin};

#[cfg(feature = "reverse-proxy")]
use crate::config::ProxyPathConfig;
#[cfg(feature = "reverse-proxy")]
use deboa::{client::conn::pool::HttpConnectionPool, request::DeboaRequest, Client};
#[cfg(feature = "reverse-proxy")]
use std::sync::OnceLock;

#[cfg(all(feature = "static-files", feature = "smol-rt"))]
use futures_lite::AsyncSeekExt;
#[cfg(all(feature = "static-files", feature = "smol-rt"))]
use smol::fs::File;
#[cfg(all(feature = "static-files", feature = "tokio-rt"))]
use tokio::{fs::File, io::AsyncSeekExt};

#[cfg(feature = "static-files")]
use crate::{
    config::StaticPathConfig, errors::FileError, server::http::static_response, VetisBodyExt,
};
#[cfg(feature = "static-files")]
use http::{HeaderMap, HeaderValue};
#[cfg(feature = "static-files")]
use std::path::PathBuf;

use std::sync::Arc;

use crate::{
    errors::{HandlerError, VetisError, VirtualHostError},
    server::virtual_host::BoxedHandlerClosure,
    Request, Response, VetisBody,
};

#[cfg(feature = "reverse-proxy")]
static CLIENT: OnceLock<Client> = OnceLock::new();

pub trait Path {
    fn uri(&self) -> &str;
    fn handle(
        &self,
        request: Request,
        uri: Arc<String>,
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
        uri: Arc<String>,
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
    uri: Arc<String>,
    handler: Option<BoxedHandlerClosure>,
}

impl HandlerPathBuilder {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Arc::from(uri.to_string());
        self
    }

    pub fn handler(mut self, handler: BoxedHandlerClosure) -> Self {
        self.handler = Some(handler);
        self
    }

    pub fn build(self) -> Result<HostPath, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::VirtualHost(VirtualHostError::Handler(HandlerError::Uri(
                "URI cannot be empty".to_string(),
            ))));
        }

        if self
            .handler
            .is_none()
        {
            return Err(VetisError::VirtualHost(VirtualHostError::Handler(HandlerError::Handler(
                "Handler cannot be empty".to_string(),
            ))));
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
    uri: Arc<String>,
    handler: BoxedHandlerClosure,
}

impl HandlerPath {
    pub fn builder() -> HandlerPathBuilder {
        HandlerPathBuilder { uri: Arc::from("/".to_string()), handler: None }
    }
}

impl Path for HandlerPath {
    fn uri(&self) -> &str {
        self.uri.as_ref()
    }

    fn handle(
        &self,
        request: Request,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        (self.handler)(request)
    }
}

#[cfg(feature = "static-files")]
pub struct StaticPath {
    config: StaticPathConfig,
}

#[cfg(feature = "static-files")]
impl StaticPath {
    pub fn new(config: StaticPathConfig) -> StaticPath {
        StaticPath { config }
    }

    pub async fn serve_file(
        &self,
        file: PathBuf,
        range: Option<&str>,
    ) -> Result<Response, VetisError> {
        let result = File::open(file).await;
        if let Ok(mut data) = result {
            let filesize = match data
                .metadata()
                .await
            {
                Ok(metadata) => metadata.len(),
                Err(_) => 0u64,
            };

            if let Some(range) = range {
                let (unit, range) = range
                    .split_once("=")
                    .unwrap();
                if unit != "bytes" {
                    return Err(VetisError::VirtualHost(VirtualHostError::File(
                        FileError::InvalidRange,
                    )));
                }

                let (start, end) = range
                    .split_once("-")
                    .unwrap();
                let start = start
                    .parse::<u64>()
                    .unwrap();
                let end = end
                    .parse::<u64>()
                    .unwrap();
                if start > end || start >= filesize {
                    return Ok(Response::builder()
                        .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
                        .body(VetisBody::body_from_text("")));
                } else if start < end
                    && end < filesize
                    && data
                        .seek(std::io::SeekFrom::Start(start))
                        .await
                        .is_ok()
                {
                    return Ok(Response::builder()
                        .status(http::StatusCode::PARTIAL_CONTENT)
                        .body(VetisBody::body_from_file(data)));
                }
            }

            return Ok(Response::builder()
                .status(http::StatusCode::OK)
                .header(
                    http::header::ACCEPT_RANGES,
                    "bytes"
                        .parse()
                        .unwrap(),
                )
                .header(http::header::CONTENT_LENGTH, HeaderValue::from(filesize))
                .body(VetisBody::body_from_file(data)));
        }

        Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
    }

    async fn serve_index_file(&self, directory: PathBuf) -> Result<Response, VetisError> {
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
                return self
                    .serve_file(directory.join(index_file), None)
                    .await;
            }
        }

        Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
    }

    fn serve_metadata(&self, file: PathBuf) -> Result<Response, VetisError> {
        if let Ok(metadata) = file.metadata() {
            let len = metadata.len();
            let mut headers = HeaderMap::new();
            match len
                .to_string()
                .parse()
            {
                Ok(len) => {
                    headers.insert(http::header::CONTENT_LENGTH, len);
                }
                Err(_) => todo!(),
            }
            let last_modified = metadata.modified();
            match last_modified {
                Ok(date) => {
                    let date = crate::utils::date::format_date(date);
                    headers.insert(
                        http::header::LAST_MODIFIED,
                        date.parse()
                            .unwrap(),
                    );
                }
                Err(_) => todo!(),
            }
            match file.file_name() {
                Some(filename) => {
                    let mime_type = minimime::lookup_by_filename(
                        filename
                            .to_str()
                            .unwrap(),
                    );
                    if let Some(mime_type) = mime_type {
                        headers.insert(
                            http::header::CONTENT_TYPE,
                            HeaderValue::from_str(
                                mime_type
                                    .content_type
                                    .as_str(),
                            )
                            .unwrap(),
                        );
                    }
                }
                None => {
                    todo!()
                }
            }

            Ok(Response {
                inner: static_response(http::StatusCode::OK, Some(headers), String::new()),
            })
        } else {
            Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
        }
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
        request: Request,
        uri: Arc<String>,
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
                        return self
                            .serve_index_file(directory)
                            .await;
                    }
                }
            } else if file.is_dir() {
                return self
                    .serve_index_file(file)
                    .await;
            }

            if request.method() == http::Method::HEAD {
                return self.serve_metadata(file);
            }

            let range = if request
                .headers()
                .contains_key(http::header::RANGE)
            {
                let value = request
                    .headers()
                    .get(http::header::RANGE);
                Some(
                    value
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
            } else {
                None
            };

            self.serve_file(file, range)
                .await
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
