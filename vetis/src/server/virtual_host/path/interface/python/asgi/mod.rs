use std::{collections::HashMap, ffi::CString, fs, future::Future, pin::Pin, sync::Arc};

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use log::error;
use pyo3::{
    types::{PyAnyMethods, PyIterator, PyModule, PyModuleMethods},
    Bound, PyAny, PyErr, PyResult, Python,
};

#[cfg(feature = "tokio-rt")]
use tokio::sync::oneshot;

#[cfg(feature = "smol-rt")]
use smol::oneshot;

use crate::{
    errors::{VetisError, VirtualHostError},
    server::{
        http::{Request, Response, VetisBody, VetisBodyExt},
        virtual_host::path::interface::{
            python::wsgi::callback::StartResponse, Interface, InterfaceWorker,
        },
    },
};

pub mod callback;

impl From<AsgiWorker> for Interface {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `Interface` - The interface
    fn from(value: AsgiWorker) -> Self {
        Interface::Asgi(value)
    }
}

pub struct AsgiWorker {
    directory: String,
    target: String,
}

impl AsgiWorker {
    pub fn new(directory: String, target: String) -> AsgiWorker {
        AsgiWorker { directory, target }
    }
}

impl InterfaceWorker for AsgiWorker {
    fn handle(
        &self,
        request: Arc<Request>,
        _uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + 'static>> {
        let mut response_body: Option<Vec<u8>> = None;

        let (tx, rx) = oneshot::channel::<(String, Vec<(String, String)>)>();
        let code = fs::read_to_string(&self.target);
        let code = match code {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to read script from target: {}", e);
                return Box::pin(async move {
                    Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())))
                });
            }
        };

        let code = CString::new(code);
        let code = match code {
            Ok(code) => code,
            Err(e) => {
                error!("Failed to initialize script: {}", e);
                return Box::pin(async move {
                    Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())))
                });
            }
        };

        let file = CString::new(self.target.as_str());
        let file = match file {
            Ok(file) => file,
            Err(e) => {
                error!("Failed to initialize target: {}", e);
                return Box::pin(async move {
                    Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())))
                });
            }
        };

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

        let result = Python::attach(|py| {
            let script_module = PyModule::from_code(py, &code, &file, c"main")?;

            let app = script_module.getattr("app")?;
            let handler_func = app.getattr("wsgi_app")?;
            let result: Bound<'_, PyAny> = if handler_func.is_callable() {
                let mut environ = HashMap::new();
                environ.insert("wsgi.url_scheme", "https");
                environ.insert("wsgi.version", "1.0");
                environ.insert("wsgi.input", "");
                environ.insert("wsgi.errors", "");
                environ.insert("wsgi.multithread", "false");
                environ.insert("wsgi.multiprocess", "false");
                environ.insert("wsgi.run_once", "false");
                environ.insert(
                    "REQUEST_METHOD",
                    request
                        .method()
                        .as_str(),
                );
                environ.insert("PATH_INFO", request.uri().path());
                environ.insert(
                    "QUERY_STRING",
                    request
                        .uri()
                        .query()
                        .unwrap_or_default(),
                );
                environ.insert("CONTENT_TYPE", content_type);
                environ.insert("CONTENT_LENGTH", content_length);
                environ.insert("SERVER_NAME", "localhost");
                environ.insert("SERVER_PORT", "8080");
                environ.insert("SERVER_PROTOCOL", "HTTP/1.1");
                environ.insert("SERVER_SOFTWARE", "Vetis");
                handler_func
                    .call1((environ, callback))?
                    .extract()?
            } else {
                handler_func.extract()?
            };

            script_module.add_class::<StartResponse>()?;

            py.run(&code, Some(&script_module.dict()), None)?;

            let iter = PyIterator::from_object(&result)?;

            let bytes = iter
                .map(|item| item?.extract::<Vec<u8>>())
                .collect::<PyResult<Vec<Vec<u8>>>>()?;

            response_body = Some(bytes[0].clone());

            Ok::<(), PyErr>(())
        });

        Box::pin(async move {
            let channel_result = rx.await;
            let (status, headers) = match channel_result {
                Ok(data) => data,
                Err(_) => {
                    return Err(VetisError::VirtualHost(VirtualHostError::Interface(
                        "Failed to run script".to_string(),
                    )))
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

            let headers = headers
                .into_iter()
                .fold(HeaderMap::new(), |mut map, (key, value)| {
                    map.insert(
                        HeaderName::from_bytes(key.as_bytes()).unwrap(),
                        HeaderValue::from_bytes(value.as_bytes()).unwrap(),
                    );
                    map
                });

            match result {
                Ok(_) => {
                    if response_body.is_none() {
                        return Err(VetisError::VirtualHost(VirtualHostError::Interface(
                            "No response body".to_string(),
                        )));
                    }
                    Ok(Response::builder()
                        .status(status_code)
                        .headers(headers)
                        .body(VetisBody::body_from_bytes(&response_body.unwrap())))
                }
                Err(e) => {
                    error!("Failed to run script: {}", e);
                    println!("Failed to run script: {}", e);
                    Err(VetisError::VirtualHost(VirtualHostError::Interface(e.to_string())))
                }
            }
        })
    }
}
