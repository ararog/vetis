use std::{ffi::CString, fs, future::Future, path::Path, pin::Pin, sync::Arc};

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use hyper_body_utils::HttpBody;
use log::error;
use pyo3::{
    intern,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyIterator, PyModule, PyModuleMethods},
    Py, PyAny, PyErr, PyResult, Python,
};

#[cfg(feature = "smol-rt")]
use smol::unblock as spawn_blocking;
#[cfg(feature = "tokio-rt")]
use tokio::task::spawn_blocking;

use crossfire::oneshot;

pub mod callback;

use crate::{
    errors::{VetisError, VirtualHostError},
    server::{
        http::{Request, Response},
        virtual_host::path::interface::{
            python::wsgi::callback::StartResponse, Interface, InterfaceWorker,
        },
    },
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
    env: Arc<Py<PyDict>>,
}

impl WsgiWorker {
    pub fn new(directory: String, target: String) -> Result<WsgiWorker, VetisError> {
        let directory = Path::new(&directory);
        let target = target.split_once(":");
        let (module, app) = target.unwrap();

        let code = fs::read_to_string(directory.join(format!("{}.py", module)));
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

        let app = Python::attach(|py| {
            let script_module = PyModule::from_code(py, &code, c"", c"main")?;
            let app = script_module.getattr(app)?;
            script_module.add_class::<StartResponse>()?;

            let environ = PyDict::new(py);
            environ.set_item(intern!(py, "wsgi.version"), [1, 0])?;
            environ.set_item(intern!(py, "wsgi.multithread"), "false")?;
            environ.set_item(intern!(py, "wsgi.multiprocess"), "false")?;
            environ.set_item(intern!(py, "wsgi.run_once"), "false")?;
            environ.set_item(intern!(py, "SERVER_NAME"), "localhost")?;
            environ.set_item(intern!(py, "SERVER_PORT"), "8080")?;
            environ.set_item(intern!(py, "SERVER_PROTOCOL"), "HTTP/1.1")?;
            environ.set_item(intern!(py, "SERVER_SOFTWARE"), "Vetis")?;
            Ok::<(Py<PyAny>, Py<PyDict>), PyErr>((app.unbind(), environ.unbind()))
        });

        let (func, env) = app.unwrap();
        Ok(WsgiWorker { func: Arc::new(func), env: Arc::new(env) })
    }
}

impl InterfaceWorker for WsgiWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        let (tx, rx) = oneshot::oneshot::<(String, Vec<(String, String)>)>();
        let request = request.clone();
        let func = self.func.clone();
        let env = self.env.clone();

        Box::pin(async move {
            let response_body = spawn_blocking(move || {
                let path = request.uri().path();

                let method = request
                    .method()
                    .as_str();

                let query_string = request
                    .uri()
                    .query()
                    .unwrap_or_default();

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
                    let environ = env.bind(py);
                    environ.set_item(intern!(py, "wsgi.url_scheme"), "https")?;
                    environ.set_item(intern!(py, "wsgi.input"), "")?;
                    environ.set_item(intern!(py, "wsgi.errors"), "")?;
                    environ.set_item(intern!(py, "REQUEST_METHOD"), method)?;
                    environ.set_item(intern!(py, "QUERY_STRING"), query_string)?;
                    environ.set_item(intern!(py, "PATH_INFO"), path)?;
                    environ.set_item(intern!(py, "CONTENT_TYPE"), content_type)?;
                    environ.set_item(intern!(py, "CONTENT_LENGTH"), content_length)?;
                    let response_body = func.call1((environ, callback))?;
                    let iter = response_body
                        .cast::<PyIterator>()?
                        .into_iter();
                    let bytes = iter
                        .map(|item| item?.extract::<Vec<u8>>())
                        .collect::<PyResult<Vec<Vec<u8>>>>()?;
                    Ok::<Vec<u8>, PyErr>(
                        bytes
                            .first()
                            .cloned()
                            .unwrap_or_default(),
                    )
                })
            })
            .await;

            #[cfg(feature = "tokio-rt")]
            let response_body = match response_body {
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

            let status_str = match status
                .split_whitespace()
                .next()
            {
                Some(str) => str,
                None => {
                    return Err(VetisError::VirtualHost(VirtualHostError::Interface(
                        "Invalid status message".to_string(),
                    )))
                }
            };

            let status_code = match status_str.parse::<StatusCode>() {
                Ok(code) => code,
                Err(_) => {
                    return Err(VetisError::VirtualHost(VirtualHostError::Interface(
                        "Invalid status code".to_string(),
                    )))
                }
            };

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
                    .body(HttpBody::from_bytes(&body))),
                Err(e) => {
                    error!("Failed to run script: {}", e);
                    Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())))
                }
            }
        })
    }
}
