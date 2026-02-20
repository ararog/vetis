use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;

use crate::{
    errors::VetisError,
    server::virtual_host::path::interface::{Interface, InterfaceWorker},
    Request, Response, VetisBody, VetisBodyExt,
};

pub mod callback;

impl From<RsgiRubyWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: RsgiRubyWorker) -> Self {
        Interface::RsgiRuby(value)
    }
}

pub struct RsgiRubyWorker {
    file: String,
}

impl RsgiRubyWorker {
    pub fn new(file: String) -> RsgiRubyWorker {
        RsgiRubyWorker { file }
    }
}

impl InterfaceWorker for RsgiRubyWorker {
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
