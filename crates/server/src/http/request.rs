use std::{
    collections::HashMap,
    net::IpAddr,
    ops::{Deref, DerefMut},
};

use config::frontend::FrontendConfig;
use hyper::{StatusCode, body::Incoming, header};
use proto::types::{DataRequestType, DataResponseType};

use crate::{
    SharedConfig,
    backend::{BackendHandle, SharedBackendRegistry},
};

use super::response::ServerResponse;

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

pub struct BackendData {
    pub backend_list: Vec<(IpAddr, String)>,
    pub current_backend: (IpAddr, BackendHandle),
}

pub struct ServerRequest {
    req: HyperRequest,
    cookies: HashMap<String, String>,
    backends: SharedBackendRegistry,
    config: SharedConfig,
}

impl ServerRequest {
    pub fn new(req: HyperRequest, backends: SharedBackendRegistry, config: SharedConfig) -> Self {
        let cookies = get_cookies(&req);

        Self {
            req,
            cookies,
            backends,
            config,
        }
    }

    // This function should only be called if the correct middleware was used to add the extension
    pub fn extract_backends(&self) -> Result<BackendData, ServerResponse> {
        let backends = self.backends.borrow();
        let backend_list: Vec<_> = backends
            .iter()
            .map(|(addr, info)| (*addr, info.nickname.clone()))
            .collect();

        if backend_list.is_empty() {
            return Err(ServerResponse::new()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body("no connected backends"));
        }

        let current_backend = {
            let cookie_ip = self
                .cookies
                .get("backend")
                .and_then(|x| x.parse::<IpAddr>().ok());

            let (addr, backend_info) = cookie_ip
                .and_then(|x| backends.get_key_value(&x))
                .or_else(|| backends.get_key_value(&backend_list[0].0))
                .unwrap();

            (*addr, backend_info.handle.clone())
        };

        Ok(BackendData {
            backend_list,
            current_backend,
        })
    }

    pub async fn send_backend_req_oneshot(
        &self,
        req: DataRequestType,
    ) -> Result<DataResponseType, ServerResponse> {
        let backend_handle = self.extract_backends()?.current_backend.1;

        backend_handle.send_req_oneshot(req).await.map_err(|err| {
            ServerResponse::new()
                .status(StatusCode::BAD_GATEWAY)
                .body(format!("backend request failed: {err}"))
        })
    }

    pub fn extract_query<Qu: serde::de::DeserializeOwned>(&self) -> Result<Qu, ServerResponse> {
        let query = self.uri().query().unwrap_or_default();

        serde_urlencoded::from_str::<Qu>(query).map_err(|_| {
            ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body("invalid query params")
        })
    }

    pub fn is_fixi(&self) -> bool {
        self.headers().contains_key("fx-request")
    }

    pub fn config(&self) -> &FrontendConfig {
        &self.config
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

// Wrapper type that makes de/serialize multiple fields in a query param easier
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct QueryArray(String);

impl QueryArray {
    pub fn from_iter<T, I>(iter: I) -> Self
    where
        T: std::fmt::Display,
        I: IntoIterator<Item = T>,
    {
        use core::fmt::Write;

        let inner = iter.into_iter().fold(String::new(), |mut acc, x| {
            let _ = write!(&mut acc, "{x},");
            acc
        });

        Self(inner)
    }

    pub fn to_iter<T>(&self) -> impl Iterator<Item = T> + Clone
    where
        T: std::str::FromStr,
    {
        self.0.split(',').filter_map(|x| x.parse().ok())
    }
}
