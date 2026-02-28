use filedescriptor::{AsRawFileDescriptor, FileDescriptor, RawFileDescriptor};
use hyper_body_utils::HttpBody;
use log::error;

#[cfg(feature = "smol-rt")]
use futures_lite::AsyncSeekExt;
use lru::LruCache;
#[cfg(feature = "tokio-rt")]
use tokio::io::AsyncSeekExt;

use crate::{
    config::server::virtual_host::path::static_files::StaticPathConfig,
    errors::{FileError, VetisError, VirtualHostError},
    server::{
        http::{static_response, Request, Response},
        virtual_host::path::{HostPath, Path},
    },
    VetisFile, VetisRwLock,
};
use http::{HeaderMap, HeaderValue};
use std::{future::Future, num::NonZeroUsize, path::PathBuf, pin::Pin, sync::Arc};

#[cfg(feature = "auth")]
use crate::server::virtual_host::path::auth::Auth;

pub(crate) type VetisFileCache = Arc<VetisRwLock<LruCache<String, RawFileDescriptor>>>;

/// Static path
pub struct StaticPath {
    config: StaticPathConfig,
    index_file: Option<String>,
    file_cache: VetisFileCache,
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
        let file_cache = Arc::new(VetisRwLock::new(LruCache::new(NonZeroUsize::new(100).unwrap())));
        if let Some(index_files) = config.index_files() {
            let directory = PathBuf::from(config.directory());
            if let Some(index_file) = index_files
                .iter()
                .find(|index_file| {
                    directory
                        .join(index_file)
                        .exists()
                })
            {
                return StaticPath {
                    config: config.clone(),
                    index_file: Some(index_file.to_string()),
                    file_cache,
                };
            }
        }
        StaticPath { config, index_file: None, file_cache }
    }

    async fn cache_file(&self, file_path: &std::path::Path) -> Result<VetisFile, VetisError> {
        let path = file_path
            .display()
            .to_string();
        let lock = self
            .file_cache
            .clone();

        let open_file = move || {
            #[cfg(feature = "tokio-rt")]
            let mut lock = lock.blocking_write();

            #[cfg(feature = "smol-rt")]
            let mut lock = lock.write_blocking();

            if let Some(raw_fd) = lock.get(&path) {
                FileDescriptor::dup(raw_fd)
            } else {
                let file = std::fs::File::open(path.clone());
                match file {
                    Ok(file) => {
                        let raw_fd =
                            lock.get_or_insert(path.clone(), || file.as_raw_file_descriptor());
                        FileDescriptor::dup(raw_fd)
                    }
                    Err(e) => {
                        error!("Error opening file {}: {}", path, e);
                        Err(filedescriptor::Error::Io(e))
                    }
                }
            }
        };

        #[cfg(feature = "tokio-rt")]
        let task = {
            let task = tokio::task::spawn_blocking(open_file);
            match task.await {
                Ok(result) => result,
                Err(e) => Err(filedescriptor::Error::Io(e.into())),
            }
        };

        #[cfg(feature = "smol-rt")]
        let task = smol::unblock(open_file).await;

        match task {
            Ok(raw_fd) => {
                let file = raw_fd.as_file();
                match file {
                    Ok(file) => {
                        #[cfg(feature = "tokio-rt")]
                        let file = VetisFile::from_std(file);

                        #[cfg(feature = "smol-rt")]
                        let file = VetisFile::from(file);

                        Ok(file)
                    }
                    Err(e) => {
                        error!("Error opening file {}", e);
                        Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
                    }
                }
            }
            Err(e) => {
                error!("Error opening file: {}", e);
                Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
            }
        }
    }

    async fn serve_file(
        &self,
        file_path: &std::path::Path,
        range: Option<&str>,
    ) -> Result<Response, VetisError> {
        let mut file = self
            .cache_file(file_path)
            .await?;

        let filesize = match file
            .metadata()
            .await
        {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                error!("Error getting metadata for file {}: {}", file_path.display(), e);
                return Err(VetisError::VirtualHost(VirtualHostError::File(
                    FileError::InvalidMetadata,
                )));
            }
        };

        if let Some(range) = range {
            let range_info = match range
                .split_once("=")
                .ok_or(VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange)))
            {
                Ok(info) => info,
                Err(e) => return Err(e),
            };

            let (unit, range) = range_info;
            if unit != "bytes" {
                return Err(VetisError::VirtualHost(VirtualHostError::File(
                    FileError::InvalidRange,
                )));
            }

            let (start, end) = range
                .split_once("-")
                .ok_or(VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange)))?;
            let start = start
                .parse::<u64>()
                .map_err(|_| {
                    VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange))
                })?;
            let end = end
                .parse::<u64>()
                .map_err(|_| {
                    VetisError::VirtualHost(VirtualHostError::File(FileError::InvalidRange))
                })?;
            if start > end || start >= filesize {
                return Ok(Response::builder()
                    .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
                    .body(HttpBody::from_text("")));
            } else if start < end
                && end < filesize
                && file
                    .seek(std::io::SeekFrom::Start(start))
                    .await
                    .is_ok()
            {
                return Ok(Response::builder()
                    .status(http::StatusCode::PARTIAL_CONTENT)
                    .body(HttpBody::from_file(file)));
            }
        }

        Ok(Response::builder()
            .status(http::StatusCode::OK)
            .header(
                http::header::ACCEPT_RANGES,
                "bytes"
                    .parse()
                    .unwrap(),
            )
            .header(http::header::CONTENT_LENGTH, HeaderValue::from(filesize))
            .body(HttpBody::from_file(file)))
    }

    async fn serve_metadata(&self, file_path: PathBuf) -> Result<Response, VetisError> {
        let file = self
            .cache_file(&file_path)
            .await?;

        let metadata = match file
            .metadata()
            .await
        {
            Ok(metadata) => metadata,
            Err(e) => {
                error!("Error getting metadata for file {:?}: {}", file_path, e);
                return Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)));
            }
        };

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
                        .map_err(|_| {
                            VetisError::VirtualHost(VirtualHostError::File(
                                FileError::InvalidMetadata,
                            ))
                        })?,
                );
            }
            Err(_) => todo!(),
        }

        match file_path.file_name() {
            Some(filename) => {
                let mime_type = minimime::lookup_by_filename(
                    filename
                        .to_str()
                        .ok_or(VetisError::VirtualHost(VirtualHostError::File(
                            FileError::InvalidMetadata,
                        )))?,
                );
                if let Some(mime_type) = mime_type {
                    headers.insert(
                        http::header::CONTENT_TYPE,
                        HeaderValue::from_str(
                            mime_type
                                .content_type
                                .as_str(),
                        )
                        .map_err(|_| {
                            VetisError::VirtualHost(VirtualHostError::File(
                                FileError::InvalidMetadata,
                            ))
                        })?,
                    );
                }
            }
            None => {
                return Err(VetisError::VirtualHost(VirtualHostError::File(
                    FileError::InvalidMetadata,
                )));
            }
        }

        Ok(Response { inner: static_response(http::StatusCode::OK, Some(headers), String::new()) })
    }

    async fn serve_index_file(&self, directory: &std::path::Path) -> Result<Response, VetisError> {
        match &self.index_file {
            Some(index_file) => {
                let full_path = directory.join(index_file);
                self.serve_file(&full_path, None)
                    .await
            }
            None => {
                println!("No index file configured");
                Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)))
            }
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
                    .await
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
                if !file.exists() {
                    if let Ok(ext_regex) = ext_regex {
                        if !ext_regex.is_match(uri.as_ref()) {
                            return self
                                .serve_index_file(&directory)
                                .await;
                        }
                    }
                } else if file.is_dir() {
                    return self
                        .serve_index_file(&file)
                        .await;
                }
            } else if !file.exists() {
                return Err(VetisError::VirtualHost(VirtualHostError::File(FileError::NotFound)));
            }

            if request.method() == http::Method::HEAD {
                return self
                    .serve_metadata(file)
                    .await;
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

            self.serve_file(&file, range)
                .await
        })
    }
}
