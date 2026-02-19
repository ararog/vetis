use std::{ffi::CString, fs, future::Future, pin::Pin, sync::Arc};

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use log::error;
use pyo3::{
    types::{PyAnyMethods, PyDict, PyIterator, PyModule, PyModuleMethods},
    Py, PyAny, PyErr, PyResult, Python,
};
use tokio::{sync::oneshot, task::spawn_blocking};

pub mod callback;

use crate::{
    errors::{VetisError, VirtualHostError},
    server::virtual_host::path::interface::{
        python::wsgi::callback::StartResponse, Interface, InterfaceWorker,
    },
    Request, Response, VetisBody, VetisBodyExt,
};

impl From<WsgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: WsgiWorker) -> Self {
        Interface::Wsgi(value)
    }
}

pub struct WsgiWorker {
    func: Arc<Py<PyAny>>,
}

impl WsgiWorker {
    pub fn new(file: String) -> Result<WsgiWorker, VetisError> {
        let code = fs::read_to_string(&file);
        let code = match code {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to read script from file: {}", e);
                return Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())));
            }
        };

        let code = CString::new(code);
        let code = match code {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to initialize script: {}", e);
                return Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())));
            }
        };

        let file = CString::new(file.as_str());
        let file = match file {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to initialize file: {}", e);
                return Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())));
            }
        };

        let app = Python::attach(|py| {
            let script_module = PyModule::from_code(py, &code, &file, c"main")?;
            let app = script_module.getattr("app")?;
            script_module.add_class::<StartResponse>()?;
            Ok::<Py<PyAny>, PyErr>(app.unbind())
        });

        Ok(WsgiWorker { func: Arc::new(app.unwrap()) })
    }
}

impl InterfaceWorker for WsgiWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        let (tx, rx) = oneshot::channel::<(CString, Vec<(CString, CString)>)>();
        let request = request.clone();
        let func = self.func.clone();

        Box::pin(async move {
            let python_task = spawn_blocking(move || {
                let content_type = match request
                    .headers()
                    .get(http::header::CONTENT_TYPE)
                {
                    Some(content_type) => content_type
                        .to_str()
                        .unwrap_or_default(),
                    None => "application/json",
                };

                let content_length = match request
                    .headers()
                    .get(http::header::CONTENT_LENGTH)
                {
                    Some(content_length) => content_length
                        .to_str()
                        .unwrap_or_default(),
                    None => "0",
                };

                let callback = StartResponse::new(Some(tx));

                Python::attach(|py| {
                    let func = func.bind(py);
                    let environ = PyDict::new(py);
                    environ.set_item("wsgi.url_scheme", "https")?;
                    environ.set_item("wsgi.version", [1, 0])?;
                    environ.set_item("wsgi.input", "")?;
                    environ.set_item("wsgi.errors", "")?;
                    environ.set_item("wsgi.multithread", "false")?;
                    environ.set_item("wsgi.multiprocess", "false")?;
                    environ.set_item("wsgi.run_once", "false")?;
                    environ.set_item(
                        "REQUEST_METHOD",
                        request
                            .method()
                            .as_str(),
                    )?;
                    environ.set_item(
                        "QUERY_STRING",
                        request
                            .uri()
                            .query()
                            .unwrap_or_default(),
                    )?;
                    environ.set_item("PATH_INFO", request.uri().path())?;
                    environ.set_item("CONTENT_TYPE", content_type)?;
                    environ.set_item("CONTENT_LENGTH", content_length)?;
                    environ.set_item("SERVER_NAME", "localhost")?;
                    environ.set_item("SERVER_PORT", "8080")?;
                    environ.set_item("SERVER_PROTOCOL", "HTTP/1.1")?;
                    environ.set_item("SERVER_SOFTWARE", "Vetis")?;

                    let response_body = func.call1((environ, callback))?;

                    let iter = response_body.cast::<PyIterator>()?;
                    let bytes = iter
                        .clone()
                        .map(|item| item?.extract::<Vec<u8>>())
                        .collect::<PyResult<Vec<Vec<u8>>>>()?;

                    Ok::<Vec<u8>, PyErr>(bytes[0].clone())
                })
            })
            .await;

            let response_body = match python_task {
                Ok(body) => body,
                Err(e) => {
                    error!("Failed to run script: {}", e);
                    return Err(VetisError::VirtualHost(VirtualHostError::Interface(
                        e.to_string(),
                    )));
                }
            };

            let channel_result = rx.await;
            let (status, headers) = match channel_result {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to run script: {}", e);
                    return Err(VetisError::VirtualHost(VirtualHostError::Interface(
                        e.to_string(),
                    )));
                }
            };

            let binding = status
                .into_string()
                .unwrap();
            let status_str = binding
                .split_whitespace()
                .next()
                .unwrap();
            let status_code = status_str
                .parse::<StatusCode>()
                .unwrap();

            // Need performance improvement, maybe specialize?
            let headers = headers
                .into_iter()
                .fold(HeaderMap::new(), |mut map, (key, value)| {
                    map.insert(
                        HeaderName::from_bytes(key.as_bytes()).unwrap(),
                        HeaderValue::from_bytes(value.as_bytes()).unwrap(),
                    );
                    map
                });

            match response_body {
                Ok(body) => Ok(Response::builder()
                    .status(status_code)
                    .headers(headers)
                    .body(VetisBody::body_from_bytes(&body))),
                Err(e) => {
                    error!("Failed to run script: {}", e);
                    Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())))
                }
            }
        })
    }
}
