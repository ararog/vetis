use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;
#[cfg(feature = "python")]
use pyo3::Python;

#[cfg(feature = "php")]
use crate::server::virtual_host::path::interface::php::PhpWorker;
#[cfg(feature = "python")]
use crate::server::virtual_host::path::interface::python::asgi::AsgiWorker;
#[cfg(feature = "rsgi")]
use crate::server::virtual_host::path::interface::rsgi::RsgiWorker;
use crate::{
    config::server::virtual_host::path::interface::{InterfacePathConfig, InterfaceType},
    errors::VetisError,
    server::virtual_host::path::{interface::python::wsgi::WsgiWorker, HostPath, Path},
    Request, Response, VetisBody, VetisBodyExt,
};

#[cfg(feature = "php")]
pub mod php;
#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "rsgi")]
pub mod rsgi;

pub trait InterfaceWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>>;
}

pub enum Interface {
    #[cfg(feature = "php")]
    Php(PhpWorker),
    #[cfg(feature = "python")]
    Asgi(AsgiWorker),
    #[cfg(feature = "python")]
    Wsgi(WsgiWorker),
    #[cfg(feature = "rsgi")]
    Rsgi(RsgiWorker),
}

impl InterfaceWorker for Interface {
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
        request: Arc<Request>,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        #[cfg(feature = "python")]
        Python::initialize();

        match self {
            #[cfg(feature = "php")]
            Interface::Php(handler) => handler.handle(request, uri),
            #[cfg(feature = "python")]
            Interface::Asgi(handler) => handler.handle(request, uri),
            #[cfg(feature = "python")]
            Interface::Wsgi(handler) => handler.handle(request, uri),
            #[cfg(feature = "rsgi")]
            Interface::Rsgi(handler) => handler.handle(request, uri),
            _ => {
                panic!("Unsupported interface type");
            }
        }
    }
}

/// Proxy path
pub struct InterfacePath {
    config: InterfacePathConfig,
    interface: Interface,
}

impl InterfacePath {
    /// Create a new proxy path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The proxy path configuration
    ///
    /// # Returns
    ///
    /// * `InterfacePath` - The proxy path
    pub fn new(config: InterfacePathConfig) -> InterfacePath {
        let file = config
            .target()
            .to_string();

        let interface = match config.interface_type() {
            #[cfg(feature = "php")]
            InterfaceType::Php => Interface::Php(PhpWorker::new(file)),
            #[cfg(feature = "python")]
            InterfaceType::Asgi => Interface::Asgi(AsgiWorker::new(file)),
            #[cfg(feature = "python")]
            InterfaceType::Wsgi => Interface::Wsgi(WsgiWorker::new(file)),
            #[cfg(feature = "rsgi")]
            InterfaceType::Rsgi => Interface::Rsgi(RsgiWorker::new(file)),
            _ => {
                panic!("Unsupported interface type");
            }
        };

        InterfacePath { config, interface }
    }
}

impl From<InterfacePath> for HostPath {
    /// Convert proxy path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The proxy path to convert
    ///
    /// # Returns
    ///
    /// * `HostPath` - The host path
    fn from(value: InterfacePath) -> Self {
        HostPath::Interface(value)
    }
}

impl Path for InterfacePath {
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
        let request = Arc::new(request);

        Box::pin(async move {
            let response = match self
                .config
                .interface_type()
            {
                #[cfg(feature = "php")]
                InterfaceType::Php => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "python")]
                InterfaceType::Asgi => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "python")]
                InterfaceType::Wsgi => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "rsgi")]
                InterfaceType::Rsgi => self
                    .interface
                    .handle(request.clone(), uri),
                _ => {
                    panic!("Unsupported interface type");
                }
            };

            let response = response.await?;

            Ok::<Response, VetisError>(response)
        })
    }
}
