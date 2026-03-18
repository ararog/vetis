use std::{collections::HashMap, fs, future::Future, path::Path, pin::Pin, sync::Arc};

use http::StatusCode;
use hyper_body_utils::HttpBody;
use log::error;
use vetis_core::{
    errors::{VetisError, VirtualHostError},
    http::{Request, Response},
    interface::InterfaceWorker,
};

#[cfg(feature = "smol-rt")]
use smol::unblock as spawn_blocking;
#[cfg(feature = "tokio-rt")]
use tokio::task::spawn_blocking;

pub struct FcgiWorker {
    params: Arc<HashMap<String, String>>,
    script: Arc<String>,
}

impl FcgiWorker {
    pub fn new(directory: String, target: String) -> Result<FcgiWorker, VetisError> {
        let directory = Path::new(&directory);
        let params = HashMap::new();
        let code = fs::read_to_string(directory.join(format!("{}.php", target)));
        let code = match code {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to read script from file: {}", e);
                return Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())));
            }
        };
        Ok(FcgiWorker { params: Arc::new(params), script: Arc::new(code) })
    }
}

impl InterfaceWorker for FcgiWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        let script = self.script.clone();
        let params = self.params.clone();
        let request = request.clone();
        Box::pin(async move {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(HttpBody::from_bytes(&[])))
        })
    }
}
