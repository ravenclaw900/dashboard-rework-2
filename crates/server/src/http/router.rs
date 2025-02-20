use hyper::Method;

use crate::pages;

use super::request::{extract_backends, extract_query};
use super::response::{RedirectType, ServerResponse, not_found, redirect};
use super::{request::ServerRequest, statics};

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;

macro_rules! router {
    ($req:expr, $path:expr, {
        $( ($method:pat, $paths:pat) => ([$( $middleware:expr ),*], $handler:expr), )*
        _ => $fallback:expr,
    }) => {{
        match ($req.method(), $path) {
            $(
                ($method, $paths) => {
                        $(
                            if let Some(resp) = $middleware(&mut $req) {
                                resp
                            } else
                        )*
                        { $handler($req).await }
                },
            )*
            _ => $fallback()
        }
    }};
}

pub async fn router(mut req: ServerRequest) -> Result<ServerResponse, std::convert::Infallible> {
    let path = req.uri().path().to_string();
    let path_segments: Vec<_> = path.split('/').filter(|x| !x.is_empty()).collect();
    let path_segments = &path_segments[..];

    let resp = router!(req, path_segments, {
        (GET, ["static", ..]) => ([], statics::static_file),

        (GET, []) => ([], |_| async { redirect(RedirectType::Permanent, "/system") }),

        (GET, ["system"]) => ([extract_backends], pages::system::page),
        (GET, ["system", "meters"]) => ([extract_backends], pages::system::meters),

        (GET, ["set-backend"]) => ([extract_query::<pages::misc::SetBackendQuery>], pages::misc::set_backend),

        _ => not_found,
    });

    Ok(resp)
}
