use std::{clone, fs, future::Future, path::Path, pin::Pin, sync::Arc};

use http::StatusCode;
use log::error;
use ripht_php_sapi::{RiphtSapi, WebRequest};

use crate::{
    errors::{VetisError, VirtualHostError},
    server::virtual_host::path::interface::{Interface, InterfaceWorker},
    Request, Response, VetisBody, VetisBodyExt,
};

#[cfg(feature = "smol-rt")]
use smol::unblock as spawn_blocking;
#[cfg(feature = "tokio-rt")]
use tokio::task::spawn_blocking;

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
    php: Arc<RiphtSapi>,
    code: Arc<String>,
}

impl PhpWorker {
    pub fn new(directory: String, target: String) -> Result<PhpWorker, VetisError> {
        let directory = Path::new(&directory);
        let php = RiphtSapi::instance();
        let code = fs::read_to_string(directory.join(format!("{}.php", target)));
        let code = match code {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to read script from file: {}", e);
                return Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())));
            }
        };
        Ok(PhpWorker { php: Arc::new(php), code: Arc::new(code) })
    }
}

impl InterfaceWorker for PhpWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        let code = self.code.clone();
        let php = self.php.clone();
        let request = request.clone();
        Box::pin(async move {
            let result = spawn_blocking(move || {
                let mut php_request = match request.method() {
                    &http::Method::GET => {
                        WebRequest::get()
                    }
                };
                php_request
                    .with_uri(uri.as_ref())
                    .with_path_info(request.uri().path());

                //exec.with_body(request.body().clone());

                let exec = match php_request.build(code.as_ref()) {
                    Ok(exec) => exec,
                    Err(e) => {
                        error!("Failed to build request: {}", e);
                        return Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())));
                    }
                };
                match php.execute(exec)
                {
                    Ok(result) => {
                        let body = result.body();
                        let status = StatusCode::from_u16(result.status_code());
                        match status {
                            Ok(status) => Ok(Response::builder()
                                .status(status)
                                .body(VetisBody::body_from_bytes(&body))),
                            Err(e) => {
                                Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())))
                            }
                        }
                    }
                    Err(e) => Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string()))),
                }
            }).await;

            match result {
                Ok(result) => result,
                Err(e) => Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string()))),
            }
        })
    }
}
