#[cfg(feature = "auth")]
use std::collections::HashMap;

use std::path::Path;

use serde::Deserialize;

use crate::errors::{ConfigError, VetisError};

#[cfg(feature = "auth")]
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

#[cfg(feature = "auth")]
pub struct BasicAuthConfigBuilder {
    users: HashMap<String, String>,
    algorithm: Algorithm,
    htpasswd: Option<String>,
}

#[cfg(feature = "auth")]
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
            let htpasswd = Path::new(htpasswd);
            if !htpasswd.exists() {
                return self;
            }

            let htpasswd = std::fs::read_to_string(htpasswd);
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
                    use log::error;

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

#[cfg(feature = "auth")]
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

#[cfg(feature = "auth")]
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

    /// Returns users
    ///
    /// # Returns
    ///
    /// * `&HashMap<String, String>` - The users.
    pub fn users(&self) -> &HashMap<String, String> {
        &self.users
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
