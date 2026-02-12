use crate::{errors::VetisError, server::virtual_host::path::auth::basic_auth::BasicAuth};

use http::HeaderMap;

use serde::Deserialize;

pub mod basic_auth;

/// A trait for authentication methods.
pub trait Auth {
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

#[derive(Clone, Deserialize)]
/// An enum with authentication configuration.
pub enum AuthType {
    Basic(BasicAuth),
}

impl Auth for AuthType {
    fn authenticate(&self, headers: &HeaderMap) -> Result<bool, VetisError> {
        match self {
            AuthType::Basic(auth) => auth.authenticate(headers),
        }
    }
}
