use maud::html;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::template;

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let content = html! {
        section {
            h2 { "Terminal" }
            web-terminal {}
        }
    };

    template(&req, content)
}
