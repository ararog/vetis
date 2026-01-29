/// # Examples
///
/// ```rust,ignore
/// use vetis::{
///     config::VirtualHostConfig,
///     server::virtual_host::{DefaultVirtualHost, VirtualHost, handler_fn},
///     Request, Response,
/// };
///
/// // Create a virtual host with a simple handler
/// let config = VirtualHostConfig::builder()
///     .hostname("example.com".to_string())
///     .port(80)
///     .build()?;
///
/// let mut vhost = DefaultVirtualHost::new(config);
/// vhost.set_handler(handler_fn(|request: Request| async move {
///     let response = Response::builder()
///         .status(http::StatusCode::OK)
///         .body(http_body_util::Full::new(bytes::Bytes::from("Hello, World!")));
///     Ok(response)
/// }));
/// ```
use std::{future::Future, pin::Pin};

use crate::{config::VirtualHostConfig, errors::VetisError, Request, Response};

pub mod directory;

/// Type alias for boxed handler closures.
///
/// This represents an async function that takes a `Request` and returns
/// a `Response` or an error. Handlers are the core of request processing
/// in VeTiS virtual hosts.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::server::virtual_host::BoxedHandlerClosure;
/// use vetis::{Request, Response, errors::VetisError};
///
/// let handler: BoxedHandlerClosure = Box::new(|request: Request| {
///     Box::pin(async move {
///         // Process request...
///         Ok(Response::builder()
///             .status(http::StatusCode::OK)
///             .body(http_body_util::Full::new(bytes::Bytes::from("OK"))))
///     })
/// });
/// ```
pub type BoxedHandlerClosure = Box<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>>
        + Send
        + Sync,
>;

/// Creates a handler closure from a function.
///
/// This utility function converts any compatible async function into a
/// `BoxedHandlerClosure` that can be used with virtual hosts.
///
/// # Arguments
///
/// * `f` - An async function that takes a `Request` and returns a `Result<Response, VetisError>`
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::{
///     server::virtual_host::{handler_fn, VirtualHost, DefaultVirtualHost},
///     config::VirtualHostConfig,
///     Request, Response,
/// };
///
/// async fn hello_handler(request: Request) -> Result<Response, vetis::VetisError> {
///     Ok(Response::builder()
///         .status(http::StatusCode::OK)
///         .body(http_body_util::Full::new(bytes::Bytes::from("Hello!"))))
/// }
///
/// let config = VirtualHostConfig::builder()
///     .hostname("example.com".to_string())
///     .port(80)
///     .build()?;
///
/// let mut vhost = DefaultVirtualHost::new(config);
/// vhost.set_handler(handler_fn(hello_handler));
/// ```
pub fn handler_fn<F, Fut>(f: F) -> BoxedHandlerClosure
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response, VetisError>> + Send + Sync + 'static,
{
    Box::new(move |req| Box::pin(f(req)))
}

/// Trait for virtual host implementations.
///
/// Virtual hosts allow multiple domains to be served by a single server instance.
/// Each virtual host has its own configuration, security settings, and request handler.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::{
///     config::VirtualHostConfig,
///     server::virtual_host::{VirtualHost, DefaultVirtualHost, handler_fn},
///     Request, Response,
/// };
///
/// // Create a virtual host
/// let config = VirtualHostConfig::builder()
///     .hostname("api.example.com".to_string())
///     .port(443)
///     .build()?;
///
/// let mut vhost = DefaultVirtualHost::new(config);
/// vhost.set_handler(handler_fn(|request| async move {
///     Ok(Response::builder()
///         .status(http::StatusCode::OK)
///         .body(http_body_util::Full::new(bytes::Bytes::from("API response"))))
/// }));
///
/// println!("Virtual host: {}:{}", vhost.hostname(), vhost.port());
/// ```
pub trait VirtualHost: Send + Sync + 'static {
    /// Creates a new virtual host with the given configuration.
    fn new(config: VirtualHostConfig) -> Self
    where
        Self: Sized;

    /// Returns a reference to the virtual host configuration.
    fn config(&self) -> &VirtualHostConfig;

    /// Returns the hostname for this virtual host.
    fn hostname(&self) -> String;

    /// Returns the port for this virtual host.
    fn port(&self) -> u16;

    /// Returns whether this virtual host uses HTTPS.
    fn is_secure(&self) -> bool;

    /// Sets the request handler for this virtual host.
    fn set_handler(&mut self, handler: BoxedHandlerClosure);

    /// Executes the handler for the given request.
    fn execute(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>>;
}

// All of them should have a handler to process requests
pub struct DefaultVirtualHost {
    config: VirtualHostConfig,
    handler: Option<BoxedHandlerClosure>,
}

impl VirtualHost for DefaultVirtualHost {
    fn new(config: VirtualHostConfig) -> Self {
        Self { config, handler: None }
    }

    fn config(&self) -> &VirtualHostConfig {
        &self.config
    }

    fn hostname(&self) -> String {
        self.config
            .hostname()
            .clone()
    }

    fn port(&self) -> u16 {
        self.config.port()
    }

    fn is_secure(&self) -> bool {
        self.config
            .security()
            .is_some()
    }

    fn set_handler(&mut self, handler: BoxedHandlerClosure) {
        self.handler = Some(handler);
    }

    fn execute(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        if let Some(handler) = &self.handler {
            handler(request)
        } else {
            Box::pin(async move { Err(VetisError::Handler("No handler set".to_string())) })
        }
    }
}

impl<V: VirtualHost> VirtualHost for Box<V> {
    fn new(config: VirtualHostConfig) -> Self
    where
        Self: Sized,
    {
        Box::new(V::new(config))
    }

    fn config(&self) -> &VirtualHostConfig {
        self.as_ref()
            .config()
    }

    fn hostname(&self) -> String {
        self.as_ref()
            .hostname()
    }

    fn port(&self) -> u16 {
        self.as_ref().port()
    }

    fn is_secure(&self) -> bool {
        self.as_ref()
            .is_secure()
    }

    fn set_handler(&mut self, handler: BoxedHandlerClosure) {
        self.as_mut()
            .set_handler(handler)
    }

    fn execute(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        self.as_ref()
            .execute(request)
    }
}
