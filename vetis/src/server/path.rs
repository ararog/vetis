use std::{borrow::Cow, fs, future::Future, pin::Pin};

use crate::{
    errors::{VetisError, VirtualHostError},
    server::virtual_host::BoxedHandlerClosure,
    Request, Response,
};

pub trait Path {
    fn uri(&self) -> &str;
    fn handle<'a>(
        &'a self,
        request: Request,
        uri: Cow<'a, str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'a>>;
}

pub enum HostPath {
    Handler(HandlerPath),
    #[cfg(feature = "reverse-proxy")]
    Proxy(Box<ProxyPath>),
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

    fn handle<'a>(
        &'a self,
        request: Request,
        uri: Cow<'a, str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'a>> {
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
    uri: String,
    handler: BoxedHandlerClosure,
}

impl HandlerPath {
    pub fn new_host_path(uri: String, handler: BoxedHandlerClosure) -> HostPath {
        HostPath::Handler(Self { uri, handler })
    }
}

impl Path for HandlerPath {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn handle<'a>(
        &'a self,
        request: Request,
        _uri: Cow<'a, str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'a>> {
        (self.handler)(request)
    }
}

#[cfg(feature = "static-files")]
pub struct StaticPathBuilder {
    uri: String,
    extensions: String,
    directory: String,
}

#[cfg(feature = "static-files")]
impl StaticPathBuilder {
    pub fn uri(mut self, uri: String) -> Self {
        self.uri = uri;
        self
    }

    pub fn extensions(mut self, extensions: String) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn directory(mut self, directory: String) -> Self {
        self.directory = directory;
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
    uri: String,
    extensions: String,
    directory: String,
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
            uri: String::new(),
            extensions: String::new(),
            directory: String::new(),
        }
    }
}

#[cfg(feature = "static-files")]
impl Path for StaticPath {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn handle<'a>(
        &'a self,
        _request: Request,
        uri: Cow<'a, str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'a>> {
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
    uri: String,
    target: String,
    client: deboa::Client,
}

#[cfg(feature = "reverse-proxy")]
impl ProxyPathBuilder {
    pub fn uri(mut self, uri: String) -> Self {
        self.uri = uri;
        self
    }

    pub fn target(mut self, target: String) -> Self {
        self.target = target;
        self
    }

    pub fn client(mut self, client: deboa::Client) -> Self {
        self.client = client;
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

        Ok(HostPath::Proxy(Box::new(ProxyPath {
            uri: self.uri,
            target: self.target,
            client: self.client,
        })))
    }
}

#[cfg(feature = "reverse-proxy")]
pub struct ProxyPath {
    uri: String,
    target: String,
    client: deboa::Client,
}

#[cfg(feature = "reverse-proxy")]
impl ProxyPath {
    pub fn builder() -> ProxyPathBuilder {
        ProxyPathBuilder {
            uri: String::new(),
            target: String::new(),
            client: deboa::Client::default(),
        }
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}

#[cfg(feature = "reverse-proxy")]
impl Path for ProxyPath {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn handle<'a>(
        &'a self,
        request: Request,
        uri: Cow<'a, str>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'a>> {
        let (request_parts, request_body) = request.into_http_parts();

        let target_path = request_parts
            .uri
            .path()
            .strip_prefix(uri.as_ref())
            .unwrap_or("")
            .to_string();

        let target = self
            .target()
            .to_string();

        Box::pin(async move {
            use deboa::request::DeboaRequest;

            let target_url = format!("{}{}", target, target_path);
            let deboa_request = DeboaRequest::at(target_url, request_parts.method)
                .map_err(|e| VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))?
                .headers(request_parts.headers)
                .build()
                .map_err(|e| VetisError::VirtualHost(VirtualHostError::Proxy(e.to_string())))?;

            let response = self
                .client
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
