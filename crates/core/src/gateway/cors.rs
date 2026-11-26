use axum_core::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header};

pub(super) fn preflight(headers: &HeaderMap) -> Response {
    let method = headers
        .get(header::ACCESS_CONTROL_REQUEST_METHOD)
        .and_then(|v| v.to_str().ok());
    if !matches!(method, Some("GET" | "POST" | "OPTIONS")) {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    let out = response.headers_mut();
    let mut requested = Vec::new();
    for value in headers.get_all(header::ACCESS_CONTROL_REQUEST_HEADERS) {
        let Ok(value) = value.to_str() else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        for name in value.split(',').map(str::trim) {
            let Ok(name) = HeaderName::from_bytes(name.as_bytes()) else {
                return StatusCode::BAD_REQUEST.into_response();
            };
            requested.push(name);
        }
    }
    if !requested.is_empty() {
        let names = requested
            .iter()
            .map(HeaderName::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        out.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_str(&names).expect("valid header names"),
        );
    }
    out.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    out.insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("600"),
    );
    if headers
        .get("access-control-request-private-network")
        .is_some_and(|v| v == "true")
    {
        out.insert(
            "access-control-allow-private-network",
            HeaderValue::from_static("true"),
        );
    }
    out.append(header::VARY, HeaderValue::from_static("Access-Control-Request-Method, Access-Control-Request-Headers, Access-Control-Request-Private-Network"));
    response
}
