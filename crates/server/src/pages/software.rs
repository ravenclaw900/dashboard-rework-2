use maud::{Markup, html};
use proto::backend::SoftwareInfo;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{fetch_data, template};

fn software_table(list: &[SoftwareInfo]) -> Markup {
    html! {
        table {
            tr {
                th { "ID" }
                th { "Name" }
                th { "Description" }
                th { "Dependencies" }
                th { "Docs" }
            }
            @for item in list {
                tr {
                    td { (item.id) }
                    td { (item.name) }
                    td { (item.desc) }
                    td { (item.deps) }
                    td {
                        @if item.docs.starts_with("https://") {
                            a href=(item.docs) { (item.docs) }
                        } @else {
                            (item.docs)
                        }
                    }
                }
            }
        }
    }
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = fetch_data!(req, Software)?;

    let content = html! {
        section {
            h2 { "Installed Software" }
            (software_table(&data.installed))
        }
        section {
            h2 { "Not Installed Software" }
            (software_table(&data.uninstalled))
        }
    };

    template(&req, content)
}
