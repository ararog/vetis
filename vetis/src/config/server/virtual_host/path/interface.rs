use serde::Deserialize;

use crate::errors::{ConfigError, VetisError};

use std::{collections::HashMap, path::Path};

#[derive(Clone, Deserialize)]
#[non_exhaustive]
pub enum InterfaceType {
    Php,
    Asgi,
    Wsgi,
    Rsgi,
    Ruby,
}

/// Builder for creating `InterfacePathConfig` instances.
pub struct InterfacePathConfigBuilder {
    uri: String,
    directory: String,
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

    /// Allow set the directory of the interface path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn directory(mut self, directory: &str) -> Self {
        self.directory = directory.to_string();
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

        if self
            .directory
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path("Missing directory".to_string())));
        } else {
            let path = Path::new(&self.directory);
            if !path.exists() {
                return Err(VetisError::Config(ConfigError::Path(
                    "Directory does not exist".to_string(),
                )));
            }
        }

        if self
            .target
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path("Missing target".to_string())));
        } else {
            match self.interface_type {
                InterfaceType::Asgi | InterfaceType::Wsgi | InterfaceType::Rsgi => {
                    let target_parts = self
                        .target
                        .split_once(":");
                    match target_parts {
                        Some((module, _application)) => {
                            let path = Path::new(&self.directory);
                            let file = path.join(format!("{}.py", module));
                            if !file.exists() {
                                return Err(VetisError::Config(ConfigError::Path(
                                    "Module file does not exist".to_string(),
                                )));
                            }
                        }
                        None => {
                            return Err(VetisError::Config(ConfigError::Path("Target must be in format 'module:application' for API interface type".to_string())));
                        }
                    }
                }
                InterfaceType::Php => {
                    // For PHP interface type, target is not used
                }
                InterfaceType::Ruby => {
                    // For PHP interface type, target is not used
                }
            }
        }

        Ok(InterfacePathConfig {
            uri: self.uri,
            directory: self.directory,
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
    directory: String,
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
            directory: ".".to_string(),
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

    /// Returns directory
    ///
    /// # Returns
    ///
    /// * `&str` - The directory.
    pub fn directory(&self) -> &str {
        &self.directory
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
