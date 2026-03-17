use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;
use hyper_body_utils::HttpBody;

use crate::{
    errors::VetisError,
    server::{
        http::{Request, Response},
        virtual_host::path::interface::{Interface, InterfaceWorker},
    },
};

pub mod callback;

impl From<RackWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: RackWorker) -> Self {
        Interface::Rack(value)
    }
}

pub struct RackWorker {
    directory: String,
    target: String,
}

impl RackWorker {
    pub fn new(directory: String, target: String) -> RackWorker {
        RackWorker { directory, target }
    }

    pub fn directory(&self) -> &String {
        &self.directory
    }

    pub fn target(&self) -> &String {
        &self.target
    }
}

impl InterfaceWorker for RackWorker {
    fn handle(
        &self,
        _request: Arc<Request>,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(HttpBody::from_text("Ok!")))
        })
    }
}
