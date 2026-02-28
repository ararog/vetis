use hyper_body_utils::HttpBody;

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
    pub(crate) inner: Option<http::Request<HttpBody>>,
}

impl Request {
    /// Creates a `Request` from an HTTP/1 or HTTP/2 request.
    ///
    /// This is used internally by the server to wrap incoming HTTP requests.
    pub fn from_parts(parts: http::request::Parts, body: HttpBody) -> Self {
        Self { inner: Some(http::Request::from_parts(parts, body)) }
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
        match &self.inner {
            Some(req) => req.uri(),
            None => panic!("No request"),
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
        match &self.inner {
            Some(req) => req.headers(),
            None => panic!("No request"),
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
        match &mut self.inner {
            Some(req) => req.headers_mut(),
            None => panic!("No request"),
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
        match &self.inner {
            Some(req) => req.method(),
            None => panic!("No request"),
        }
    }

    pub fn into_parts(self) -> (http::request::Parts, HttpBody) {
        match self.inner {
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
