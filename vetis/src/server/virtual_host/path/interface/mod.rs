use std::{future::Future, pin::Pin, sync::Arc};

use vetis_core::{
    errors::VetisError,
    http::{Request, Response},
    interface::InterfaceWorker,
};

#[cfg(feature = "asgi")]
use vetis_asgi::AsgiWorker;
#[cfg(feature = "fcgi")]
use vetis_fcgi::FcgiWorker;
#[cfg(feature = "rack")]
use vetis_rack::RackWorker;
#[cfg(feature = "rsgi")]
use vetis_rsgi::RsgiWorker;
#[cfg(feature = "sapi")]
use vetis_sapi::SapiWorker;
#[cfg(feature = "scgi")]
use vetis_scgi::ScgiWorker;
#[cfg(feature = "wsgi")]
use vetis_wsgi::WsgiWorker;

use crate::{
    config::server::virtual_host::path::interface::{InterfacePathConfig, InterfaceType},
    server::virtual_host::path::{HostPath, Path},
};

pub enum Interface {
    #[cfg(feature = "asgi")]
    Asgi(AsgiWorker),
    #[cfg(feature = "fcgi")]
    Fcgi(FcgiWorker),
    #[cfg(feature = "rack")]
    Rack(RackWorker),
    #[cfg(feature = "rsgi")]
    Rsgi(RsgiWorker),
    #[cfg(feature = "sapi")]
    Sapi(SapiWorker),
    #[cfg(feature = "scgi")]
    Scgi(ScgiWorker),
    #[cfg(feature = "wsgi")]
    Wsgi(WsgiWorker),
}

#[cfg(feature = "asgi")]
impl From<AsgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: AsgiWorker) -> Self {
        Interface::Asgi(value)
    }
}

#[cfg(feature = "fcgi")]
impl From<FcgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: FcgiWorker) -> Self {
        Interface::Fcgi(value)
    }
}

#[cfg(feature = "scgi")]
impl From<ScgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: ScgiWorker) -> Self {
        Interface::Scgi(value)
    }
}

#[cfg(feature = "sapi")]
impl From<SapiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: SapiWorker) -> Self {
        Interface::Sapi(value)
    }
}

#[cfg(feature = "rack")]
impl From<RackWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: RackWorker) -> Self {
        Interface::Rack(value)
    }
}

#[cfg(feature = "rsgi")]
impl From<RsgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: RsgiWorker) -> Self {
        Interface::Rsgi(value)
    }
}

#[cfg(feature = "wsgi")]
impl From<WsgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: WsgiWorker) -> Self {
        Interface::Wsgi(value)
    }
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
        match self {
            #[cfg(feature = "asgi")]
            Interface::Asgi(handler) => handler.handle(request, uri),
            #[cfg(feature = "fcgi")]
            Interface::Fcgi(handler) => handler.handle(request, uri),
            #[cfg(feature = "rack")]
            Interface::Rack(handler) => handler.handle(request, uri),
            #[cfg(feature = "rsgi")]
            Interface::Rsgi(handler) => handler.handle(request, uri),
            #[cfg(feature = "sapi")]
            Interface::Sapi(handler) => handler.handle(request, uri),
            #[cfg(feature = "scgi")]
            Interface::Scgi(handler) => handler.handle(request, uri),
            #[cfg(feature = "wsgi")]
            Interface::Wsgi(handler) => handler.handle(request, uri),
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
        let directory = config
            .directory()
            .to_string();
        let target = config
            .target()
            .to_string();

        let interface = match config.interface_type() {
            #[cfg(feature = "asgi")]
            InterfaceType::Asgi => Interface::Asgi(AsgiWorker::new(directory, target)),
            #[cfg(feature = "fcgi")]
            InterfaceType::Fcgi => {
                let worker = FcgiWorker::new(directory, target);
                match worker {
                    Ok(worker) => Interface::Fcgi(worker),
                    Err(e) => {
                        panic!("Could not initialize fcgi worker: {}", e);
                    }
                }
            }
            #[cfg(feature = "rack")]
            InterfaceType::Rack => Interface::Rack(RackWorker::new(directory, target)),
            #[cfg(feature = "rsgi")]
            InterfaceType::Rsgi => Interface::Rsgi(RsgiWorker::new(directory, target)),
            #[cfg(feature = "sapi")]
            InterfaceType::Sapi => Interface::Sapi(SapiWorker::new(directory, target)),
            #[cfg(feature = "scgi")]
            InterfaceType::Scgi => Interface::Scgi(ScgiWorker::new(directory, target)),
            #[cfg(feature = "wsgi")]
            InterfaceType::Wsgi => {
                let worker = WsgiWorker::new(directory, target);
                match worker {
                    Ok(worker) => Interface::Wsgi(worker),
                    Err(e) => {
                        panic!("Could not initialize wsgi worker: {}", e);
                    }
                }
            }
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
                #[cfg(feature = "asgi")]
                InterfaceType::Asgi => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "fcgi")]
                InterfaceType::Fcgi => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "rack")]
                InterfaceType::Rack => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "rsgi")]
                InterfaceType::Rsgi => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "sapi")]
                InterfaceType::Sapi => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "scgi")]
                InterfaceType::Scgi => self
                    .interface
                    .handle(request.clone(), uri),
                #[cfg(feature = "wsgi")]
                InterfaceType::Wsgi => self
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
