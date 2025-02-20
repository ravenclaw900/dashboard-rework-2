use include_dir::{include_dir, Dir};

use super::{
    request::ServerRequest,
    response::{full_with_mime, not_found, ServerResponse},
};

static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

fn get_mime_type(ext: &str) -> &str {
    match ext {
        "js" => "text/javascript;charset=UTF-8",
        "css" => "text/css;charset=UTF-8",
        "svg" => "image/svg+xml",
        _ => unreachable!(),
    }
}

pub async fn static_file(req: ServerRequest) -> ServerResponse {
    let path = req.uri().path().trim_start_matches("/static/");

    let Some(file) = ASSETS.get_file(path) else {
        return not_found();
    };

    // Unwraps are safe here because every static file has an extension that is valid UTF-8
    let ext = file.path().extension().unwrap();
    let ext = ext.to_str().unwrap();

    let mime = get_mime_type(ext);

    full_with_mime(file.contents(), mime)
}
