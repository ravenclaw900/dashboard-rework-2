use maud::html;
use pretty_bytes_typed::pretty_bytes_binary;
use proto::types::{Request, SystemResponse};

use crate::http::{
    request::{BackendData, ServerRequest},
    response::{html, ServerResponse},
};

use super::template::template;

pub async fn page(req: ServerRequest) -> ServerResponse {
    let content = html! {
        section {
            h2 { "System Statistics" }
            div
                fx-action="/system/meters"
                fx-swap="innerHTML"
                fx-trigger="multi"
                ext-fx-multi-trigger="fx:inited poll"
                ext-fx-poll-interval="2000"
            {
                "Loading..."
            }
        }
    };

    template(req.get_extension(), content)
}

pub async fn meters(req: ServerRequest) -> ServerResponse {
    let backend_handle = &req.get_extension::<BackendData>().current_backend.1;

    let data: SystemResponse = backend_handle.send_req(Request::System).await.unwrap();

    let pretty_ram_used = pretty_bytes_binary(data.ram.used, Some(2));
    let pretty_ram_total = pretty_bytes_binary(data.ram.total, Some(2));

    let pretty_swap_used = pretty_bytes_binary(data.swap.used, Some(2));
    let pretty_swap_total = pretty_bytes_binary(data.swap.total, Some(2));

    let content = html! {
        "CPU: " (data.cpu) "%"
        div .meter-container {
            div style={"--meter-color:var(--red-6);--meter-width:" (data.cpu)"%;"} {}
        }
        br;
        "RAM usage: " (pretty_ram_used) " / " (pretty_ram_total)
        div .meter-container {
            div style={"--meter-color:var(--green-6);--meter-width:" (data.ram.percent)"%;"} {}
        }
        br;
        "Swap usage: " (pretty_swap_used) " / " (pretty_swap_total)
        div .meter-container {
            div style={"--meter-color:var(--blue-6);--meter-width:" (data.swap.percent)"%;"} {}
        }
    };

    html(content.into_string())
}
