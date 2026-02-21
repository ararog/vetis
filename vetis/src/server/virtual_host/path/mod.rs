//! Path module for handling different types of paths in the server

use std::{future::Future, pin::Pin};

use std::sync::Arc;

#[cfg(feature = "interface")]
use crate::server::virtual_host::path::interface::InterfacePath;
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
#[cfg(feature = "interface")]
pub mod interface;
#[cfg(feature = "reverse-proxy")]
pub mod proxy;
#[cfg(feature = "static-files")]
pub mod static_files;

/// Trait for handling different types of paths in the server
pub trait Path {
    /// Returns the URI of the path
    ///
    /// # Returns
    ///
    /// * `&str` - The URI of the path
    fn uri(&self) -> &str;

    /// Handles the request for the path
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle
    /// * `uri` - The URI of the path
    ///
    /// # Returns
    ///
    /// * `Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>` - The future that will handle the request
    fn handle(
        &self,
        request: Request,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>;
}

/// Enum for different types of paths in the server
pub enum HostPath {
    /// Handler path
    Handler(HandlerPath),
    #[cfg(feature = "reverse-proxy")]
    /// Proxy path
    Proxy(ProxyPath),
    #[cfg(feature = "static-files")]
    /// Static path
    Static(StaticPath),
    #[cfg(feature = "interface")]
    /// Interface path
    Interface(InterfacePath),
}

impl Path for HostPath {
    /// Returns the URI of the path
    ///
    /// # Returns
    ///
    /// * `&str` - The URI of the path
    fn uri(&self) -> &str {
        match self {
            HostPath::Handler(handler) => handler.uri(),
            #[cfg(feature = "reverse-proxy")]
            HostPath::Proxy(proxy) => proxy.uri(),
            #[cfg(feature = "static-files")]
            HostPath::Static(static_path) => static_path.uri(),
            #[cfg(feature = "interface")]
            HostPath::Interface(interface_path) => interface_path.uri(),
        }
    }

    /// Handles the request for the path
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle
    /// * `uri` - The URI of the path
    ///
    /// # Returns
    ///
    /// * `Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>` - The future that will handle the request
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
            #[cfg(feature = "interface")]
            HostPath::Interface(interface_path) => interface_path.handle(request, uri),
        }
    }
}

/// Builder for handler path
pub struct HandlerPathBuilder {
    uri: Arc<String>,
    handler: Option<BoxedHandlerClosure>,
}

impl HandlerPathBuilder {
    /// Allow set handler uri path
    ///
    /// # Arguments
    ///
    /// * `uri` - The uri of the handler path
    ///
    /// # Returns
    ///
    /// * `Self` - The builder
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = Arc::from(uri.to_string());
        self
    }

    /// Allow set handler function
    ///
    /// # Arguments
    ///
    /// * `handler` - The handler function
    ///
    /// # Returns
    ///
    /// * `Self` - The builder
    pub fn handler(mut self, handler: BoxedHandlerClosure) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Build the handler path
    ///
    /// # Returns
    ///
    /// * `Result<HostPath, VetisError>` - The handler path or error
    pub fn build(self) -> Result<HostPath, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::VirtualHost(VirtualHostError::Handler(HandlerError::Uri(
                "URI cannot be empty".to_string(),
            ))));
        }

        let handler = match self.handler {
            Some(handler) => handler,
            None => {
                return Err(VetisError::VirtualHost(VirtualHostError::Handler(
                    HandlerError::Handler("Handler must be set".to_string()),
                )))
            }
        };

        Ok(HostPath::Handler(HandlerPath { uri: self.uri, handler }))
    }
}

/// Handler path
pub struct HandlerPath {
    uri: Arc<String>,
    handler: BoxedHandlerClosure,
}

impl HandlerPath {
    /// Allow create a new handler path builder
    ///
    /// # Returns
    ///
    /// * `HandlerPathBuilder` - The builder
    pub fn builder() -> HandlerPathBuilder {
        HandlerPathBuilder { uri: Arc::from("/".to_string()), handler: None }
    }
}

impl Path for HandlerPath {
    /// Allow get handler uri path
    ///
    /// # Returns
    ///
    /// * `&str` - The uri of the handler path
    fn uri(&self) -> &str {
        self.uri.as_ref()
    }

    /// Handles the request for the path
    ///
    /// # Arguments
    ///
    /// * `request` - The request to handle
    /// * `uri` - The URI of the path
    ///
    /// # Returns
    ///
    /// * `Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>` - The future that will handle the request
    fn handle(
        &self,
        request: Request,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        (self.handler)(request)
    }
}
