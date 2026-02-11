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

use std::fs;
use std::{collections::HashMap, path::Path};

use serde::Deserialize;

#[cfg(feature = "auth")]
use crate::config::auth::Auth;

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
#[non_exhaustive]
pub enum Protocol {
    /// HTTP/1.1 protocol
    Http1,
    /// HTTP/2 protocol (requires TLS)
    Http2,
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
    pub fn build(self) -> Result<ListenerConfig, ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::Listener("Port cannot be 0".to_string()));
        }

        if self
            .interface
            .is_empty()
        {
            return Err(ConfigError::Listener("Interface cannot be empty".to_string()));
        }

        Ok(ListenerConfig { port: self.port, protocol: self.protocol, interface: self.interface })
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
        ListenerConfigBuilder { port: 80, protocol: Protocol::Http1, interface: "0.0.0.0".into() }
    }

    /// Returns the port number.
    pub fn port(&self) -> u16 {
        self.port
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
    pub fn build(self) -> Result<ServerConfig, ConfigError> {
        if self
            .listeners
            .is_empty()
        {
            return Err(ConfigError::Server("No listeners configured".to_string()));
        }

        Ok(ServerConfig { listeners: self.listeners })
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
    root_directory: String,
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

    /// Sets the root directory for the virtual host.
    ///
    /// This is the base directory for all static file paths.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::config::VirtualHostConfig;
    ///
    /// let config = VirtualHostConfig::builder()
    ///     .root_directory("/var/www")
    ///     .build()?;
    /// ```
    pub fn root_directory(mut self, root_directory: &str) -> Self {
        self.root_directory = root_directory.to_string();
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
                "hostname is not provided".to_string(),
            )));
        }

        if self
            .root_directory
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::VirtualHost(
                "root_directory is not provided".to_string(),
            )));
        } else {
            let root_path = Path::new(&self.root_directory);
            if !root_path.exists() {
                return Err(VetisError::Config(ConfigError::VirtualHost(format!(
                    "root_directory does not exist: {}",
                    self.root_directory
                ))));
            }
        }

        Ok(VirtualHostConfig {
            hostname: self.hostname,
            port: self.port,
            root_directory: self.root_directory,
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
    root_directory: String,
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
            root_directory: "/var/vetis/www".to_string(),
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
    ///
    /// # Returns
    ///
    /// * `&str` - The hostname.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// Returns the port.
    ///
    /// # Returns
    ///
    /// * `u16` - The port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the root directory.
    ///
    /// # Returns
    ///
    /// * `&str` - The root directory.
    pub fn root_directory(&self) -> &str {
        &self.root_directory
    }

    /// Returns the default headers.
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<(String, String)>>` - The default headers.
    pub fn default_headers(&self) -> &Option<Vec<(String, String)>> {
        &self.default_headers
    }

    /// Returns the security configuration if present.
    ///
    /// # Returns
    ///
    /// * `&Option<SecurityConfig>` - The security configuration if present.
    pub fn security(&self) -> &Option<SecurityConfig> {
        &self.security
    }

    /// Returns the status pages.
    ///
    /// # Returns
    ///
    /// * `&Option<HashMap<u16, String>>` - The status pages.
    pub fn status_pages(&self) -> &Option<HashMap<u16, String>> {
        &self.status_pages
    }

    /// Returns the logging setting.
    ///
    /// # Returns
    ///
    /// * `bool` - The logging setting.
    pub fn enable_logging(&self) -> bool {
        self.enable_logging
    }

    #[cfg(feature = "static-files")]
    /// Returns the static paths.
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<StaticPathConfig>>` - The static paths.
    pub fn static_paths(&self) -> &Option<Vec<StaticPathConfig>> {
        &self.static_paths
    }

    #[cfg(feature = "reverse-proxy")]
    /// Returns the proxy paths.
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<ProxyPathConfig>>` - The proxy paths.
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
    #[cfg(feature = "auth")]
    auth: Option<Auth>,
}

#[cfg(feature = "static-files")]
impl StaticPathConfigBuilder {
    /// Allow set the URI of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    /// Allow set the URI of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Allow set the extensions of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn extensions(mut self, extensions: &str) -> Self {
        self.extensions = extensions.to_string();
        self
    }

    /// Allow set the directory of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn directory(mut self, directory: &str) -> Self {
        self.directory = directory.to_string();
        self
    }

    /// Allow set the index files of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn index_files(mut self, index_files: Vec<String>) -> Self {
        self.index_files = Some(index_files);
        self
    }

    #[cfg(feature = "auth")]
    /// Allow set the authentication of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Build the `StaticPathConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `Result<StaticPathConfig, VetisError>` - The `StaticPathConfig` with the configured settings.
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
            #[cfg(feature = "auth")]
            auth: self.auth,
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
    #[cfg(feature = "auth")]
    auth: Option<Auth>,
}

#[cfg(feature = "static-files")]
impl StaticPathConfig {
    /// Allow create a new `StaticPathConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `StaticPathConfigBuilder` - The builder.
    pub fn builder() -> StaticPathConfigBuilder {
        StaticPathConfigBuilder {
            uri: "/".to_string(),
            extensions: ".html".to_string(),
            directory: ".".to_string(),
            index_files: None,
            #[cfg(feature = "auth")]
            auth: None,
        }
    }

    /// Returns uri
    ///
    /// # Returns
    ///
    /// * `&str` - The uri.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns extensions
    ///
    /// # Returns
    ///
    /// * `&str` - The extensions.
    pub fn extensions(&self) -> &str {
        &self.extensions
    }

    /// Returns directory
    ///
    /// # Returns
    ///
    /// * `&str` - The directory.
    pub fn directory(&self) -> &str {
        &self.directory
    }

    /// Returns index_files
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<String>>` - The index_files.
    pub fn index_files(&self) -> &Option<Vec<String>> {
        &self.index_files
    }

    #[cfg(feature = "auth")]
    /// Returns auth
    ///
    /// # Returns
    ///
    /// * `&Option<Auth>` - The auth.
    pub fn auth(&self) -> &Option<Auth> {
        &self.auth
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
    /// Allow set the URI of the proxy path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Allow set the target of the proxy path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn target(mut self, target: &str) -> Self {
        self.target = target.to_string();
        self
    }

    /// Build the `ProxyPathConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `Result<ProxyPathConfig, VetisError>` - The `ProxyPathConfig` with the configured settings.
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
    /// Creates a new `ProxyPathConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `ProxyPathConfigBuilder` - The builder.
    pub fn builder() -> ProxyPathConfigBuilder {
        ProxyPathConfigBuilder {
            uri: "/test".to_string(),
            target: "http://localhost:8080".to_string(),
        }
    }

    /// Returns the URI of the proxy path.
    ///
    /// # Returns
    ///
    /// * `&str` - The URI of the proxy path.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns the target of the proxy path.
    ///
    /// # Returns
    ///
    /// * `&str` - The target of the proxy path.
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
    ///
    /// # Returns
    ///
    /// * `Result<SecurityConfig, VetisError>` - The `SecurityConfig` with the configured settings.
    pub fn build(self) -> Result<SecurityConfig, VetisError> {
        if self.cert.is_empty() {
            return Err(VetisError::Config(ConfigError::Security(
                "Certificate is empty".to_string(),
            )));
        }

        if self.key.is_empty() {
            return Err(VetisError::Config(ConfigError::Security("Key is empty".to_string())));
        }

        Ok(SecurityConfig {
            cert: self.cert,
            key: self.key,
            ca_cert: self.ca_cert,
            client_auth: self.client_auth,
        })
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
    ///
    /// # Returns
    ///
    /// * `&Vec<u8>` - The server certificate bytes.
    pub fn cert(&self) -> &Vec<u8> {
        &self.cert
    }

    /// Returns the private key bytes.
    ///
    /// # Returns
    ///
    /// * `&Vec<u8>` - The private key bytes.
    pub fn key(&self) -> &Vec<u8> {
        &self.key
    }

    /// Returns the CA certificate bytes if present.
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<u8>>` - The CA certificate bytes if present.
    pub fn ca_cert(&self) -> &Option<Vec<u8>> {
        &self.ca_cert
    }

    /// Returns whether client authentication is enabled.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether client authentication is enabled.
    pub fn client_auth(&self) -> bool {
        self.client_auth
    }
}

#[cfg(feature = "auth")]
/// A module with authentication configuration.
pub mod auth {
    use argon2::{PasswordHash, PasswordVerifier};
    use base64::Engine;
    use http::HeaderMap;
    use log::error;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    use crate::errors::{ConfigError, VetisError, VirtualHostError};

    #[derive(Clone, Deserialize)]
    /// An enum with authentication configuration.
    pub enum Auth {
        Basic(BasicAuthConfig),
    }

    impl AuthConfig for Auth {
        fn authenticate(&self, headers: &HeaderMap) -> Result<bool, VetisError> {
            match self {
                Auth::Basic(config) => config.authenticate(headers),
            }
        }
    }

    /// A trait for authentication configuration.
    pub trait AuthConfig {
        /// Authenticate method takes a reference to a `HeaderMap` and returns a `Result<bool, VetisError>`.
        ///
        /// # Arguments
        ///
        /// * `headers` - A reference to a `HeaderMap` containing the request headers.
        ///
        /// # Returns
        ///
        /// * `Result<bool, VetisError>` - A result containing a boolean indicating whether the authentication was successful, or a `VetisError` if the authentication failed.
        fn authenticate(&self, headers: &HeaderMap) -> Result<bool, VetisError>;
    }

    #[derive(Clone, Debug, Deserialize, PartialEq)]
    /// An enum with authentication algorithms.
    ///
    /// # Variants
    ///
    /// * `BCrypt` - The bcrypt algorithm.
    /// * `Argon2` - The argon2 algorithm.
    pub enum Algorithm {
        BCrypt,
        Argon2,
    }

    pub struct BasicAuthConfigBuilder {
        users: HashMap<String, String>,
        algorithm: Algorithm,
        htpasswd: Option<String>,
    }

    impl BasicAuthConfigBuilder {
        /// Allow manually set a hashmap of user and passowrd
        ///
        /// # Returns
        ///
        /// * `Self` - The builder.
        pub fn users(mut self, users: HashMap<String, String>) -> Self {
            self.users = users;
            self
        }

        /// Allow manually set the algorithm
        ///
        /// # Returns
        ///
        /// * `Self` - The builder.
        pub fn algorithm(mut self, algorithm: Algorithm) -> Self {
            self.algorithm = algorithm;
            self
        }

        /// Allow manually set the htpasswd file
        ///
        /// # Returns
        ///
        /// * `Self` - The builder.
        pub fn htpasswd(mut self, htpasswd: Option<String>) -> Self {
            self.htpasswd = htpasswd;
            self
        }

        /// Caches the users from the htpasswd file.
        ///
        /// # Note
        ///
        /// This will read the htpasswd file and cache the users in memory.
        /// You must call this method before building the config.
        ///
        /// # Returns
        ///
        /// * `Self` - The builder.
        pub fn cache_users(mut self) -> Self {
            if self
                .htpasswd
                .is_none()
            {
                return self;
            }

            if let Some(htpasswd) = &self.htpasswd {
                let htpasswd = fs::read_to_string(htpasswd);
                match htpasswd {
                    Ok(file) => {
                        file.lines()
                            .for_each(|line| {
                                let (username, password) = line
                                    .split_once(':')
                                    .unwrap();
                                self.users
                                    .insert(username.to_string(), password.to_string());
                            });
                    }
                    Err(e) => {
                        error!("Failed to read htpasswd file: {}", e);
                    }
                }
            }

            self
        }

        /// Build the `BasicAuthConfig` with the configured settings.
        ///
        /// # Returns
        ///
        /// * `Result<BasicAuthConfig, VetisError>` - The `BasicAuthConfig` with the configured settings.
        pub fn build(self) -> Result<BasicAuthConfig, VetisError> {
            if let Some(htpasswd) = &self.htpasswd {
                let htpasswd = Path::new(htpasswd);
                if !htpasswd.exists() {
                    return Err(VetisError::Config(ConfigError::Auth(
                        ".htpasswd file not found".to_string(),
                    )));
                }
            }

            Ok(BasicAuthConfig {
                users: self.users,
                algorithm: self.algorithm,
                htpasswd: self.htpasswd,
            })
        }
    }

    #[derive(Clone, Deserialize)]
    /// A struct with basic authentication configuration.
    ///
    /// # Fields
    ///
    /// * `users` - A map of username to hashed password.
    /// * `algorithm` - The algorithm used for password hashing.
    /// * `htpasswd` - The path to the htpasswd file.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let auth = BasicAuthConfig::builder()
    ///     .users(HashMap::new())
    ///     .algorithm(Algorithm::BCrypt)
    ///     .htpasswd(None)
    ///     .build();
    /// ```
    pub struct BasicAuthConfig {
        users: HashMap<String, String>,
        algorithm: Algorithm,
        htpasswd: Option<String>,
    }

    impl BasicAuthConfig {
        /// Creates a new `BasicAuthConfigBuilder` with default settings.
        ///
        /// # Returns
        ///
        /// * `BasicAuthConfigBuilder` - The builder.
        pub fn builder() -> BasicAuthConfigBuilder {
            BasicAuthConfigBuilder {
                users: HashMap::new(),
                algorithm: Algorithm::BCrypt,
                htpasswd: None,
            }
        }

        /// Returns the algorithm used for password hashing.
        ///
        /// # Returns
        ///
        /// * `&Algorithm` - The algorithm used for password hashing.
        pub fn algorithm(&self) -> &Algorithm {
            &self.algorithm
        }

        /// Returns the path to the htpasswd file.
        ///
        /// # Returns
        ///
        /// * `&Option<String>` - The path to the htpasswd file.
        pub fn htpasswd(&self) -> &Option<String> {
            &self.htpasswd
        }
    }

    impl AuthConfig for BasicAuthConfig {
        fn authenticate(&self, headers: &HeaderMap) -> Result<bool, VetisError> {
            let auth_header = headers
                .get(http::header::AUTHORIZATION)
                .ok_or(VetisError::VirtualHost(VirtualHostError::Auth(
                    "Missing Authorization header".to_string(),
                )))?;

            let auth_header = auth_header
                .to_str()
                .map_err(|_| {
                    VetisError::VirtualHost(VirtualHostError::Auth(
                        "Invalid Authorization header".to_string(),
                    ))
                })?
                .strip_prefix("Basic ")
                .ok_or(VetisError::VirtualHost(VirtualHostError::Auth(
                    "Expected basic authentication".to_string(),
                )))?;

            let auth_header = base64::engine::general_purpose::STANDARD.decode(auth_header);
            let auth_header = auth_header.map_err(|e| {
                VetisError::VirtualHost(VirtualHostError::Auth(format!(
                    "Could not decode header: {}",
                    e
                )))
            })?;

            let auth_header = String::from_utf8(auth_header).map_err(|_| {
                VetisError::VirtualHost(VirtualHostError::Auth(
                    "Invalid Authorization header".to_string(),
                ))
            })?;

            let (username, password) = auth_header
                .split_once(':')
                .ok_or(VetisError::VirtualHost(VirtualHostError::Auth(
                    "Invalid credentials".to_string(),
                )))?;

            if let Some(hashed_password) = self
                .users
                .get(username)
            {
                return Ok(verify_password(password, hashed_password, &self.algorithm));
            }

            Ok(false)
        }
    }

    fn verify_password(password: &str, hashed_password: &str, algorithm: &Algorithm) -> bool {
        match algorithm {
            Algorithm::BCrypt => bcrypt::verify(password, hashed_password).unwrap_or(false),
            Algorithm::Argon2 => {
                let argon2 = argon2::Argon2::default();
                let parsed_hash = PasswordHash::new(hashed_password).unwrap();
                let result = argon2.verify_password(password.as_bytes(), &parsed_hash);
                result.is_ok()
            }
        }
    }
}
