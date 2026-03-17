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

impl From<ScgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: ScgiWorker) -> Self {
        Interface::Scgi(value)
    }
}

pub struct ScgiWorker {
    directory: String,
    target: String,
}

impl ScgiWorker {
    pub fn new(directory: String, target: String) -> ScgiWorker {
        ScgiWorker { directory, target }
    }

    pub fn directory(&self) -> &String {
        &self.directory
    }

    pub fn target(&self) -> &String {
        &self.target
    }
}

impl InterfaceWorker for ScgiWorker {
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
