use std::{
    collections::HashMap,
    net::IpAddr,
    ops::{Deref, DerefMut},
};

use hyper::{StatusCode, body::Incoming, header};

use crate::backend::{BackendHandle, SharedBackendRegistry};

use super::response::{ServerResponse, full_with_status};

pub type HyperRequest = hyper::Request<Incoming>;

fn get_cookies(req: &HyperRequest) -> HashMap<String, String> {
    let cookie_header = req
        .headers()
        .get(header::COOKIE)
        .and_then(|x| x.to_str().ok())
        .unwrap_or_default();

    cookie_header
        .split("; ")
        .filter_map(|x| x.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub struct ServerRequest {
    req: HyperRequest,
    cookies: HashMap<String, String>,
    backends: SharedBackendRegistry,
}

impl ServerRequest {
    pub fn new(req: HyperRequest, backends: SharedBackendRegistry) -> Self {
        let cookies = get_cookies(&req);

        Self {
            req,
            cookies,
            backends,
        }
    }

    // This function should only be called if the correct middleware was used to add the extension
    pub fn get_extension<T: Send + Sync + 'static>(&self) -> &T {
        self.extensions().get().unwrap()
    }
}

impl Deref for ServerRequest {
    type Target = HyperRequest;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}

impl DerefMut for ServerRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.req
    }
}

#[derive(Clone)]
pub struct BackendData {
    pub backend_list: Vec<(IpAddr, String)>,
    pub current_backend: (IpAddr, BackendHandle),
}

pub fn extract_backends(req: &mut ServerRequest) -> Option<ServerResponse> {
    let backends = req.backends.lock().unwrap();
    let backend_list: Vec<_> = backends
        .iter()
        .map(|(addr, info)| (*addr, info.nickname.clone()))
        .collect();

    if backend_list.is_empty() {
        return Some(full_with_status(
            "no connected backends",
            StatusCode::BAD_GATEWAY,
        ));
    }

    let current_backend = {
        let cookie_ip = req
            .cookies
            .get("backend")
            .and_then(|x| x.parse::<IpAddr>().ok());

        let (addr, backend_info) = cookie_ip
            .and_then(|x| backends.get_key_value(&x))
            .or_else(|| backends.get_key_value(&backend_list[0].0))
            .unwrap();

        (*addr, backend_info.handle.clone())
    };

    let backend_data = BackendData {
        backend_list,
        current_backend,
    };

    drop(backends);

    req.extensions_mut().insert(backend_data);

    None
}

pub fn extract_query<Qu: serde::de::DeserializeOwned + Clone + Send + Sync + 'static>(
    req: &mut ServerRequest,
) -> Option<ServerResponse> {
    let query = req.uri().query().unwrap_or_default();

    let Ok(query) = serde_urlencoded::from_str::<Qu>(query) else {
        return Some(full_with_status(
            "invalid query params",
            StatusCode::BAD_REQUEST,
        ));
    };

    req.extensions_mut().insert(query);

    None
}
