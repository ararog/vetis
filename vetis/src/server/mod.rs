//! Server implementation and virtual host system.
//!
//! This module provides the core HTTP server implementation and the virtual host
//! system that allows multiple domains to be served by a single server instance.
//!
//! # Modules
//!
//! - [`conn`]: Connection handling for different protocols
//! - [`http`]: HTTP/1 and HTTP/2 server implementation
//! - [`tls`]: TLS/SSL support for secure connections
//! - [`virtual_host`]: Virtual host system and request handlers
//!
//! # Examples
//!
//! ```rust,ignore
//! use vetis::{
//!     config::{ServerConfig, VirtualHostConfig},
//!     server::virtual_host::{DefaultVirtualHost, VirtualHost, handler_fn},
//! };
//!
//! // Create a virtual host with a custom handler
//! let vhost_config = VirtualHostConfig::builder()
//!     .hostname("example.com".to_string())
//!     .port(80)
//!     .build()?;
//!
//! let mut vhost = DefaultVirtualHost::new(vhost_config);
//! vhost.set_handler(handler_fn(|request| async move {
//!     // Handle the request...
//!     Ok(vetis::Response::builder()
//!         .status(http::StatusCode::OK)
//!         .body(http_body_util::Full::new(bytes::Bytes::from("Hello"))))
//! }));
//! ```

use std::future::Future;

use crate::{config::ServerConfig, errors::VetisError, VetisVirtualHosts};

pub mod conn;
pub mod http;
pub mod tls;
pub mod virtual_host;

/// Trait for server implementations.
///
/// This trait defines the interface that all server implementations must provide.
/// It allows for different server backends while maintaining a consistent API.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::{Vetis, config::ServerConfig, errors::VetisError};
///
/// // Create a server instance
/// let config = ServerConfig::builder().build();
/// let mut server = Vetis::new(config);
///
/// // Start the server
/// async fn run_server() -> Result<(), VetisError> {
///     server.start().await?;
///     // Server is running...
///     server.stop().await?;
///     Ok(())
/// }
/// ```
pub trait Server {
    /// Creates a new server instance with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration containing listeners and settings
    fn new(config: ServerConfig) -> Self;

    /// Sets the virtual hosts for the server.
    ///
    /// This must be called before starting the server.
    ///
    /// # Arguments
    ///
    /// * `virtual_hosts` - Arc containing the virtual host registry
    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts);

    /// Starts the server and begins accepting connections.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start, bind to addresses,
    /// or initialize TLS.
    fn start(&mut self) -> impl Future<Output = Result<(), VetisError>>;

    /// Stops the server gracefully.
    ///
    /// This method waits for ongoing connections to complete
    /// before shutting down.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to stop properly.
    fn stop(&mut self) -> impl Future<Output = Result<(), VetisError>>;
}
