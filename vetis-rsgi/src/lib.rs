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

pub struct RsgiWorker {
    directory: String,
    target: String,
}

impl RsgiWorker {
    pub fn new(directory: String, target: String) -> RsgiWorker {
        RsgiWorker { directory, target }
    }

    pub fn directory(&self) -> &String {
        &self.directory
    }

    pub fn target(&self) -> &String {
        &self.target
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
                .body(HttpBody::from_text("Ok!")))
        })
    }
}
