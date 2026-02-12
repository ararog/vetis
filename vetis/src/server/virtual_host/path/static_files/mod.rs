#[cfg(feature = "smol-rt")]
use futures_lite::AsyncSeekExt;
#[cfg(feature = "smol-rt")]
use smol::fs::File;
#[cfg(feature = "tokio-rt")]
use tokio::{fs::File, io::AsyncSeekExt};

use crate::{
    config::server::virtual_host::path::static_files::StaticPathConfig,
    errors::{FileError, VetisError, VirtualHostError},
    server::{
        http::static_response,
        virtual_host::path::{HostPath, Path},
    },
    Request, Response, VetisBody, VetisBodyExt,
};
use http::{HeaderMap, HeaderValue};
use std::{future::Future, path::PathBuf, pin::Pin, sync::Arc};

#[cfg(feature = "auth")]
use crate::server::virtual_host::path::auth::Auth;

/// Static path
pub struct StaticPath {
    config: StaticPathConfig,
}

impl StaticPath {
    /// Create a new static path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the static path
    ///
    /// # Returns
    ///
    /// * `StaticPath` - The static path
    pub fn new(config: StaticPathConfig) -> StaticPath {
        StaticPath { config }
    }

    async fn serve_file(&self, file: PathBuf, range: Option<&str>) -> Result<Response, VetisError> {
        let result = File::open(file).await;
        if let Ok(mut data) = result {
            let filesize = match data
                .metadata()
                .await
            {
                Ok(metadata) => metadata.len(),
                Err(_) => 0u64,
            };

            if let Some(range) = range {
                let (unit, range) = range
                    .split_once("=")
                    .unwrap();
                if unit != "bytes" {
                    return Err(VetisError::VirtualHost(VirtualHostError::File(
                        FileError::InvalidRange,
                    )));
                }

                let (start, end) = range
                    .split_once("-")
                    .unwrap();
                let start = start
                    .parse::<u64>()
                    .unwrap();
                let end = end
                    .parse::<u64>()
                    .unwrap();
                if start > end || start >= filesize {
                    return Ok(Response::builder()
                        .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
                        .body(VetisBody::body_from_text("")));
                } else if start < end
                    && end < filesize
                    && data
                        .seek(std::io::SeekFrom::Start(start))
                        .await
                        .is_ok()
                {
                    return Ok(Response::builder()
                        .status(http::StatusCode::PARTIAL_CONTENT)
                        .body(VetisBody::body_from_file(data)));
                }
            }

            return Ok(Response::builder()
                .status(http::StatusCode::OK)
                .header(
                    http::header::ACCEPT_RANGES,
                    "bytes"
                        .parse()
                        .unwrap(),
                )
                .header(http::header::CONTENT_LENGTH, HeaderValue::from(filesize))
                .body(VetisBody::body_from_file(data)));
        }

        Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
    }

    async fn serve_index_file(&self, directory: PathBuf) -> Result<Response, VetisError> {
        if let Some(index_files) = self
            .config
            .index_files()
        {
            if let Some(index_file) = index_files
                .iter()
                .find(|index_file| {
                    directory
                        .join(index_file)
                        .exists()
                })
            {
                return self
                    .serve_file(directory.join(index_file), None)
                    .await;
            }
        }

        Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
    }

    fn serve_metadata(&self, file: PathBuf) -> Result<Response, VetisError> {
        if let Ok(metadata) = file.metadata() {
            let len = metadata.len();
            let mut headers = HeaderMap::new();
            match len
                .to_string()
                .parse()
            {
                Ok(len) => {
                    headers.insert(http::header::CONTENT_LENGTH, len);
                }
                Err(_) => todo!(),
            }
            let last_modified = metadata.modified();
            match last_modified {
                Ok(date) => {
                    let date = crate::utils::date::format_date(date);
                    headers.insert(
                        http::header::LAST_MODIFIED,
                        date.parse()
                            .unwrap(),
                    );
                }
                Err(_) => todo!(),
            }
            match file.file_name() {
                Some(filename) => {
                    let mime_type = minimime::lookup_by_filename(
                        filename
                            .to_str()
                            .unwrap(),
                    );
                    if let Some(mime_type) = mime_type {
                        headers.insert(
                            http::header::CONTENT_TYPE,
                            HeaderValue::from_str(
                                mime_type
                                    .content_type
                                    .as_str(),
                            )
                            .unwrap(),
                        );
                    }
                }
                None => {
                    todo!()
                }
            }

            Ok(Response {
                inner: static_response(http::StatusCode::OK, Some(headers), String::new()),
            })
        } else {
            Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
        }
    }
}

impl From<StaticPath> for HostPath {
    /// Convert static path to host path
    ///
    /// # Arguments
    ///
    /// * `value` - The static path to convert
    ///
    /// # Returns
    ///
    /// * `HostPath` - The host path
    fn from(value: StaticPath) -> Self {
        HostPath::Static(value)
    }
}

impl Path for StaticPath {
    /// Returns the uri of the static path
    ///
    /// # Returns
    ///
    /// * `&str` - The uri of the static path
    fn uri(&self) -> &str {
        self.config.uri()
    }

    /// Handles the request for the static path
    ///
    /// # Returns
    ///
    /// * `Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>>` - The response to the request
    fn handle(
        &self,
        request: Request,
        uri: Arc<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send + '_>> {
        Box::pin(async move {
            let ext_regex = regex::Regex::new(
                self.config
                    .extensions(),
            );

            let directory = PathBuf::from(
                self.config
                    .directory(),
            );

            #[cfg(feature = "auth")]
            if let Some(auth) = self.config.auth() {
                if !auth
                    .authenticate(request.headers())
                    .unwrap_or(false)
                {
                    return Err(VetisError::VirtualHost(VirtualHostError::Auth(
                        "Unauthorized".to_string(),
                    )));
                }
            }

            let uri = uri
                .strip_prefix("/")
                .unwrap_or(&uri);
            let file = directory.join(uri);

            if self
                .config
                .index_files()
                .is_some()
            {
                // check if file exists
                if !file.exists() {
                    // check file by mimetype
                    if let Ok(ext_regex) = ext_regex {
                        if !ext_regex.is_match(uri.as_ref()) {
                            return self
                                .serve_index_file(directory)
                                .await;
                        }
                    }
                } else if file.is_dir() {
                    return self
                        .serve_index_file(file)
                        .await;
                }
            } else {
                // no index files configured, just check if file exists
                if !file.exists() {
                    return Err(VetisError::VirtualHost(VirtualHostError::File(
                        FileError::NotFound,
                    )));
                }
            }

            if request.method() == http::Method::HEAD {
                return self.serve_metadata(file);
            }

            let range = if request
                .headers()
                .contains_key(http::header::RANGE)
            {
                let value = request
                    .headers()
                    .get(http::header::RANGE);
                Some(
                    value
                        .unwrap()
                        .to_str()
                        .unwrap(),
                )
            } else {
                None
            };

            self.serve_file(file, range)
                .await
        })
    }
}
