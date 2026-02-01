//! # VeTiS (Very Tiny Server)
//!
//! 🚀 **A blazingly fast, minimalist HTTP server built for modern Rust applications**
//!
//! VeTiS is a lightweight yet powerful web server that brings simplicity and performance together.
//! Designed with Rust's safety guarantees in mind, it delivers HTTP/1, HTTP/2, and HTTP/3 support
//! with a clean, intuitive API that makes building web services a breeze.
//!
//! ## ✨ Features
//!
//! - **🎯 Minimalist Design**: Focus on what matters - serving HTTP requests efficiently
//! - **🔧 Flexible Runtime**: Choose between Tokio or Smol async runtimes
//! - **🌐 Protocol Support**: Full HTTP/1, HTTP/2, and HTTP/3 implementation
//! - **🛡️ Secure by Default**: Built-in TLS support with modern cryptography
//! - **⚡ Zero-Cost Abstractions**: Leverage Rust's performance without overhead
//! - **📦 Feature-Gated**: Include only what you need for optimal binary size
//! - **🏠 Virtual Hosts**: Host multiple domains on a single server
//!
//! ## 🛠️ Quick Start
//!
//! Add VeTiS to your `Cargo.toml`:
//!
//! ```toml
//! vetis = { version = "0.1.3", features = ["tokio-rt", "http1", "tokio-rust-tls"] }
//! ```
//!
//! ## 💡 Basic Usage
//!
//! ```rust,ignore
//! use bytes::Bytes;
//! use http_body_util::Full;
//! use hyper::StatusCode;
//! use vetis::{
//!     Vetis,
//!     config::{ListenerConfig, SecurityConfig, ServerConfig, VirtualHostConfig},
//!     server::virtual_host::{DefaultVirtualHost, VirtualHost, handler_fn},
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure server listener
//!     let https = ListenerConfig::builder()
//!         .port(8443)
//!         .protocol(vetis::config::Protocol::HTTP1)
//!         .interface("0.0.0.0".to_string())
//!         .build();
//!
//!     let config = ServerConfig::builder()
//!         .add_listener(https)
//!         .build();
//!
//!     // Configure security (TLS)
//!     let security_config = SecurityConfig::builder()
//!         .cert_from_bytes(include_bytes!("server.der").to_vec())
//!         .key_from_bytes(include_bytes!("server.key.der").to_vec())
//!         .build();
//!
//!     // Configure virtual host
//!     let localhost_config = VirtualHostConfig::builder()
//!         .hostname("localhost".to_string())
//!         .port(8443)
//!         .security(security_config)
//!         .build()?;
//!
//!     let mut localhost_virtual_host = VirtualHost::new(localhost_config);
//!
//!     // Set up request handler
//!     let mut root_path = HandlerPath::new("/", handler_fn(|request| async move {
//!         let response = vetis::Response::builder()
//!             .status(StatusCode::OK)
//!             .body(Full::new(Bytes::from("Hello, World!")));
//!         Ok(response)
//!     }));
//!
//!     localhost_virtual_host.add_path(root_path);
//!
//!     // Create and run server
//!     let mut server = Vetis::new(config);
//!     server.add_virtual_host(localhost_virtual_host).await;
//!     server.run().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## 🏗️ Architecture
//!
//! VeTiS is built around several key components:
//!
//! - **[`Vetis`]**: Main server instance that manages virtual hosts and listeners
//! - **[`ServerConfig`]**: Configuration for server listeners and global settings
//! - **[`VirtualHost`]**: Trait for implementing virtual hosts that handle requests
//! - **[`Request`]**: HTTP request wrapper supporting multiple protocols
//! - **[`Response`]**: HTTP response builder for creating responses
//!
//! ## 🔧 Runtime Configuration
//!
//! VeTiS supports two async runtimes:
//!
//! - **Tokio** (default): Enable with `tokio-rt` feature
//! - **Smol**: Enable with `smol-rt` feature
//!
//! Only one runtime can be enabled at a time.
//!
//! ## 🌐 Protocol Support
//!
//! - **HTTP/1**: Enable with `http1` feature
//! - **HTTP/2**: Enable with `http2` feature (requires TLS)
//! - **HTTP/3**: Enable with `http3` feature (requires TLS)
//!
//! ## 🛡️ TLS Configuration
//!
//! For HTTPS support, enable one of:
//!
//! - **Tokio TLS**: `tokio-rust-tls` feature (default)
//! - **Smol TLS**: `smol-rust-tls` feature
//!
//! ## 📦 Modules
//!
//! - [`config`]: Server and virtual host configuration builders
//! - [`errors`]: Comprehensive error handling types
//! - [`server`]: HTTP server implementation and virtual host system
//!
//! ## 🚀 Examples
//!
//! Check out the `examples/` directory for more comprehensive examples including:
//!
//! - Basic HTTP server
//! - HTTPS with TLS
//! - Multiple virtual hosts
//! - Custom request handlers

#[cfg(all(
    any(feature = "http2", feature = "http3"),
    not(any(feature = "tokio-rust-tls", feature = "smol-rust-tls"))
))]
compile_error!("http2 and http3 requires tokio-rust-tls or smol-rust-tls!");

#[cfg(all(feature = "tokio-rt", feature = "smol-rt"))]
compile_error!("Only one runtime feature can be enabled at a time.");

use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use http_body_util::{Either, Full};
use hyper::body::Incoming;

use log::{error, info};

#[cfg(feature = "smol-rt")]
use async_signal::Signals;
#[cfg(feature = "smol-rt")]
use futures_lite::prelude::*;
#[cfg(feature = "smol-rt")]
use signal_hook::low_level;

#[cfg(feature = "smol-rt")]
use smol::lock::RwLock;

#[cfg(feature = "tokio-rt")]
use tokio::sync::RwLock;

pub(crate) type VetisRwLock<T> = RwLock<T>;

pub(crate) type VetisVirtualHosts = Arc<VetisRwLock<HashMap<(String, u16), VirtualHost>>>;

use crate::{
    config::ServerConfig,
    errors::{VetisError, VirtualHostError},
    server::{virtual_host::VirtualHost, Server},
};

pub mod config;
pub mod errors;
mod rt;
pub mod server;
mod tests;

/// Main server instance that manages virtual hosts and listeners.
///
/// The `Vetis` struct is the core of the VeTiS server. It handles:
/// - Managing multiple virtual hosts
/// - Coordinating server listeners
/// - Starting and stopping the server
/// - Signal handling for graceful shutdown
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::{Vetis, config::ServerConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ServerConfig::builder().build();
///     let mut server = Vetis::new(config);
///     
///     // Add virtual hosts...
///     
///     server.run().await?;
///     Ok(())
/// }
/// ```
pub struct Vetis {
    config: ServerConfig,
    virtual_hosts: VetisVirtualHosts,
    instance: Option<server::http::HttpServer>,
}

impl Vetis {
    /// Creates a new `Vetis` server instance with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration containing listeners and global settings
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::{Vetis, config::ServerConfig};
    ///
    /// let config = ServerConfig::builder().build();
    /// let server = Vetis::new(config);
    /// ```
    pub fn new(config: ServerConfig) -> Vetis {
        Vetis { config, virtual_hosts: Arc::new(VetisRwLock::new(HashMap::new())), instance: None }
    }

    /// Adds a virtual host to the server.
    ///
    /// Virtual hosts allow you to host multiple domains on a single server instance.
    /// Each virtual host is identified by its hostname and port combination.
    ///
    /// # Arguments
    ///
    /// * `virtual_host` - A type implementing the `VirtualHost` trait
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::{
    ///     Vetis,
    ///     config::{ServerConfig, VirtualHostConfig},
    ///     server::virtual_host::{VirtualHost, handler_fn},
    /// };
    ///
    /// let config = ServerConfig::builder().build();
    /// let mut server = Vetis::new(config);
    ///
    /// let vhost_config = VirtualHostConfig::builder()
    ///     .hostname("example.com".to_string())
    ///     .port(80)
    ///     .build()?;
    ///
    /// let mut vhost = VirtualHost::new(vhost_config);
    ///
    /// let mut root_path = HandlerPath::new("/", handler_fn(|request| async move {
    ///     let response = vetis::Response::builder()
    ///         .status(StatusCode::OK)
    ///         .body(Full::new(Bytes::from("Hello, World!")));
    ///     Ok(response)
    /// }));
    ///
    /// vhost.add_path(root_path);
    ///
    /// server.add_virtual_host(vhost).await;
    /// ```
    pub async fn add_virtual_host(&mut self, virtual_host: VirtualHost) {
        let key = (virtual_host.hostname(), virtual_host.port());

        self.virtual_hosts
            .write()
            .await
            .insert(key, virtual_host);
    }

    /// Returns a reference to the server configuration.
    ///
    /// This provides access to the listeners and global settings
    /// configured when the server was created.
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Starts the server and runs until interrupted.
    ///
    /// This method combines `start()` and graceful shutdown handling:
    /// 1. Starts the server with all configured virtual hosts
    /// 2. Listens for shutdown signals (Ctrl+C on Tokio, SIGQUIT on Smol)
    /// 3. Stops the server gracefully
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No virtual hosts have been added
    /// - Server fails to start
    /// - Server fails to stop
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::{Vetis, config::ServerConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ServerConfig::builder().build();
    ///     let mut server = Vetis::new(config);
    ///     
    ///     // Add virtual hosts...
    ///     
    ///     server.run().await?; // Runs until Ctrl+C
    ///     Ok(())
    /// }
    /// ```
    pub async fn run(&mut self) -> Result<(), VetisError> {
        self.start().await?;

        for listener in self
            .config
            .listeners()
        {
            info!("Server listening on port {}:{}", listener.interface(), listener.port());
        }

        #[cfg(feature = "tokio-rt")]
        let _ = tokio::signal::ctrl_c().await;

        #[cfg(feature = "smol-rt")]
        {
            use async_signal::Signal;

            let mut signals = Signals::new([Signal::Quit]).unwrap();
            while let Some(signal) = signals.next().await {
                low_level::emulate_default_handler(signal.unwrap() as i32).unwrap();
            }
        }

        info!("\nStopping server...");

        self.stop().await?;

        Ok(())
    }

    /// Starts the server without blocking.
    ///
    /// This method starts the server and returns immediately, allowing
    /// you to perform additional setup or handle shutdown manually.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No virtual hosts have been added
    /// - Server fails to bind to configured addresses
    /// - TLS configuration fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::{Vetis, config::ServerConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ServerConfig::builder().build();
    ///     let mut server = Vetis::new(config);
    ///     
    ///     // Add virtual hosts...
    ///     
    ///     server.start().await?;
    ///     
    ///     // Server is now running, do other work...
    ///     
    ///     server.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn start(&mut self) -> Result<(), VetisError> {
        if self
            .virtual_hosts
            .read()
            .await
            .is_empty()
        {
            error!("You must add at least one virtual host");
            return Err(VetisError::VirtualHost(VirtualHostError::NoVirtualHosts));
        }

        let mut server = server::http::HttpServer::new(self.config.clone());

        server.set_virtual_hosts(
            self.virtual_hosts
                .clone(),
        );

        server
            .start()
            .await?;
        self.instance = Some(server);

        Ok(())
    }

    /// Stops the server gracefully.
    ///
    /// This method shuts down all listeners and waits for ongoing
    /// requests to complete before returning.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No server instance is running
    /// - Server fails to stop properly
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::{Vetis, config::ServerConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = ServerConfig::builder().build();
    ///     let mut server = Vetis::new(config);
    ///     
    ///     server.start().await?;
    ///     // Server running...
    ///     server.stop().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn stop(&mut self) -> Result<(), VetisError> {
        if let Some(instance) = &mut self.instance {
            instance
                .stop()
                .await?;
        } else {
            return Err(VetisError::NoInstances);
        }
        Ok(())
    }
}

/// HTTP request wrapper supporting multiple protocols.
///
/// The `Request` struct provides a unified interface for handling HTTP requests
/// from different protocols (HTTP/1, HTTP/2, HTTP/3). It abstracts away the protocol-specific
/// details while providing access to common request properties.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::Request;
///
/// // In a request handler:
/// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
///     let method = request.method();
///     let uri = request.uri();
///     let user_agent = request.headers().get("user-agent");
///     
///     // Process request...
///     
///     Ok(vetis::Response::builder()
///         .status(http::StatusCode::OK)
///         .body(http_body_util::Full::new(bytes::Bytes::from("Hello"))))
/// }
/// ```
pub struct Request {
    pub(crate) inner_http: Option<http::Request<Incoming>>,
    pub(crate) inner_quic: Option<http::Request<Full<Bytes>>>,
}

impl Request {
    /// Creates a `Request` from an HTTP/1 or HTTP/2 request.
    ///
    /// This is used internally by the server to wrap incoming HTTP requests.
    pub fn from_http(req: http::Request<Incoming>) -> Self {
        Self { inner_http: Some(req), inner_quic: None }
    }

    /// Creates a `Request` from an HTTP/3 (QUIC) request.
    ///
    /// This is used internally by the server to wrap incoming QUIC requests.
    pub fn from_quic(req: http::Request<Full<Bytes>>) -> Self {
        Self { inner_http: None, inner_quic: Some(req) }
    }

    /// Returns the request URI.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Request;
    ///
    /// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
    ///     let path = request.uri().path();
    ///     let query = request.uri().query();
    ///     Ok(/* response */)
    /// }
    /// ```
    pub fn uri(&self) -> &http::Uri {
        match &self.inner_http {
            Some(req) => req.uri(),
            None => match &self.inner_quic {
                Some(req) => req.uri(),
                None => panic!("No request"),
            },
        }
    }

    /// Returns the request headers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Request;
    ///
    /// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
    ///     let content_type = request.headers().get("content-type");
    ///     let user_agent = request.headers().get("user-agent");
    ///     Ok(/* response */)
    /// }
    /// ```
    pub fn headers(&self) -> &http::HeaderMap {
        match &self.inner_http {
            Some(req) => req.headers(),
            None => match &self.inner_quic {
                Some(req) => req.headers(),
                None => panic!("No request"),
            },
        }
    }

    /// Returns the HTTP method.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Request;
    ///
    /// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
    ///     match request.method() {
    ///         &http::Method::GET => { /* handle GET */ }
    ///         &http::Method::POST => { /* handle POST */ }
    ///         _ => { /* handle other methods */ }
    ///     }
    ///     Ok(/* response */)
    /// }
    /// ```
    pub fn method(&self) -> &http::Method {
        match &self.inner_http {
            Some(req) => req.method(),
            None => match &self.inner_quic {
                Some(req) => req.method(),
                None => panic!("No request"),
            },
        }
    }

    pub fn into_http_parts(self) -> (http::request::Parts, hyper::body::Incoming) {
        match self.inner_http {
            Some(req) => {
                let (parts, body) = req.into_parts();
                (parts, body)
            }
            None => {
                panic!("No request");
            }
        }
    }

    pub fn into_quic_parts(self) -> (http::request::Parts, Full<Bytes>) {
        match self.inner_quic {
            Some(req) => {
                let (parts, body) = req.into_parts();
                (parts, body)
            }
            None => {
                panic!("No request");
            }
        }
    }
}

/// Builder for creating HTTP responses.
///
/// `ResponseBuilder` provides a fluent interface for constructing HTTP responses
/// with custom status codes, headers, and body content.
///
/// # Examples
///
/// ```rust,ignore
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use http::StatusCode;
/// use vetis::Response;
///
/// // Simple response
/// let response = Response::builder()
///     .status(StatusCode::OK)
///     .body(Full::new(Bytes::from("Hello, World!")));
///
/// // Response with custom headers
/// let mut headers = http::HeaderMap::new();
/// headers.insert("content-type", "application/json".parse().unwrap());
/// let response = Response::builder()
///     .status(StatusCode::CREATED)
///     .headers(headers)
///     .body(Full::new(Bytes::from(r#"{"status": "success"}"#)));
/// ```
pub struct ResponseBuilder {
    status: http::StatusCode,
    version: http::Version,
    headers: http::HeaderMap,
}

impl ResponseBuilder {
    /// Sets the HTTP status code for the response.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    /// use http::StatusCode;
    ///
    /// let response = Response::builder()
    ///     .status(StatusCode::NOT_FOUND)
    ///     .body(http_body_util::Full::new(bytes::Bytes::from("Not found")));
    /// ```
    pub fn status(mut self, status: http::StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Sets the HTTP version for the response.
    ///
    /// By default, responses use HTTP/1.1.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    /// use http::Version;
    ///
    /// let response = Response::builder()
    ///     .version(http::Version::HTTP_2)
    ///     .body(http_body_util::Full::new(bytes::Bytes::from("Response")));
    /// ```
    pub fn version(mut self, version: http::Version) -> Self {
        self.version = version;
        self
    }

    /// Sets the headers for the response.
    ///
    /// This replaces all existing headers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let mut headers = http::HeaderMap::new();
    /// headers.insert("content-type", "text/plain".parse().unwrap());
    ///
    /// let response = Response::builder()
    ///     .headers(headers)
    ///     .body(http_body_util::Full::new(bytes::Bytes::from("Plain text")));
    /// ```
    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Sets the body from a text string and creates the final `Response`.
    ///
    /// # Arguments
    ///
    /// * `text` - The response body as a text slice
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .text("Hello, World!");
    /// ```    
    pub fn text(self, text: &str) -> Response {
        self.body(Either::Right(Full::new(Bytes::from(
            text.as_bytes()
                .to_vec(),
        ))))
    }

    /// Sets the body and creates the final `Response`.
    ///
    /// # Arguments
    ///
    /// * `body` - The response body as a byte slice
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .body(b"Hello, World!");
    /// ```
    pub fn body(self, body: Either<Incoming, Full<Bytes>>) -> Response {
        let response = http::Response::builder()
            .status(self.status)
            .version(self.version);

        Response {
            inner: response
                .body(body)
                .unwrap(),
        }
    }
}

/// HTTP response containing status, headers, and body.
///
/// The `Response` struct represents an HTTP response that can be sent back to clients.
/// It's created using the `Response::builder()` method and contains the response body.
///
/// # Examples
///
/// ```rust,ignore
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use http::StatusCode;
/// use vetis::Response;
///
/// // Create a simple response
/// let response = Response::builder()
///     .status(StatusCode::OK)
///     .body(Full::new(Bytes::from("Hello, World!")));
///
/// // Convert to inner http::Response if needed
/// let inner_response = response.into_inner();
/// ```
pub struct Response {
    pub(crate) inner: http::Response<Either<Incoming, Full<Bytes>>>,
}

impl Response {
    /// Creates a new `ResponseBuilder` with default settings.
    ///
    /// The builder starts with:
    /// - Status: 200 OK
    /// - Version: HTTP/1.1
    /// - No headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let builder = Response::builder();
    /// let response = builder.body(http_body_util::Full::new(bytes::Bytes::from("Hello")));
    /// ```
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder {
            status: http::StatusCode::OK,
            version: http::Version::HTTP_11,
            headers: http::HeaderMap::new(),
        }
    }

    /// Converts the response into the underlying `http::Response`.
    ///
    /// This is useful when you need to work with the standard library HTTP types
    /// or pass the response to other libraries that expect `http::Response`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .body(http_body_util::Full::new(bytes::Bytes::from("Hello")));
    /// let inner = response.into_inner();
    /// ```
    pub fn into_inner(self) -> http::Response<Either<Incoming, Full<Bytes>>> {
        self.inner
    }
}
