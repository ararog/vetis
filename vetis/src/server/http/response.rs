use crate::server::http::{VetisBody, VetisBodyExt};

/// Builder for creating HTTP responses.
///
/// `ResponseBuilder` provides a fluent interface for constructing HTTP responses
/// with custom status codes, headers, and body content.
///
/// # Examples
///
/// ```rust,ignore
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use http::StatusCode;
/// use vetis::Response;
///
/// // Simple response
/// let response = Response::builder()
///     .status(StatusCode::OK)
///     .text("Hello, World!");
///
/// // Response with custom headers
/// let mut headers = http::HeaderMap::new();
/// headers.insert("content-type", "application/json".parse().unwrap());
/// let response = Response::builder()
///     .status(StatusCode::CREATED)
///     .headers(headers)
///     .text(r#"{"status": "success"}"#);
/// ```
pub struct ResponseBuilder {
    status: http::StatusCode,
    version: http::Version,
    headers: Option<http::HeaderMap>,
}

impl ResponseBuilder {
    /// Sets the HTTP status code for the response.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    /// use http::StatusCode;
    ///
    /// let response = Response::builder()
    ///     .status(StatusCode::NOT_FOUND)
    ///     .text("Not found");
    /// ```
    pub fn status(mut self, status: http::StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Sets the HTTP version for the response.
    ///
    /// By default, responses use HTTP/1.1.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    /// use http::Version;
    ///
    /// let response = Response::builder()
    ///     .version(http::Version::HTTP_2)
    ///     .text("Response");
    /// ```
    pub fn version(mut self, version: http::Version) -> Self {
        self.version = version;
        self
    }

    /// Adds a header to the response.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .header("content-type", "text/plain".parse().unwrap())
    ///     .text("Plain text");
    /// ```
    pub fn header<K>(mut self, key: K, value: http::header::HeaderValue) -> Self
    where
        K: http::header::IntoHeaderName,
    {
        if self
            .headers
            .is_none()
        {
            self.headers = Some(http::HeaderMap::new());
        }
        self.headers
            .as_mut()
            .unwrap()
            .append(key, value);
        self
    }

    /// Sets the headers for the response.
    ///
    /// This replaces all existing headers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let mut headers = http::HeaderMap::new();
    /// headers.insert("content-type", "text/plain".parse().unwrap());
    ///
    /// let response = Response::builder()
    ///     .headers(headers)
    ///     .text("Plain text");
    /// ```
    pub fn headers(mut self, headers: http::HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Sets the body from a text string and creates the final `Response`.
    ///
    /// # Arguments
    ///
    /// * `text` - The response body as a text slice
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .text("Hello, World!");
    /// ```    
    pub fn text(self, text: &str) -> Response {
        self.body(VetisBody::body_from_text(text))
    }

    /// Sets the body with bytes and creates the final `Response`.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The response body as a `Bytes`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .bytes(b"Hello, World!");
    /// ```
    pub fn bytes(self, bytes: &[u8]) -> Response {
        self.body(VetisBody::body_from_bytes(bytes))
    }

    /// Sets the body and creates the final `Response`.
    ///
    /// # Arguments
    ///
    /// * `body` - The response body as a `VetisBody`
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .body(b"Hello, World!");
    /// ```
    pub fn body(self, body: VetisBody) -> Response {
        let response = http::Response::new(body);

        let (mut parts, body) = response.into_parts();
        parts.status = self.status;
        parts.version = self.version;
        if let Some(headers) = self.headers {
            parts.headers = headers;
        }

        let response = http::Response::from_parts(parts, body);

        Response { inner: response }
    }
}

/// HTTP response containing status, headers, and body.
///
/// The `Response` struct represents an HTTP response that can be sent back to clients.
/// It's created using the `Response::builder()` method and contains the response body.
///
/// # Examples
///
/// ```rust,ignore
/// use bytes::Bytes;
/// use http_body_util::Full;
/// use http::StatusCode;
/// use vetis::Response;
///
/// // Create a simple response
/// let response = Response::builder()
///     .status(StatusCode::OK)
///     .text("Hello, World!");
///
/// // Convert to inner http::Response if needed
/// let inner_response = response.into_inner();
/// ```
pub struct Response {
    pub(crate) inner: http::Response<VetisBody>,
}

impl Response {
    /// Creates a new `ResponseBuilder` with default settings.
    ///
    /// The builder starts with:
    /// - Status: 200 OK
    /// - Version: HTTP/1.1
    /// - No headers
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let builder = Response::builder();
    /// let response = builder.text("Hello");
    /// ```
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder {
            status: http::StatusCode::OK,
            version: http::Version::HTTP_11,
            headers: None,
        }
    }

    /// Converts the response into the underlying `http::Response`.
    ///
    /// This is useful when you need to work with the standard library HTTP types
    /// or pass the response to other libraries that expect `http::Response`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use vetis::Response;
    ///
    /// let response = Response::builder()
    ///     .text("Hello");
    /// let inner = response.into_inner();
    /// ```
    pub fn into_inner(self) -> http::Response<VetisBody> {
        self.inner
    }
}
