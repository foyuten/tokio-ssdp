use std::net::SocketAddr;

/// Represents a Universal Resource Name (URN) used in SSDP.
#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error("The request is incomplete and cannot be parsed.")]
    Incomplete,
    #[error("ParseError: {0}")]
    ParseError(#[from] httparse::Error),
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
}

/// Represents a NOTIFY request in the SSDP protocol.
#[derive(Debug, Clone)]
pub struct NotifyMessage {
    /// The remote address of the client that sent the NOTIFY request.
    pub remote_addr: SocketAddr,
    /// The data of the NOTIFY request, which includes the method, path, headers, and body.
    pub data: Vec<u8>,
}

impl NotifyMessage {
    /// Creates a new `NotifyMessage` with the given remote address and data.
    pub fn new(remote_addr: SocketAddr, data: Vec<u8>) -> Self {
        Self { remote_addr, data }
    }

    /// Parses the NOTIFY request from the raw data.
    pub fn parse(&self) -> Result<NotifyRequest, NotifyError> {
        NotifyRequest::parse(self.remote_addr, &self.data)
    }
}

/// Represents a NOTIFY request with parsed information.
#[derive(Debug, Clone)]
pub struct NotifyRequest {
    /// The remote address of the client that sent the request.
    pub remote_addr: SocketAddr,
    /// The HTTP method of the request (e.g., "NOTIFY").
    pub method: String,
    /// The path of the request (e.g., "/").
    pub path: String,
    /// The headers of the request, represented as a vector of tuples (header name, header value).
    pub headers: Vec<(String, String)>,
    /// The body of the request, which can contain additional information.
    pub body: String,
}

impl NotifyRequest {
    /// Parse the HTTP request from the given byte slice.
    /// # Arguments
    /// * `remote_addr` - The remote address of the client making the request.
    /// * `data` - A byte slice containing the HTTP request data.
    /// # Returns
    /// * `Ok(HttpRequest)` if the request is successfully parsed.
    /// * `Err(NotifyError)` if the request is incomplete or cannot be parsed.
    /// # Errors
    /// * `NotifyError::Incomplete` if the request is incomplete.
    pub fn parse(remote_addr: SocketAddr, data: &[u8]) -> Result<Self, NotifyError> {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let result = req.parse(data)?;

        match result {
            httparse::Status::Complete(n) => {
                let method = req.method.unwrap_or("").to_string();
                let path = req.path.unwrap_or("").to_string();
                let mut parsed_headers = Vec::new();
                for h in req.headers.iter() {
                    parsed_headers.push((
                        h.name.to_string(),
                        String::from_utf8_lossy(h.value).to_string(),
                    ));
                }
                let body = if n < data.len() {
                    String::from_utf8_lossy(&data[n..]).to_string()
                } else {
                    String::new()
                };
                Ok(NotifyRequest {
                    remote_addr,
                    method,
                    path,
                    headers: parsed_headers,
                    body,
                })
            }
            httparse::Status::Partial => Err(NotifyError::Incomplete),
        }
    }

    /// Check if the request header contains the given name and value.
    /// # Arguments
    /// * `name` - The name of the header to check.
    /// * `value` - The value to check for in the header.
    /// # Returns
    /// * `true` if the header contains the value, `false` otherwise.
    pub fn header_contains(&self, name: &str, value: &str) -> bool {
        self.headers
            .iter()
            .any(|(h_name, h_value)| h_name.eq_ignore_ascii_case(name) && h_value.contains(value))
    }

    /// Check if the request header matches the given name and value.
    ///
    /// # Arguments
    /// * `name` - The name of the header to match.
    /// * `value` - The value of the header to match.
    /// # Returns
    /// * `true` if the header matches, `false` otherwise.
    pub fn header_match(&self, name: &str, value: &str) -> bool {
        self.headers.iter().any(|(h_name, h_value)| {
            h_name.eq_ignore_ascii_case(name) && h_value.eq_ignore_ascii_case(value)
        })
    }
}

/// Represents a NOTIFY response sent to the client.
#[derive(Debug, Clone)]
pub struct NotifyResponse {
    /// The remote address of the client that sent the request.
    pub remote_addr: SocketAddr,
    /// The HTTP status code of the response (e.g., 200 for OK).
    pub status_code: u16,
    /// The headers of the response, represented as a vector of tuples (header name, header value).
    pub headers: Vec<(String, String)>,
    /// The body of the response, which can contain additional information.
    pub body: String,
}
