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
use std::{borrow::Cow, future::Future, pin::Pin};

use arcstr::ArcStr;
use radix_trie::Trie;

use crate::{
    config::VirtualHostConfig,
    errors::VetisError,
    server::path::{HostPath, Path},
    Request, Response,
};

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

// All of them should have a handler to process requests
pub struct VirtualHost {
    config: VirtualHostConfig,
    paths: Trie<String, HostPath>,
}

impl VirtualHost {
    pub fn new(config: VirtualHostConfig) -> Self {
        Self { config, paths: Trie::new() }
    }

    pub fn add_path(&mut self, path: HostPath) {
        self.paths.insert(
            path.uri()
                .to_string(),
            path,
        );
    }

    pub fn config(&self) -> &VirtualHostConfig {
        &self.config
    }

    pub fn hostname(&self) -> &str {
        self.config
            .hostname()
            .clone()
    }

    pub fn port(&self) -> u16 {
        self.config.port()
    }

    pub fn is_secure(&self) -> bool {
        self.config
            .security()
            .is_some()
    }

    pub fn route(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        let uri_path = request
            .uri()
            .path()
            .to_string();

        let matches = self
            .paths
            .get_ancestor_value(&uri_path);

        let Some(path) = matches else {
            return Box::pin(async move {
                Ok(Response::builder()
                    .status(http::StatusCode::NOT_FOUND)
                    .text("Not Found"))
            });
        };

        let target_path = uri_path
            .strip_prefix(path.uri())
            .unwrap_or(&uri_path);

        path.handle(request, ArcStr::from(target_path))
    }
}
