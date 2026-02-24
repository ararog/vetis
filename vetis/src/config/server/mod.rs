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

use serde::Deserialize;

use crate::errors::ConfigError;

pub mod virtual_host;

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
