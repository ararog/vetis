use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;

use crate::{
    errors::VetisError,
    server::virtual_host::path::interface::{Interface, InterfaceWorker},
    Request, Response, VetisBody, VetisBodyExt,
};

impl From<PhpWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: PhpWorker) -> Self {
        Interface::Php(value)
    }
}

pub struct PhpWorker {
    file: String,
}

impl PhpWorker {
    pub fn new(file: String) -> PhpWorker {
        PhpWorker { file }
    }
}

impl InterfaceWorker for PhpWorker {
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
