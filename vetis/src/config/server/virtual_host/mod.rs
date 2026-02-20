use std::{collections::HashMap, fs};

use serde::{Deserialize, Deserializer};

#[cfg(feature = "interface")]
use crate::config::server::virtual_host::path::interface::InterfacePathConfig;
#[cfg(feature = "reverse-proxy")]
use crate::config::server::virtual_host::path::proxy::ProxyPathConfig;
#[cfg(feature = "static-files")]
use crate::config::server::virtual_host::path::static_files::StaticPathConfig;

use crate::errors::{ConfigError, VetisError};

pub mod path;

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
    #[cfg(feature = "interface")]
    interface_paths: Option<Vec<InterfacePathConfig>>,
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
        match self.default_headers {
            None => {
                let vec = vec![(key.to_string(), value.to_string())];
                self.default_headers = Some(vec);
            }
            Some(ref mut headers) => {
                headers.push((key.to_string(), value.to_string()));
            }
        }
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
    /// Sets the reverse proxy paths for the virtual host.
    ///
    /// These reverse proxy paths will be used to serve custom error pages.
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

    #[cfg(feature = "interface")]
    /// Sets the interface paths for the virtual host.
    ///
    /// These interface paths will be used to serve custom error pages.
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
    pub fn interface_paths(mut self, interface_paths: Vec<InterfacePathConfig>) -> Self {
        self.interface_paths = Some(interface_paths);
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
            let root_path = std::path::Path::new(&self.root_directory);
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
            #[cfg(feature = "interface")]
            interface_paths: self.interface_paths,
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
    #[serde(deserialize_with = "deserialize_security_from_file")]
    security: Option<SecurityConfig>,
    status_pages: Option<HashMap<u16, String>>,
    enable_logging: bool,
    #[cfg(feature = "static-files")]
    static_paths: Option<Vec<StaticPathConfig>>,
    #[cfg(feature = "reverse-proxy")]
    proxy_paths: Option<Vec<ProxyPathConfig>>,
    #[cfg(feature = "interface")]
    interface_paths: Option<Vec<InterfacePathConfig>>,
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
            #[cfg(feature = "interface")]
            interface_paths: None,
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

    #[cfg(feature = "interface")]
    /// Returns the interface paths.
    ///
    /// # Returns
    ///
    /// * `&Option<Vec<InterfacePathConfig>>` - The interface paths.
    pub fn interface_paths(&self) -> &Option<Vec<InterfacePathConfig>> {
        &self.interface_paths
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
        let cert = fs::read(path);
        // TODO: Handle error properly
        if let Ok(cert) = cert {
            self.cert = cert;
        }
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
        let key = fs::read(path);
        // TODO: Handle error properly
        if let Ok(key) = key {
            self.key = key;
        }
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
        let ca_cert = fs::read(path);
        // TODO: Handle error properly
        if let Ok(ca_cert) = ca_cert {
            self.ca_cert = Some(ca_cert);
        }
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

#[derive(Clone, Deserialize)]
pub struct SecurityConfigFromFile {
    cert_from_file: String,
    key_from_file: String,
    ca_cert_from_file: Option<String>,
    client_auth: Option<bool>,
}

fn deserialize_security_from_file<'de, D>(
    deserializer: D,
) -> Result<Option<SecurityConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let security =
        SecurityConfigFromFile::deserialize(deserializer).map_err(serde::de::Error::custom)?;

    let mut builder = SecurityConfig::builder()
        .cert_from_file(&security.cert_from_file)
        .key_from_file(&security.key_from_file);

    if let Some(ca_cert_from_file) = security.ca_cert_from_file {
        builder = builder.ca_cert_from_file(&ca_cert_from_file);
    }

    if let Some(client_auth) = security.client_auth {
        builder = builder.client_auth(client_auth);
    }

    builder
        .build()
        .map_err(serde::de::Error::custom)
        .map(Some)
}
