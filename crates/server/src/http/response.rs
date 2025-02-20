use http_body_util::Full;
use hyper::{
    body::Bytes,
    header::{self},
    Response, StatusCode,
};

pub type ServerResponse = hyper::Response<Full<Bytes>>;

pub fn full_with_status<T: Into<Bytes>>(body: T, code: StatusCode) -> ServerResponse {
    let body = Full::from(body.into());

    Response::builder().status(code).body(body).unwrap()
}

pub fn full_with_mime<T: Into<Bytes>>(body: T, mime: &str) -> ServerResponse {
    let body = Full::from(body.into());

    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .body(body)
        .unwrap()
}

pub fn html<T: Into<Bytes>>(body: T) -> ServerResponse {
    full_with_mime(body, "text/html;charset=UTF-8")
}

pub fn not_found() -> ServerResponse {
    full_with_status("page not found", StatusCode::NOT_FOUND)
}

pub fn set_cookie(name: &str, val: &str, max_age: Option<u64>) -> ServerResponse {
    // Default of roughly 31 years
    let max_age = max_age.unwrap_or(999999999);

    let cookie_val = format!("{name}={val}; Path=/; SameSite=Lax; Max-Age={max_age}");
    Response::builder()
        .header(header::SET_COOKIE, cookie_val)
        .body(Full::from(""))
        .unwrap()
}

pub enum RedirectType {
    Permanent,
    SeeOther,
    // Fixi,
}

pub fn redirect(typ: RedirectType, path: &str) -> ServerResponse {
    let (status, header) = match typ {
        RedirectType::Permanent => (StatusCode::PERMANENT_REDIRECT, header::LOCATION),
        RedirectType::SeeOther => (StatusCode::SEE_OTHER, header::LOCATION),
        // RedirectType::Fixi => (StatusCode::OK, HeaderName::from_static("fx-redirect")),
    };

    Response::builder()
        .status(status)
        .header(header, path)
        .body(Full::from(""))
        .unwrap()
}
