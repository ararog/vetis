use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;

/// HTTP request wrapper supporting multiple protocols.
///
/// The `Request` struct provides a unified interface for handling HTTP requests
/// from different protocols (HTTP/1, HTTP/2, HTTP/3). It abstracts away the protocol-specific
/// details while providing access to common request properties.
///
/// # Examples
///
/// ```rust,ignore
/// use vetis::Request;
///
/// // In a request handler:
/// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
///     let method = request.method();
///     let uri = request.uri();
///     let user_agent = request.headers().get("user-agent");
///     
///     // Process request...
///     
///     Ok(vetis::Response::builder()
///         .status(http::StatusCode::OK)
///         .text("Hello")))
/// }
/// ```
pub struct Request {
    pub(crate) inner_http: Option<http::Request<Incoming>>,
    pub(crate) inner_quic: Option<http::Request<Full<Bytes>>>,
}

impl Request {
    /// Creates a `Request` from an HTTP/1 or HTTP/2 request.
    ///
    /// This is used internally by the server to wrap incoming HTTP requests.
    pub fn from_http(req: http::Request<Incoming>) -> Self {
        Self { inner_http: Some(req), inner_quic: None }
    }

    /// Creates a `Request` from an HTTP/3 (QUIC) request.
    ///
    /// This is used internally by the server to wrap incoming QUIC requests.
    pub fn from_quic(req: http::Request<Full<Bytes>>) -> Self {
        Self { inner_http: None, inner_quic: Some(req) }
    }

    /// Returns the request URI.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Request;
    ///
    /// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
    ///     let path = request.uri().path();
    ///     let query = request.uri().query();
    ///     Ok(/* response */)
    /// }
    /// ```
    pub fn uri(&self) -> &http::Uri {
        match &self.inner_http {
            Some(req) => req.uri(),
            None => match &self.inner_quic {
                Some(req) => req.uri(),
                None => panic!("No request"),
            },
        }
    }

    /// Returns the request headers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Request;
    ///
    /// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
    ///     let content_type = request.headers().get("content-type");
    ///     let user_agent = request.headers().get("user-agent");
    ///     Ok(/* response */)
    /// }
    /// ```
    pub fn headers(&self) -> &http::HeaderMap {
        match &self.inner_http {
            Some(req) => req.headers(),
            None => match &self.inner_quic {
                Some(req) => req.headers(),
                None => panic!("No request"),
            },
        }
    }

    /// Returns the request headers (mutable).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Request;
    ///
    /// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
    ///     let content_type = request.headers().get("content-type");
    ///     let user_agent = request.headers().get("user-agent");
    ///     Ok(/* response */)
    /// }
    /// ```
    pub fn headers_mut(&mut self) -> &mut http::HeaderMap {
        match &mut self.inner_http {
            Some(req) => req.headers_mut(),
            None => match &mut self.inner_quic {
                Some(req) => req.headers_mut(),
                None => panic!("No request"),
            },
        }
    }

    /// Returns the HTTP method.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Request;
    ///
    /// async fn handler(request: Request) -> Result<vetis::Response, vetis::VetisError> {
    ///     match request.method() {
    ///         &http::Method::GET => { /* handle GET */ }
    ///         &http::Method::POST => { /* handle POST */ }
    ///         _ => { /* handle other methods */ }
    ///     }
    ///     Ok(/* response */)
    /// }
    /// ```
    pub fn method(&self) -> &http::Method {
        match &self.inner_http {
            Some(req) => req.method(),
            None => match &self.inner_quic {
                Some(req) => req.method(),
                None => panic!("No request"),
            },
        }
    }

    pub fn into_http_parts(self) -> (http::request::Parts, hyper::body::Incoming) {
        match self.inner_http {
            Some(req) => {
                let (parts, body) = req.into_parts();
                (parts, body)
            }
            None => {
                panic!("No request");
            }
        }
    }

    pub fn into_quic_parts(self) -> (http::request::Parts, Full<Bytes>) {
        match self.inner_quic {
            Some(req) => {
                let (parts, body) = req.into_parts();
                (parts, body)
            }
            None => {
                panic!("No request");
            }
        }
    }
}
