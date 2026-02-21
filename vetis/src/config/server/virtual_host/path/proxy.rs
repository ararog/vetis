use serde::Deserialize;

use crate::errors::{ConfigError, VetisError};

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
