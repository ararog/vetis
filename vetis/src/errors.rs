//! Error handling types for VeTiS.
//!
//! This module defines the comprehensive error types used throughout
//! the VeTiS server, providing detailed error information for
//! configuration, runtime, and TLS-related issues.
//!
//! # Examples
//!
//! ```rust,ignore
//! use vetis::errors::{VetisError, ConfigError};
//!
//! match some_operation() {
//!     Ok(result) => println!("Success: {:?}", result),
//!     Err(VetisError::Config(ConfigError::VirtualHost(msg))) => {
//!         eprintln!("Virtual host configuration error: {}", msg);
//!     }
//!     Err(other) => eprintln!("Other error: {}", other),
//! }
//! ```

use thiserror::Error;

/// Main error type for VeTiS operations.
///
/// This enum encompasses all possible errors that can occur during
/// server configuration, startup, and operation. Each variant provides
/// specific context about what went wrong.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::errors::VetisError;
///
/// match result {
///     Err(VetisError::Config(config_err)) => {
///         println!("Configuration issue: {}", config_err);
///     }
///     Err(VetisError::Bind(addr)) => {
///         println!("Failed to bind to address: {}", addr);
///     }
///     Err(other) => println!("Error: {}", other),
///     Ok(_) => println!("Success!"),
/// }
/// ```
#[derive(Debug, Error, PartialEq)]
pub enum VetisError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Failed to bind to a network address
    #[error("Failed to bind to address: {0}")]
    Bind(String),

    /// Server startup errors
    #[error("Failed to start server: {0}")]
    Start(#[from] StartError),

    /// Server shutdown errors
    #[error("Failed to stop server: {0}")]
    Stop(String),

    /// Request handler errors
    #[error("Handler error: {0}")]
    Handler(String),

    /// TLS/SSL related errors
    #[error("Tls error: {0}")]
    Tls(String),

    /// No server instances are running
    #[error("No instances")]
    NoInstances,

    /// Virtual host related errors
    #[error("Virtual host error: {0}")]
    VirtualHost(#[from] VirtualHostError),
}

/// Configuration-related errors.
///
/// These errors occur during the parsing and validation of
/// server and virtual host configurations.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::errors::ConfigError;
///
/// match error {
///     ConfigError::VirtualHost(msg) => {
///         println!("Virtual host configuration failed: {}", msg);
///     }
/// }
/// ```
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ConfigError {
    /// Invalid virtual host configuration
    #[error("Invalid virtual host config: {0}")]
    VirtualHost(String),
}

/// Server startup errors.
///
/// These errors occur when the server fails to start properly,
/// typically due to TLS initialization issues.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::errors::StartError;
///
/// match error {
///     StartError::Tls(msg) => {
///         println!("TLS initialization failed: {}", msg);
///     }
/// }
/// ```
#[derive(Debug, Clone, Error, PartialEq)]
pub enum StartError {
    /// TLS/SSL initialization errors
    #[error("Tls initialization: {0}")]
    Tls(String),
}

/// Virtual host related errors.
///
/// These errors occur when working with virtual hosts,
/// such as missing handlers or configuration issues.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::errors::VirtualHostError;
///
/// match error {
///     VirtualHostError::NoVirtualHosts => {
///         println!("No virtual hosts have been configured");
///     }
/// }
/// ```
#[derive(Debug, Clone, Error, PartialEq)]
pub enum VirtualHostError {
    /// No virtual hosts have been added to the server
    #[error("No virtual hosts")]
    NoVirtualHosts,

    /// Invalid path configuration
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Proxy errors
    #[error("Proxy error: {0}")]
    Proxy(String),
}
