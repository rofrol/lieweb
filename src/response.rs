use crate::utils::StdError;
use crate::Error;

const NOT_FOUND: &str = "Not Found";

pub(crate) type HyperResponse = hyper::Response<hyper::Body>;

pub trait IntoResponse: Send + Sized {
    /// Convert the value into a `Response`.
    fn into_response(self) -> Result<Response, StdError>;
}

pub struct Builder {
    inner: http::response::Builder,
}

impl Builder {
    pub fn new() -> Self {
        Builder::default()
    }

    pub fn with_text(self, html: impl Into<String>) -> Result<Response, Error> {
        let resp = self
            .inner
            .header(
                hyper::header::CONTENT_TYPE,
                mime::TEXT_PLAIN_UTF_8.to_string(),
            )
            .body(hyper::Body::from(html.into()))?;

        Ok(Response { inner: resp })
    }

    pub fn with_html(self, html: impl Into<String>) -> Result<Response, Error> {
        let resp = self
            .inner
            .header(
                hyper::header::CONTENT_TYPE,
                mime::TEXT_HTML_UTF_8.to_string(),
            )
            .body(hyper::Body::from(html.into()))?;

        Ok(Response { inner: resp })
    }

    pub fn with_json(self, json: impl serde::Serialize) -> Result<Response, Error> {
        let json = serde_json::to_string(&json)?;

        let resp = self
            .inner
            .header(
                hyper::header::CONTENT_TYPE,
                mime::APPLICATION_JSON.to_string(),
            )
            .body(hyper::Body::from(json))?;

        Ok(Response { inner: resp })
    }

    pub fn not_found() -> Result<Response, Error> {
        let resp = Self::new()
            .inner
            .header(
                hyper::header::CONTENT_TYPE,
                mime::TEXT_PLAIN_UTF_8.to_string(),
            )
            .body(hyper::Body::from(NOT_FOUND))?;

        Ok(Response { inner: resp })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            inner: hyper::Response::builder()
                .header(hyper::header::SERVER, crate::server::SERVER_ID.to_string()),
        }
    }
}

pub struct Response {
    pub(crate) inner: HyperResponse,
}

impl Response {
    pub fn builder() -> Builder {
        Builder {
            inner: hyper::Response::builder(),
        }
    }

    pub fn with_text(text: impl Into<String>) -> Result<Response, Error> {
        Self::builder().with_text(text)
    }

    pub fn with_html(html: impl Into<String>) -> Result<Response, Error> {
        Self::builder().with_html(html)
    }

    pub fn with_json(json: impl serde::Serialize) -> Result<Response, Error> {
        Self::builder().with_json(json)
    }

    pub fn not_found() -> Result<Response, Error> {
        Builder::not_found()
    }
}

impl From<Response> for HyperResponse {
    fn from(resp: Response) -> HyperResponse {
        let Response { inner, .. } = resp;
        inner
    }
}

impl From<HyperResponse> for Response {
    fn from(resp: HyperResponse) -> Response {
        Response { inner: resp }
    }
}

impl<E> IntoResponse for Result<Response, E>
where
    E: Into<StdError> + Send + Sync + 'static,
{
    fn into_response(self) -> Result<Response, StdError> {
        self.map_err(|e| e.into())
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Result<Response, StdError> {
        Ok(self)
    }
}
