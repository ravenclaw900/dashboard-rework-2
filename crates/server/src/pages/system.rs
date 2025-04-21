use maud::html;
use pretty_bytes_typed::{pretty_bytes, pretty_bytes_binary};
use proto::types::{DataRequestType, DataResponseType};

use crate::http::{
    request::{QueryArray, ServerRequest},
    response::ServerResponse,
};

use super::template::{fetch_data, template};

fn calc_percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.;
    };

    let percent = used as f32 / total as f32 * 100.;
    // Round percent to 2 decimal places
    (percent * 100.).round() / 100.
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let content = html! {
        section fx-action="/system/cpu-meters" fx-trigger="fx:inited" { "Loading..." }
        section fx-action="/system/cpu-graph" fx-trigger="fx:inited" { "Loading..." }
        section fx-action="/system/temp-graph" fx-trigger="fx:inited" { "Loading..." }
        section fx-action="/system/mem-meters" fx-trigger="fx:inited" { "Loading..." }
        section fx-action="/system/mem-graph" fx-trigger="fx:inited" { "Loading..." }
    };

    template(&req, content)
}

pub async fn cpu_meters(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let cpu_data = fetch_data!(req, Cpu)?;
    let temp_data = fetch_data!(req, Temp)?;

    let cpu_iter = cpu_data.cpus.iter().zip(1_u8..);

    let span = (cpu_data.cpus.len() + 1) / 2 + 2;

    let content = html! {
        section .{"span-" (span)} fx-action="/system/cpu-meters" fx-trigger="poll" {
            h2 { "CPU Statistics" }
            @if let Some(temp) = temp_data.temp {
                p { "CPU Temperature: " (temp) "ºC" }
            }
            p { "Global CPU: " (cpu_data.global_cpu) "%" }
            div .meter-container {
                div .bar.-cpu style={"--scale:"(cpu_data.global_cpu / 100.)} {}
            }
            @for (usage, num) in cpu_iter {
                p { "CPU "(num)": "(usage)"%" }
                div .meter-container {
                    div .bar.-cpu style={"--scale:"(usage / 100.)} {}
                }
            }
        }
    };

    template(&req, content)
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct GraphQuery {
    #[serde(default)]
    points: QueryArray,
}

pub async fn cpu_graph(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let query: GraphQuery = req.extract_query()?;
    let data = fetch_data!(req, Cpu)?;

    let mut graph = SvgGraph::new(Axis::Percent);

    let points = query.points.to_iter();
    let points = std::iter::once(data.global_cpu).chain(points.take(19));
    graph.add_series(points.clone(), "var(--green-6)");

    let query_str = serde_urlencoded::to_string(GraphQuery {
        points: QueryArray::from_iter(points),
    })
    .unwrap();

    let content = html! {
        section .span-3 fx-action={"/system/cpu-graph?" (query_str)} fx-trigger="poll"
        {
            h2 { "CPU Graph" }
            (graph)
        }
    };

    template(&req, content)
}

pub async fn temp_graph(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let query: GraphQuery = req.extract_query()?;
    let data = fetch_data!(req, Temp)?;

    let content = if let Some(temp) = data.temp {
        let mut graph = SvgGraph::new(Axis::Temp);

        let points = query.points.to_iter();
        let points = std::iter::once(temp).chain(points.take(19));
        graph.add_series(points.clone(), "light-dark(#000, #fff)");

        let query_str = serde_urlencoded::to_string(GraphQuery {
            points: QueryArray::from_iter(points),
        })
        .unwrap();

        html! {
            section .span-3 fx-action={"/system/temp-graph?" (query_str)} fx-trigger="poll"
            {
                h2 { "Temperature Graph" }
                (graph)
            }
        }
    } else {
        html! {}
    };

    template(&req, content)
}

pub async fn mem_meters(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let data = fetch_data!(req, Mem)?;

    let pretty_ram_used = pretty_bytes_binary(data.ram.used, Some(2));
    let pretty_ram_total = pretty_bytes_binary(data.ram.total, Some(2));
    let ram_percent = calc_percent(data.ram.used, data.ram.total);

    let pretty_swap_used = pretty_bytes_binary(data.swap.used, Some(2));
    let pretty_swap_total = pretty_bytes_binary(data.swap.total, Some(2));
    let swap_percent = calc_percent(data.swap.used, data.swap.total);

    let content = html! {
        section .span-2 fx-action="/system/mem-meters" fx-trigger="poll" {
            h2 { "Memory Usage" }

            p { "RAM Usage: " (pretty_ram_used) " / " (pretty_ram_total) }
            div .meter-container {
                div .bar.-ram style={"--scale:"(ram_percent / 100.)} {}
            }

            p { "Swap Usage: " (pretty_swap_used) " / " (pretty_swap_total) }
            div .meter-container {
                div .bar.-swap style={"--scale:"(swap_percent / 100.)} {}
            }
        }
    };

    template(&req, content)
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct MemGraphQuery {
    #[serde(default)]
    ram_points: QueryArray,
    #[serde(default)]
    swap_points: QueryArray,
}

pub async fn mem_graph(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    let query: MemGraphQuery = req.extract_query()?;
    let data = fetch_data!(req, Mem)?;

    let mut graph = SvgGraph::new(Axis::Percent);

    let ram_percent = calc_percent(data.ram.used, data.ram.total);
    let swap_percent = calc_percent(data.swap.used, data.swap.total);

    let ram_points = query.ram_points.to_iter();
    let ram_points = std::iter::once(ram_percent).chain(ram_points).take(20);

    let swap_points = query.swap_points.to_iter();
    let swap_points = std::iter::once(swap_percent).chain(swap_points).take(20);

    graph.add_series(ram_points.clone(), "var(--red-6)");
    graph.add_series(swap_points.clone(), "var(--blue-6)");

    let query_str = serde_urlencoded::to_string(MemGraphQuery {
        ram_points: QueryArray::from_iter(ram_points.clone()),
        swap_points: QueryArray::from_iter(swap_points.clone()),
    })
    .unwrap();

    let content = html! {
        section .span-3 fx-action={"/system/mem-graph?" (query_str)} fx-trigger="poll" {
            h2 { "Memory Graph" }

            (graph)
        }
    };

    template(&req, content)
}

struct GraphSeries {
    points: Vec<(u32, f32)>,
    color: String,
}

#[derive(Clone, Copy)]
enum Axis {
    Percent,
    Temp,
    Bytes,
}

impl Axis {
    fn get_labels(self) -> [String; 11] {
        let generator_fn = match self {
            Self::Percent => |x| format!("{}%", 10 * x),
            Self::Temp => |x| format!("{}ºC", 10 * x + 20),
            Self::Bytes => |x| pretty_bytes(10_u64.pow(x as u32), None).to_string(),
        };

        std::array::from_fn(generator_fn)
    }

    // Translates data from [min, max] to [0, 100]
    // Percent: [0, 100]
    // Temp: [20, 120]
    // Bytes: [1, 10^10] (log)
    fn interpolate(self, data: f32) -> f32 {
        match self {
            Self::Percent => data,
            Self::Temp => data - 20.,
            Self::Bytes => 10. * data.log10(),
        }
    }
}

struct SvgGraph {
    series: Vec<GraphSeries>,
    axis: Axis,
}

impl SvgGraph {
    const H_LINES: u32 = 20;
    const V_LINES: u32 = 11;
    const LINE_SPACING: u32 = 10;

    pub fn new(axis: Axis) -> Self {
        Self {
            series: Vec::new(),
            axis,
        }
    }

    pub fn add_series(&mut self, points: impl Iterator<Item = f32>, color: &str) {
        let points = points.map(|x| self.axis.interpolate(x));

        // Creates (x, y) pairs starting from the right
        let points: Vec<_> = (0..Self::H_LINES).rev().zip(points).collect();

        let series = GraphSeries {
            points,
            color: color.to_string(),
        };

        self.series.push(series);
    }
}

impl maud::Render for SvgGraph {
    fn render(&self) -> maud::Markup {
        let left_margin = Self::LINE_SPACING * 4 / 3;
        let right_margin = Self::LINE_SPACING / 2;
        let top_margin = Self::LINE_SPACING / 2;
        let bottom_margin = Self::LINE_SPACING / 2;

        let graph_width = Self::LINE_SPACING * (Self::H_LINES - 1);
        let graph_height = Self::LINE_SPACING * (Self::V_LINES - 1);

        let x_end = left_margin + graph_width;
        let y_end = top_margin + graph_height;

        let total_width = left_margin + graph_width + right_margin;
        let total_height = top_margin + graph_height + bottom_margin;

        let view_box = format!("0 0 {total_width} {total_height}");

        let h_lines = (0..Self::H_LINES).map(|x| left_margin + Self::LINE_SPACING * x);
        let v_lines = (0..Self::V_LINES).map(|y| top_margin + Self::LINE_SPACING * y);

        let axis_ys = (0..Self::V_LINES)
            .rev()
            .map(|y| Self::LINE_SPACING * y + top_margin + 1);
        let axis = axis_ys.zip(self.axis.get_labels());

        html! {
            svg .graph viewBox=(view_box) {
                @for x in h_lines {
                    line x1=(x) y1=(top_margin) x2=(x) y2=(y_end) {}
                }
                @for (y, val) in axis {
                    text x="1" y=(y) { (val) }
                }
                @for y in v_lines {
                    line x1=(left_margin) y1=(y) x2=(x_end) y2=(y) {}
                }
                @for series in &self.series {
                    @let points = series.points.iter().map(|(x, y)| {
                        (left_margin + Self::LINE_SPACING * x, y_end as f32 - y)
                    });
                    @let polyline_points = {
                        use core::fmt::Write;
                        points.clone().fold(String::new(), |mut acc, (x, y)| {
                            let _ = write!(&mut acc, "{x},{y} ");
                            acc
                        })
                    };
                    g fill=(&series.color) {
                        @for (x, y) in points {
                            circle cx=(x) cy=(y) r="1.5" {}
                        }
                    }
                    polyline points=(polyline_points) stroke=(&series.color) fill="none" {}
                }
            }
        }
    }
}
