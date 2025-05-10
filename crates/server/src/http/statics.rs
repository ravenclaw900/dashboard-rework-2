use hyper::{StatusCode, header};

use super::{request::ServerRequest, response::ServerResponse};

macro_rules! static_file {
    ($name:ident, $path:literal, $mime:literal) => {
        pub async fn $name(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
            let file = include_bytes!($path);
            let sum = include_str!(concat!($path, ".md5"));

            let client_sum = req
                .headers()
                .get(header::IF_NONE_MATCH)
                .and_then(|x| x.to_str().ok())
                .unwrap_or_default();

            if client_sum == sum {
                Ok(ServerResponse::new().status(StatusCode::NOT_MODIFIED))
            } else {
                Ok(ServerResponse::new()
                    .header(header::CONTENT_TYPE, $mime)
                    .header(header::CONTENT_ENCODING, "gzip")
                    .header(header::ETAG, sum)
                    .body(&file[..]))
            }
        }
    };
}

static_file!(js, "../../dist/main.js", "text/javascript;charset=UTF-8");
static_file!(css, "../../dist/main.css", "text/css;charset=UTF-8");
static_file!(icons, "../../dist/icons.svg", "image/svg+xml");
