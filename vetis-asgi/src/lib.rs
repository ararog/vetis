use std::{future::Future, pin::Pin, sync::Arc};

use http::StatusCode;
use hyper_body_utils::HttpBody;

use vetis_core::{
    errors::VetisError,
    http::{Request, Response},
    interface::InterfaceWorker,
};

mod callback;
mod tests;

pub struct AsgiWorker {
    directory: String,
    target: String,
}

impl AsgiWorker {
    pub fn new(directory: String, target: String) -> AsgiWorker {
        AsgiWorker { directory, target }
    }

    pub fn directory(&self) -> &String {
        &self.directory
    }

    pub fn target(&self) -> &String {
        &self.target
    }
}

impl InterfaceWorker for AsgiWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        let mut response_body: Option<Vec<u8>> = None;

        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(HttpBody::from_bytes(&response_body.unwrap())))
        })
    }
}
