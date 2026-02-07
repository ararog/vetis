//! Configuration builders and types for VeTiS server.
//!
//! This module provides a fluent builder API for configuring:
//! - Server listeners (ports, protocols, interfaces)
//! - Virtual hosts (hostnames, security settings)
//! - Security/TLS configuration (certificates, keys)
//!
//! # Examples
//!
//! ```rust,ignore
//! use vetis::config::{
//!     ListenerConfig, SecurityConfig, ServerConfig, VirtualHostConfig, Protocol
//! };
//!
//! // Configure a listener
//! let listener = ListenerConfig::builder()
//!     .port(8443)
//!     .protocol(Protocol::HTTP1)
//!     .interface("0.0.0.0")
//!     .build();
//!
//! // Configure server with multiple listeners
//! let config = ServerConfig::builder()
//!     .add_listener(listener)
//!     .build();
//!
//! // Configure security
//! let security = SecurityConfig::builder()
//!     .cert_from_bytes(include_bytes!("server.der").to_vec())
//!     .key_from_bytes(include_bytes!("server.key.der").to_vec())
//!     .build();
//!
//! // Configure virtual host
//! let vhost_config = VirtualHostConfig::builder()
//!     .hostname("example.com")
//!     .port(8443)
//!     .security(security)
//!     .build()?;
//! ```

use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::errors::{ConfigError, VetisError};

/// Supported HTTP protocols.
///
/// The protocol enum is feature-gated to only include protocols
/// that are enabled in the crate's feature flags.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::Protocol;
///
/// #[cfg(feature = "http1")]
/// let protocol = Protocol::Http1;
///
/// #[cfg(feature = "http2")]
/// let protocol = Protocol::Http2;
///
/// #[cfg(feature = "http3")]
/// let protocol = Protocol::Http3;
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum Protocol {
    #[cfg(feature = "http1")]
    /// HTTP/1.1 protocol
    Http1,
    #[cfg(feature = "http2")]
    /// HTTP/2 protocol (requires TLS)
    Http2,
    #[cfg(feature = "http3")]
    /// HTTP/3 protocol over QUIC (requires TLS)
    Http3,
}

/// Builder for creating `ListenerConfig` instances.
///
/// Provides a fluent API for configuring server listeners.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::{ListenerConfig, Protocol};
///
/// let config = ListenerConfig::builder()
///     .port(8080)
///     .protocol(Protocol::Http1)
///     .interface("127.0.0.1")
///     .build();
/// ```
#[derive(Clone)]
pub struct ListenerConfigBuilder {
    port: u16,
    ssl: bool,
    protocol: Protocol,
    interface: String,
}

impl ListenerConfigBuilder {
    /// Sets the port number for the listener.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::ListenerConfig;
    ///
    /// let config = ListenerConfig::builder()
    ///     .port(8443)
    ///     .build();
    /// ```
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets whether SSL/TLS is enabled for this listener.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::ListenerConfig;
    ///
    /// let config = ListenerConfig::builder()
    ///     .ssl(true)
    ///     .build();
    /// ```
    pub fn ssl(mut self, ssl: bool) -> Self {
        self.ssl = ssl;
        self
    }

    /// Sets the network interface to bind to.
    ///
    /// Common values:
    /// - "0.0.0.0" - All interfaces
    /// - "127.0.0.1" - Localhost only
    /// - "::1" - IPv6 localhost
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::ListenerConfig;
    ///
    /// let config = ListenerConfig::builder()
    ///     .interface("127.0.0.1")
    ///     .build();
    /// ```
    pub fn interface(mut self, interface: &str) -> Self {
        self.interface = interface.to_string();
        self
    }

    /// Sets the HTTP protocol for this listener.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::{ListenerConfig, Protocol};
    ///
    /// #[cfg(feature = "http1")]
    /// let config = ListenerConfig::builder()
    ///     .protocol(Protocol::HTTP1)
    ///     .build();
    /// ```
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Creates the `ListenerConfig` with the configured settings.
    pub fn build(self) -> ListenerConfig {
        ListenerConfig {
            port: self.port,
            ssl: self.ssl,
            protocol: self.protocol,
            interface: self.interface,
        }
    }
}

/// Configuration for a server listener.
///
/// Defines how the server should listen for incoming connections,
/// including the port, protocol, interface, and SSL settings.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::{ListenerConfig, Protocol};
///
/// let config = ListenerConfig::builder()
///     .port(8443)
///     .protocol(Protocol::Http1)
///     .interface("0.0.0.0")
///     .build();
///
/// println!("Listening on port {}", config.port());
/// ```
#[derive(Clone, Deserialize)]
pub struct ListenerConfig {
    port: u16,
    ssl: bool,
    protocol: Protocol,
    interface: String,
}

impl ListenerConfig {
    /// Creates a new `ListenerConfigBuilder` with default settings.
    ///
    /// Default values:
    /// - port: 80
    /// - ssl: false
    /// - protocol: HTTP1 (if available)
    /// - interface: "0.0.0.0"
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::ListenerConfig;
    ///
    /// let builder = ListenerConfig::builder();
    /// let config = builder.port(8080).build();
    /// ```
    pub fn builder() -> ListenerConfigBuilder {
        ListenerConfigBuilder {
            port: 80,
            ssl: false,
            #[cfg(feature = "http1")]
            protocol: Protocol::Http1,
            #[cfg(feature = "http2")]
            protocol: Protocol::Http2,
            #[cfg(feature = "http3")]
            protocol: Protocol::Http3,
            interface: "0.0.0.0".into(),
        }
    }

    /// Returns the port number.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns whether SSL/TLS is enabled.
    pub fn ssl(&self) -> bool {
        self.ssl
    }

    /// Returns the HTTP protocol.
    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }

    /// Returns the network interface.
    pub fn interface(&self) -> &str {
        &self.interface
    }
}

/// Builder for creating `ServerConfig` instances.
///
/// Provides a fluent API for configuring the overall server,
/// including multiple listeners for different ports and protocols.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::{ServerConfig, ListenerConfig, Protocol};
///
/// let http_listener = ListenerConfig::builder()
///     .port(80)
///     .protocol(Protocol::Http1)
///     .build();
///
/// let https_listener = ListenerConfig::builder()
///     .port(443)
///     .protocol(Protocol::Http1)
///     .ssl(true)
///     .build();
///
/// let config = ServerConfig::builder()
///     .add_listener(http_listener)
///     .add_listener(https_listener)
///     .build();
/// ```
#[derive(Clone)]
pub struct ServerConfigBuilder {
    listeners: Vec<ListenerConfig>,
}

impl ServerConfigBuilder {
    /// Adds a listener configuration to the server.
    ///
    /// Multiple listeners can be added to support different
    /// ports, protocols, or interfaces simultaneously.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::{ServerConfig, ListenerConfig};
    ///
    /// let listener = ListenerConfig::builder().port(8080).build();
    /// let config = ServerConfig::builder()
    ///     .add_listener(listener)
    ///     .build();
    /// ```
    pub fn add_listener(mut self, listener: ListenerConfig) -> Self {
        self.listeners
            .push(listener);
        self
    }

    /// Creates the `ServerConfig` with the configured listeners.
    pub fn build(self) -> ServerConfig {
        ServerConfig { listeners: self.listeners }
    }
}

/// Global server configuration.
///
/// Contains all the listeners that the server should use to accept
/// incoming connections. Each listener can have different settings
/// for port, protocol, and interface.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::{ServerConfig, ListenerConfig};
///
/// let config = ServerConfig::builder()
///     .add_listener(ListenerConfig::builder().port(80).build())
///     .add_listener(ListenerConfig::builder().port(443).ssl(true).build())
///     .build();
///
/// println!("Server has {} listeners", config.listeners().len());
/// ```
#[derive(Clone, Default, Deserialize)]
pub struct ServerConfig {
    listeners: Vec<ListenerConfig>,
}

impl ServerConfig {
    /// Creates a new `ServerConfigBuilder` with no listeners.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::{ServerConfig, ListenerConfig};
    ///
    /// let config = ServerConfig::builder()
    ///     .add_listener(ListenerConfig::builder().port(8080).build())
    ///     .build();
    /// ```
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder { listeners: vec![] }
    }

    /// Returns a reference to all configured listeners.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::{ServerConfig, ListenerConfig};
    ///
    /// let config = ServerConfig::builder()
    ///     .add_listener(ListenerConfig::builder().port(80).build())
    ///     .build();
    ///
    /// for listener in config.listeners() {
    ///     println!("Listening on port {}", listener.port());
    /// }
    /// ```
    pub fn listeners(&self) -> &Vec<ListenerConfig> {
        &self.listeners
    }
}

/// Builder for creating `VirtualHostConfig` instances.
///
/// Provides a fluent API for configuring virtual hosts,
/// including hostname, port, and security settings.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::{VirtualHostConfig, SecurityConfig};
///
/// let security = SecurityConfig::builder()
///     .cert_from_bytes(vec![])
///     .key_from_bytes(vec![])
///     .build();
///
/// let config = VirtualHostConfig::builder()
///     .hostname("example.com")
///     .port(443)
///     .security(security)
///     .build()?;
/// ```
pub struct VirtualHostConfigBuilder {
    hostname: String,
    port: u16,
    default_headers: Option<Vec<(String, String)>>,
    security: Option<SecurityConfig>,
    status_pages: Option<HashMap<u16, String>>,
    enable_logging: bool,
    #[cfg(feature = "static-files")]
    static_paths: Option<Vec<StaticPathConfig>>,
    #[cfg(feature = "reverse-proxy")]
    proxy_paths: Option<Vec<ProxyPathConfig>>,
}

impl VirtualHostConfigBuilder {
    /// Sets the hostname for the virtual host.
    ///
    /// This is used to match incoming requests to the correct virtual host.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .hostname("api.example.com")
    ///     .build()?;
    /// ```
    pub fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = hostname.to_string();
        self
    }

    /// Sets the port for the virtual host.
    ///
    /// This should match one of the ports configured in the server listeners.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .port(8443)
    ///     .build()?;
    /// ```
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Adds a default header to the virtual host.
    ///
    /// These headers will be added to all responses from this virtual host.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .header("X-Custom", "value")
    ///     .build()?;
    /// ```
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if self
            .default_headers
            .is_none()
        {
            self.default_headers = Some(Vec::new());
        }
        self.default_headers
            .as_mut()
            .unwrap()
            .push((key.to_string(), value.to_string()));
        self
    }

    /// Sets the security configuration for HTTPS.
    ///
    /// When provided, the virtual host will use TLS for secure connections.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::{VirtualHostConfig, SecurityConfig};
    ///
    /// let security = SecurityConfig::builder()
    ///     .cert_from_bytes(vec![])
    ///     .key_from_bytes(vec![])
    ///     .build();
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .security(security)
    ///     .build()?;
    /// ```
    pub fn security(mut self, security: SecurityConfig) -> Self {
        self.security = Some(security);
        self
    }

    /// Sets the status pages for the virtual host.
    ///
    /// These status pages will be used to serve custom error pages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .status_pages(vec![(404, "404.html".to_string())])
    ///     .build()?;
    /// ```
    pub fn status_pages(mut self, status_pages: HashMap<u16, String>) -> Self {
        self.status_pages = Some(status_pages);
        self
    }

    /// Enables or disables logging for this virtual host.
    ///
    /// When enabled, all requests to this virtual host will be logged.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .enable_logging(true)
    ///     .build()?;
    /// ```
    pub fn enable_logging(mut self, logging: bool) -> Self {
        self.enable_logging = logging;
        self
    }

    #[cfg(feature = "static-files")]
    /// Sets the status pages for the virtual host.
    ///
    /// These status pages will be used to serve custom error pages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .static_paths(vec![(404, "404.html".to_string())])
    ///     .build()?;
    /// ```
    pub fn static_paths(mut self, static_paths: Vec<StaticPathConfig>) -> Self {
        self.static_paths = Some(static_paths);
        self
    }

    #[cfg(feature = "reverse-proxy")]
    /// Sets the status pages for the virtual host.
    ///
    /// These status pages will be used to serve custom error pages.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .proxy_paths(vec![(404, "404.html".to_string())])
    ///     .build()?;
    /// ```
    pub fn proxy_paths(mut self, proxy_paths: Vec<ProxyPathConfig>) -> Self {
        self.proxy_paths = Some(proxy_paths);
        self
    }

    /// Creates the `VirtualHostConfig` with the configured settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the hostname is empty.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .hostname("example.com")
    ///     .port(443)
    ///     .header("X-Custom", "value")
    ///     .build()?;
    /// ```
    pub fn build(self) -> Result<VirtualHostConfig, VetisError> {
        if self
            .hostname
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::VirtualHost(
                "hostname is empty".to_string(),
            )));
        }

        Ok(VirtualHostConfig {
            hostname: self.hostname,
            port: self.port,
            default_headers: self.default_headers,
            security: self.security,
            status_pages: self.status_pages,
            enable_logging: self.enable_logging,
            #[cfg(feature = "static-files")]
            static_paths: self.static_paths,
            #[cfg(feature = "reverse-proxy")]
            proxy_paths: self.proxy_paths,
        })
    }
}

/// Configuration for a virtual host.
///
/// Defines how a specific hostname should be handled, including
/// the port it listens on and optional security settings for HTTPS.
///
/// Virtual hosts allow multiple domains to be served by the same
/// server instance, each with its own configuration and handlers.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::VirtualHostConfig;
///
/// let config = VirtualHostConfig::builder()
///     .hostname("api.example.com")
///     .port(443)
///     .build()?;
///
/// println!("Virtual host: {}:{}", config.hostname(), config.port());
/// ```
#[derive(Clone, Deserialize)]
pub struct VirtualHostConfig {
    hostname: String,
    port: u16,
    default_headers: Option<Vec<(String, String)>>,
    security: Option<SecurityConfig>,
    status_pages: Option<HashMap<u16, String>>,
    enable_logging: bool,
    #[cfg(feature = "static-files")]
    static_paths: Option<Vec<StaticPathConfig>>,
    #[cfg(feature = "reverse-proxy")]
    proxy_paths: Option<Vec<ProxyPathConfig>>,
}

impl VirtualHostConfig {
    /// Creates a new `VirtualHostConfigBuilder` with default settings.
    ///
    /// Default values:
    /// - hostname: empty string (must be set)
    /// - port: 80
    /// - security: None
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .hostname("example.com")
    ///     .port(443)
    ///     .build()?;
    /// ```
    pub fn builder() -> VirtualHostConfigBuilder {
        VirtualHostConfigBuilder {
            hostname: "localhost".to_string(),
            port: 80,
            default_headers: None,
            security: None,
            status_pages: None,
            enable_logging: true,
            #[cfg(feature = "static-files")]
            static_paths: None,
            #[cfg(feature = "reverse-proxy")]
            proxy_paths: None,
        }
    }

    /// Returns the hostname.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// Returns the port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the default headers.
    pub fn default_headers(&self) -> &Option<Vec<(String, String)>> {
        &self.default_headers
    }

    /// Returns the security configuration if present.
    pub fn security(&self) -> &Option<SecurityConfig> {
        &self.security
    }

    /// Returns the status pages.
    pub fn status_pages(&self) -> &Option<HashMap<u16, String>> {
        &self.status_pages
    }

    /// Returns the logging setting.
    pub fn enable_logging(&self) -> bool {
        self.enable_logging
    }

    #[cfg(feature = "static-files")]
    pub fn static_paths(&self) -> &Option<Vec<StaticPathConfig>> {
        &self.static_paths
    }

    #[cfg(feature = "reverse-proxy")]
    pub fn proxy_paths(&self) -> &Option<Vec<ProxyPathConfig>> {
        &self.proxy_paths
    }
}

#[cfg(feature = "static-files")]
pub struct StaticPathConfigBuilder {
    uri: String,
    extensions: String,
    directory: String,
    index_files: Option<Vec<String>>,
}

#[cfg(feature = "static-files")]
impl StaticPathConfigBuilder {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    pub fn extensions(mut self, extensions: &str) -> Self {
        self.extensions = extensions.to_string();
        self
    }

    pub fn directory(mut self, directory: &str) -> Self {
        self.directory = directory.to_string();
        self
    }

    pub fn index_files(mut self, index_files: Vec<String>) -> Self {
        self.index_files = Some(index_files);
        self
    }

    pub fn build(self) -> Result<StaticPathConfig, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::Config(ConfigError::Path("URI cannot be empty".to_string())));
        }
        if self
            .extensions
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path(
                "Extensions cannot be empty".to_string(),
            )));
        }
        if self
            .directory
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path(
                "Directory cannot be empty".to_string(),
            )));
        }

        Ok(StaticPathConfig {
            uri: self.uri,
            extensions: self.extensions,
            directory: self.directory,
            index_files: self.index_files,
        })
    }
}

#[cfg(feature = "static-files")]
#[derive(Clone, Deserialize)]
pub struct StaticPathConfig {
    uri: String,
    extensions: String,
    directory: String,
    index_files: Option<Vec<String>>,
    // TODO: Add basicauth config
}

#[cfg(feature = "static-files")]
impl StaticPathConfig {
    pub fn builder() -> StaticPathConfigBuilder {
        StaticPathConfigBuilder {
            uri: "/test".to_string(),
            extensions: ".html".to_string(),
            directory: "./test".to_string(),
            index_files: None,
        }
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn extensions(&self) -> &str {
        &self.extensions
    }

    pub fn directory(&self) -> &str {
        &self.directory
    }

    pub fn index_files(&self) -> &Option<Vec<String>> {
        &self.index_files
    }
}

#[cfg(feature = "reverse-proxy")]
#[derive(Deserialize)]
pub struct ProxyPathConfigBuilder {
    uri: String,
    target: String,
}

#[cfg(feature = "reverse-proxy")]
impl ProxyPathConfigBuilder {
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    pub fn target(mut self, target: &str) -> Self {
        self.target = target.to_string();
        self
    }

    pub fn build(self) -> Result<ProxyPathConfig, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::Config(ConfigError::Path("URI cannot be empty".to_string())));
        }
        if self
            .target
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path(
                "Target cannot be empty".to_string(),
            )));
        }

        Ok(ProxyPathConfig { uri: self.uri, target: self.target })
    }
}

#[cfg(feature = "reverse-proxy")]
#[derive(Clone, Deserialize)]
pub struct ProxyPathConfig {
    uri: String,
    target: String,
    // TODO: Add custom proxy rules

    // TODO: Add support for custom headers
}

#[cfg(feature = "reverse-proxy")]
impl ProxyPathConfig {
    pub fn builder() -> ProxyPathConfigBuilder {
        ProxyPathConfigBuilder {
            uri: "/test".to_string(),
            target: "http://localhost:8080".to_string(),
        }
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}

/// Builder for creating `SecurityConfig` instances.
///
/// Provides a fluent API for configuring TLS/SSL security settings,
/// including certificates, private keys, and client authentication.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::SecurityConfig;
///
/// let security = SecurityConfig::builder()
///     .cert_from_bytes(include_bytes!("server.der").to_vec())
///     .key_from_bytes(include_bytes!("server.key.der").to_vec())
///     .ca_cert_from_bytes(include_bytes!("ca.der").to_vec())
///     .client_auth(true)
///     .build();
/// ```
#[derive(Clone)]
pub struct SecurityConfigBuilder {
    cert: Vec<u8>,
    key: Vec<u8>,
    ca_cert: Option<Vec<u8>>,
    client_auth: bool,
}

impl SecurityConfigBuilder {
    /// Sets the server certificate from bytes.
    ///
    /// The certificate should be in DER format.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .cert_from_bytes(include_bytes!("server.der").to_vec())
    ///     .build();
    /// ```
    pub fn cert_from_bytes(mut self, cert: Vec<u8>) -> Self {
        self.cert = cert;
        self
    }

    /// Sets the server certificate from a file.
    ///
    /// Reads the certificate file in DER format.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .cert_from_file("/path/to/server.der")
    ///     .build();
    /// ```
    pub fn cert_from_file(mut self, path: &str) -> Self {
        self.cert = fs::read(path).unwrap();
        self
    }

    /// Sets the private key from bytes.
    ///
    /// The key should be in DER format.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .key_from_bytes(include_bytes!("server.key.der").to_vec())
    ///     .build();
    /// ```
    pub fn key_from_bytes(mut self, key: Vec<u8>) -> Self {
        self.key = key;
        self
    }

    /// Sets the private key from a file.
    ///
    /// Reads the key file in DER format.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .key_from_file("/path/to/server.key.der")
    ///     .build();
    /// ```
    pub fn key_from_file(mut self, path: &str) -> Self {
        self.key = fs::read(path).unwrap();
        self
    }

    /// Sets the CA certificate from bytes.
    ///
    /// The CA certificate is used for client authentication and should be in DER format.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .ca_cert_from_bytes(include_bytes!("ca.der").to_vec())
    ///     .build();
    /// ```
    pub fn ca_cert_from_bytes(mut self, ca_cert: Vec<u8>) -> Self {
        self.ca_cert = Some(ca_cert);
        self
    }

    /// Sets the CA certificate from a file.
    ///
    /// Reads the CA certificate file in DER format.
    ///
    /// # Panics
    ///
    /// Panics if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .ca_cert_from_file("/path/to/ca.der")
    ///     .build();
    /// ```
    pub fn ca_cert_from_file(mut self, path: &str) -> Self {
        self.ca_cert = Some(fs::read(path).unwrap());
        self
    }

    /// Sets whether client authentication is required.
    ///
    /// When enabled, clients must present a valid certificate signed by the CA.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .client_auth(true)
    ///     .build();
    /// ```
    pub fn client_auth(mut self, client_auth: bool) -> Self {
        self.client_auth = client_auth;
        self
    }

    /// Creates the `SecurityConfig` with the configured settings.
    pub fn build(self) -> SecurityConfig {
        SecurityConfig {
            cert: self.cert,
            key: self.key,
            ca_cert: self.ca_cert,
            client_auth: self.client_auth,
        }
    }
}

/// Security configuration for TLS/SSL.
///
/// Contains the certificates and keys needed to establish secure HTTPS connections.
/// This configuration is used by virtual hosts to enable TLS.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::config::SecurityConfig;
///
/// let security = SecurityConfig::builder()
///     .cert_from_bytes(include_bytes!("server.der").to_vec())
///     .key_from_bytes(include_bytes!("server.key.der").to_vec())
///     .build();
///
/// println!("Certificate length: {} bytes", security.cert().len());
/// ```
#[derive(Clone, Deserialize)]
pub struct SecurityConfig {
    cert: Vec<u8>,
    key: Vec<u8>,
    ca_cert: Option<Vec<u8>>,
    client_auth: bool,
}

impl SecurityConfig {
    /// Creates a new `SecurityConfigBuilder` with default settings.
    ///
    /// Default values:
    /// - cert: empty (must be set)
    /// - key: empty (must be set)
    /// - ca_cert: None
    /// - client_auth: false
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::SecurityConfig;
    ///
    /// let security = SecurityConfig::builder()
    ///     .cert_from_bytes(vec![])
    ///     .key_from_bytes(vec![])
    ///     .build();
    /// ```
    pub fn builder() -> SecurityConfigBuilder {
        SecurityConfigBuilder {
            cert: Vec::new(),
            key: Vec::new(),
            ca_cert: None,
            client_auth: false,
        }
    }

    /// Returns the server certificate bytes.
    pub fn cert(&self) -> &Vec<u8> {
        &self.cert
    }

    /// Returns the private key bytes.
    pub fn key(&self) -> &Vec<u8> {
        &self.key
    }

    /// Returns the CA certificate bytes if present.
    pub fn ca_cert(&self) -> &Option<Vec<u8>> {
        &self.ca_cert
    }

    /// Returns whether client authentication is enabled.
    pub fn client_auth(&self) -> bool {
        self.client_auth
    }
}
