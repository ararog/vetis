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
///     .hostname("example.com")
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
use std::{future::Future, path::PathBuf, pin::Pin};

use radix_trie::Trie;
use std::sync::Arc;

use crate::{
    config::VirtualHostConfig,
    errors::{VetisError, VirtualHostError},
    server::path::{HostPath, Path},
    Request, Response, VetisBody, VetisBodyExt,
};

#[cfg(all(feature = "static-files", feature = "smol-rt"))]
use smol::fs::File;
#[cfg(all(feature = "static-files", feature = "tokio-rt"))]
use tokio::fs::File;

#[cfg(feature = "static-files")]
use crate::{errors::FileError, server::path::StaticPath};

#[cfg(feature = "reverse-proxy")]
use crate::server::path::ProxyPath;

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
///     .hostname("example.com")
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
    pub fn new(host_config: VirtualHostConfig) -> Self {
        let mut host = Self { config: host_config.clone(), paths: Trie::new() };

        #[cfg(feature = "static-files")]
        if let Some(static_paths) = &host_config.static_paths() {
            for static_path in static_paths {
                host.add_path(StaticPath::new(static_path.clone()));
            }
        }

        #[cfg(feature = "reverse-proxy")]
        if let Some(proxy_paths) = &host_config.proxy_paths() {
            for proxy_path in proxy_paths {
                host.add_path(ProxyPath::new(proxy_path.clone()));
            }
        }

        host
    }

    pub fn add_path<P>(&mut self, path: P)
    where
        P: Into<HostPath>,
    {
        let path = path.into();
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
    }

    pub fn port(&self) -> u16 {
        self.config.port()
    }

    pub fn is_secure(&self) -> bool {
        self.config
            .security()
            .is_some()
    }

    pub async fn serve_status_page(&self, status: u16) -> Result<Response, VetisError> {
        let not_found_response = Response::builder()
            .status(http::StatusCode::from_u16(status).unwrap())
            .text("Not found");

        if let Some(status_pages) = &self
            .config
            .status_pages()
        {
            if let Some(page) = status_pages.get(&status) {
                let file = PathBuf::from(page);
                if file.exists() {
                    let result = File::open(file).await;
                    if let Ok(data) = result {
                        return Ok(Response::builder()
                            .status(http::StatusCode::OK)
                            .body(VetisBody::body_from_file(data)));
                    }
                }
            }
        }
        Ok(not_found_response)
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
                self.serve_status_page(http::StatusCode::NOT_FOUND.as_u16())
                    .await
            });
        };

        let target_path = uri_path
            .strip_prefix(path.uri())
            .unwrap_or(&uri_path)
            .to_string();

        let result = path.handle(request, Arc::from(target_path));

        Box::pin(async move {
            match result.await {
                Ok(response) => Ok(response),
                Err(error) => {
                    match error {
                        VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)) => {
                            log::error!("Invalid path: {}", error);
                            return self
                                .serve_status_page(http::StatusCode::NOT_FOUND.as_u16())
                                .await;
                        }
                        VetisError::VirtualHost(VirtualHostError::Proxy(ref error)) => {
                            log::error!("Proxy error: {}", error);
                            return self
                                .serve_status_page(http::StatusCode::BAD_GATEWAY.as_u16())
                                .await;
                        }
                        _ => {}
                    }

                    Err(error)
                }
            }
        })
    }
}
