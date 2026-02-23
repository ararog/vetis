use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;

use crate::{
    errors::VetisError,
    server::{
        http::{Request, Response, VetisBody, VetisBodyExt},
        virtual_host::path::interface::{Interface, InterfaceWorker},
    },
};

pub mod callback;

impl From<RubyWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: RubyWorker) -> Self {
        Interface::Ruby(value)
    }
}

pub struct RubyWorker {
    directory: String,
    target: String,
}

impl RubyWorker {
    pub fn new(directory: String, target: String) -> RubyWorker {
        RubyWorker { directory, target }
    }
}

impl InterfaceWorker for RubyWorker {
    fn handle(
        &self,
        _request: Arc<Request>,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(VetisBody::body_from_text("Ok!")))
        })
    }
}
