use serde::Deserialize;

use crate::errors::{ConfigError, VetisError};

use std::collections::HashMap;

#[derive(Clone, Deserialize)]
#[non_exhaustive]
pub enum InterfaceType {
    Php,
    Asgi,
    Wsgi,
    Rsgi,
}

/// Builder for creating `InterfacePathConfig` instances.
pub struct InterfacePathConfigBuilder {
    uri: String,
    target: String,
    params: Option<HashMap<String, String>>,
    interface_type: InterfaceType,
}

impl InterfacePathConfigBuilder {
    /// Allow set the URI of the static path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Allow set the target of the interface path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn target(mut self, target: &str) -> Self {
        self.target = target.to_string();
        self
    }

    /// Allow set the params of the interface path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn params(mut self, params: HashMap<String, String>) -> Self {
        self.params = Some(params);
        self
    }

    /// Allow set the interface type of the interface path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn interface_type(mut self, interface_type: InterfaceType) -> Self {
        self.interface_type = interface_type;
        self
    }

    /// Build the `InterfacePathConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `Result<InterfacePathConfig, VetisError>` - The `InterfacePathConfig` with the configured settings.
    pub fn build(self) -> Result<InterfacePathConfig, VetisError> {
        if self.uri.is_empty() {
            return Err(VetisError::Config(ConfigError::Path("URI cannot be empty".to_string())));
        }

        Ok(InterfacePathConfig {
            uri: self.uri,
            target: self.target,
            params: self.params,
            interface_type: self.interface_type,
        })
    }
}

/// Interface path configuration.
#[derive(Clone, Deserialize)]
pub struct InterfacePathConfig {
    uri: String,
    target: String,
    params: Option<HashMap<String, String>>,
    interface_type: InterfaceType,
}

impl InterfacePathConfig {
    /// Allow create a new `InterfacePathConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `InterfacePathConfigBuilder` - The builder.
    pub fn builder() -> InterfacePathConfigBuilder {
        InterfacePathConfigBuilder {
            uri: "/".to_string(),
            target: "main".to_string(),
            params: None,
            interface_type: InterfaceType::Wsgi,
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

    /// Returns target
    ///
    /// # Returns
    ///
    /// * `&str` - The target.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns params
    ///
    /// # Returns
    ///
    /// * `&Option<HashMap<String, String>>` - The params.
    pub fn params(&self) -> &Option<HashMap<String, String>> {
        &self.params
    }

    /// Returns interface type
    ///
    /// # Returns
    ///
    /// * `&InterfaceType` - The interface type.
    pub fn interface_type(&self) -> &InterfaceType {
        &self.interface_type
    }
}
