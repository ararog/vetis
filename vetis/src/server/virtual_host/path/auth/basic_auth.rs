use argon2::{PasswordHash, PasswordVerifier};
use base64::Engine;
use http::HeaderMap;
use serde::Deserialize;

#[cfg(feature = "auth")]
use crate::config::server::virtual_host::path::auth::{Algorithm, BasicAuthConfig};

use crate::{
    errors::{VetisError, VirtualHostError},
    server::virtual_host::path::auth::Auth,
};

/// Basic authentication
#[derive(Clone, Deserialize)]
pub struct BasicAuth {
    config: BasicAuthConfig,
}

impl BasicAuth {
    /// Creates a new `BasicAuth` instance.
    ///
    /// # Arguments
    ///
    /// * `config` - A `BasicAuthConfig` instance containing the authentication configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - A new `BasicAuth` instance.
    pub fn new(config: BasicAuthConfig) -> Self {
        Self { config }
    }
}

impl Auth for BasicAuth {
    /// Authenticates the request using basic authentication on header field
    /// Authorization
    ///
    /// # Arguments
    ///
    /// * `headers` - A reference to a `HeaderMap` containing the request headers.
    ///
    /// # Returns
    ///
    /// * `Result<bool, VetisError>` - A result containing a boolean indicating whether the request is authenticated, or a `VetisError` if authentication fails.
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
            .config
            .users()
            .get(username)
        {
            return Ok(verify_password(
                password,
                hashed_password,
                self.config
                    .algorithm(),
            ));
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
