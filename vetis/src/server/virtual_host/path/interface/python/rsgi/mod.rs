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

pub struct RsgiWorker {
    directory: String,
    target: String,
}

impl RsgiWorker {
    pub fn new(directory: String, target: String) -> RsgiWorker {
        RsgiWorker { directory, target }
    }
}

impl InterfaceWorker for RsgiWorker {
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
