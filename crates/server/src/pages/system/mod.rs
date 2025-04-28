use maud::html;
use serde::{Deserialize, Serialize};

use crate::http::{
    request::{QueryArray, ServerRequest},
    response::ServerResponse,
};

use super::template::{fetch_data, template};

mod fragments;
mod graph;

#[derive(Serialize, Deserialize, Clone)]
pub struct SystemQuery {
    #[serde(default)]
    cpu_points: QueryArray,
    #[serde(default)]
    temp_points: QueryArray,
    #[serde(default)]
    ram_points: QueryArray,
    #[serde(default)]
    swap_points: QueryArray,
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let query: SystemQuery = req.extract_query()?;

    let cpu_data = fetch_data!(req, Cpu)?;
    let temp_data = fetch_data!(req, Temp)?;
    let mem_data = fetch_data!(req, Mem)?;
    let disk_data = fetch_data!(req, Disk)?;

    let cpu_meters = fragments::cpu_meters(&cpu_data, &temp_data);
    let mem_meters = fragments::mem_meters(&mem_data);
    let disk_meters = fragments::disk_meters(&disk_data);

    let (cpu_graph, cpu_points) = fragments::cpu_graph(&cpu_data, query.cpu_points.to_iter());
    let (temp_graph, temp_points) =
        fragments::temp_graph(&temp_data, query.temp_points.to_iter()).unzip();
    let (mem_graph, ram_points, swap_points) = fragments::mem_graph(
        &mem_data,
        query.ram_points.to_iter(),
        query.swap_points.to_iter(),
    );

    let temp_points = temp_points.into_iter().flatten();

    let new_query = SystemQuery {
        cpu_points: QueryArray::from_iter(cpu_points),
        temp_points: QueryArray::from_iter(temp_points),
        ram_points: QueryArray::from_iter(ram_points),
        swap_points: QueryArray::from_iter(swap_points),
    };

    let new_query = serde_urlencoded::to_string(new_query).unwrap();

    let content = html! {
        server-swap .card-grid action={"/system?" (new_query)} trigger="delay" {
            (cpu_meters)
            (cpu_graph)
            @if let Some(temp_graph) = temp_graph {
                (temp_graph)
            }
            (mem_meters)
            (mem_graph)
            (disk_meters)
        }
    };

    template(&req, content)
}
