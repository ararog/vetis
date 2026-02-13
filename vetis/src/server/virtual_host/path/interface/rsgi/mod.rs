use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;

use crate::{
    errors::VetisError,
    server::virtual_host::path::interface::{Interface, InterfaceWorker},
    Request, Response, VetisBody, VetisBodyExt,
};

impl From<RsgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: RsgiWorker) -> Self {
        Interface::Rsgi(value)
    }
}

pub struct RsgiWorker {}

impl RsgiWorker {
    pub fn new() -> RsgiWorker {
        RsgiWorker {}
    }
}

impl InterfaceWorker for RsgiWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(VetisBody::body_from_text("Ok!")))
        })
    }
}
