use crate::http::{
    request::ServerRequest,
    response::{set_cookie, ServerResponse},
};

#[derive(Clone, serde::Deserialize)]
pub struct SetBackendQuery {
    // Technically an IpAddr, but there's no point in converting to IpAddr and back to String
    backend: String,
}

pub async fn set_backend(req: ServerRequest) -> ServerResponse {
    let backend = &req.get_extension::<SetBackendQuery>().backend;

    set_cookie("backend", backend, None)
}
