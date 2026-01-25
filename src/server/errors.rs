use thiserror::Error;

#[derive(Debug, Error)]
pub enum VetisError {
    #[error("Failed to bind to address: {0}")]
    Bind(String),

    #[error("Failed to start server: {0}")]
    Start(#[from] StartError),

    #[error("Failed to stop server: {0}")]
    Stop(String),

    #[error("Handler error: {0}")]
    Handler(String),

    #[error("No instances")]
    NoInstances,

    #[error("No virtual hosts")]
    NoVirtualHosts,
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum StartError {
    #[error("Tls initialization: {0}")]
    Tls(String),
}
