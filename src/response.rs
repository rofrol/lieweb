use std::borrow::Cow;

use http::header::{HeaderName, HeaderValue};
use http::StatusCode;

pub type Response = http::Response<hyper::Body>;

pub struct Html<T> {
    body: T,
}

pub fn html<T>(body: T) -> Html<T>
where
    hyper::Body: From<T>,
    T: Send,
{
    Html { body }
}

impl<T> IntoResponse for Html<T>
where
    hyper::Body: From<T>,
    T: Send,
{
    fn into_response(self) -> Response {
        http::Response::builder()
            .header(
                hyper::header::CONTENT_TYPE,
                mime::TEXT_HTML_UTF_8.to_string(),
            )
            .body(hyper::Body::from(self.body))
            .unwrap()
    }
}

pub struct Json {
    inner: Result<Vec<u8>, serde_json::Error>,
}

pub fn json<T>(val: &T) -> Json
where
    T: serde::Serialize,
{
    Json {
        inner: serde_json::to_vec(val),
    }
}

impl IntoResponse for Json {
    fn into_response(self) -> Response {
        self.inner
            .map(|j| {
                http::Response::builder()
                    .header(
                        hyper::header::CONTENT_TYPE,
                        mime::APPLICATION_JSON.to_string(),
                    )
                    .body(hyper::Body::from(j))
                    .unwrap()
            })
            .map_err(|e| {
                log::error!("json serialize failed, {:?}", e);
                e
            })
            .into_response()
    }
}

pub struct WithStatus<T> {
    response: T,
    status: StatusCode,
}

pub fn with_status<R: IntoResponse>(response: R, status: StatusCode) -> WithStatus<R> {
    WithStatus { response, status }
}

impl<T> WithStatus<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut resp = self.response.into_response();
        *resp.status_mut() = self.status;
        resp
    }
}

pub struct WithHeader<T> {
    header: (HeaderName, HeaderValue),
    response: T,
}

pub fn with_header<T, K, V>(response: T, name: K, value: V) -> WithHeader<T>
where
    T: IntoResponse,
    HeaderName: From<K>,
    HeaderValue: From<V>,
{
    let header = (name.into(), value.into());
    WithHeader { header, response }
}

impl<T> IntoResponse for WithHeader<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut resp = self.response.into_response();
        resp.headers_mut().insert(self.header.0, self.header.1);
        resp
    }
}

pub trait IntoResponse: Send + Sized {
    /// Convert the value into a `Response`.
    fn into_response(self) -> Response;
}

impl<E, R> IntoResponse for Result<R, E>
where
    R: IntoResponse,
    E: std::error::Error + 'static + Send + Sync,
{
    fn into_response(self) -> Response {
        match self {
            Ok(r) => r.into_response(),
            Err(e) => {
                log::error!("on Result<R, E>, error: {:?}", e);

                http::Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(hyper::Body::from("Internal Server Error"))
                    .unwrap()
            }
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        http::Response::builder()
            .status(self)
            .body(hyper::Body::empty())
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        http::Response::builder()
            .header(
                hyper::header::CONTENT_TYPE,
                mime::TEXT_PLAIN_UTF_8.to_string(),
            )
            .body(hyper::Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        http::Response::builder()
            .header(
                hyper::header::CONTENT_TYPE,
                mime::TEXT_PLAIN_UTF_8.to_string(),
            )
            .body(hyper::Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for Cow<'static, str> {
    #[inline]
    fn into_response(self) -> Response {
        match self {
            Cow::Borrowed(s) => s.into_response(),
            Cow::Owned(s) => s.into_response(),
        }
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        http::Response::builder()
            .header(
                hyper::header::CONTENT_TYPE,
                mime::APPLICATION_OCTET_STREAM.to_string(),
            )
            .body(hyper::Body::from(self))
            .unwrap()
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        http::Response::builder()
            .header(
                hyper::header::CONTENT_TYPE,
                mime::APPLICATION_OCTET_STREAM.to_string(),
            )
            .body(hyper::Body::from(self))
            .unwrap()
    }
}
