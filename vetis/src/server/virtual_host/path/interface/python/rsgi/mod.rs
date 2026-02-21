use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;

use crate::{
    errors::VetisError,
    server::virtual_host::path::interface::{Interface, InterfaceWorker},
    Request, Response, VetisBody, VetisBodyExt,
};

pub mod callback;

impl From<RsgiPythonWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: RsgiPythonWorker) -> Self {
        Interface::RsgiPython(value)
    }
}

pub struct RsgiPythonWorker {
    file: String,
}

impl RsgiPythonWorker {
    pub fn new(file: String) -> RsgiPythonWorker {
        RsgiPythonWorker { file }
    }
}

impl InterfaceWorker for RsgiPythonWorker {
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
