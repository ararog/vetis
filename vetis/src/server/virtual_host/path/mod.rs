//! Path module for handling different types of paths in the server

use std::{future::Future, pin::Pin};

use std::sync::Arc;

#[cfg(feature = "reverse-proxy")]
use crate::server::virtual_host::path::proxy::ProxyPath;
#[cfg(feature = "static-files")]
use crate::server::virtual_host::path::static_files::StaticPath;

use crate::{
    errors::{HandlerError, VetisError, VirtualHostError},
    server::virtual_host::BoxedHandlerClosure,
    Request, Response,
};

#[cfg(feature = "auth")]
pub mod auth;
#[cfg(feature = "gate")]
pub mod gate;
#[cfg(feature = "reverse-proxy")]
pub mod proxy;
#[cfg(feature = "static-files")]
pub mod static_files;

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
