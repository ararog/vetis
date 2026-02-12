use serde::Deserialize;

use crate::errors::{ConfigError, VetisError};
#[cfg(feature = "auth")]
use crate::server::virtual_host::path::auth::AuthType;

pub struct StaticPathConfigBuilder {
    uri: String,
    extensions: String,
    directory: String,
    index_files: Option<Vec<String>>,
    #[cfg(feature = "auth")]
    auth: Option<AuthType>,
}

impl StaticPathConfigBuilder {
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
    pub fn auth(mut self, auth: AuthType) -> Self {
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
    auth: Option<AuthType>,
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
    pub fn auth(&self) -> &Option<AuthType> {
        &self.auth
    }
}
