use maud::{Markup, html};
use pretty_bytes_typed::{pretty_bytes, pretty_bytes_binary};
use proto::backend::{CpuResponse, DiskResponse, MemResponse, NetworkResponse, TempResponse};

use super::graph::{Axis, SvgGraph};

fn calc_percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.;
    };

    let percent = used as f32 / total as f32 * 100.;
    // Round percent to 2 decimal places
    (percent * 100.).round() / 100.
}

fn calc_grid_span(num_elts: usize) -> usize {
    // Starting at two rows, we need roughly 1 row for every 2 elements
    num_elts.div_ceil(2) + 1
}

pub fn cpu_meters(cpu_data: &CpuResponse, temp_data: &TempResponse) -> Markup {
    let cpu_iter = cpu_data.cpus.iter().zip(1_u8..);

    // Add 1 row to account for CPU temperature and global CPU
    let span = calc_grid_span(cpu_data.cpus.len()) + 1;

    html! {
        section .{"span-" (span)} {
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
    }
}

pub fn cpu_graph(
    data: &CpuResponse,
    old_points: impl Iterator<Item = f32> + Clone,
) -> (Markup, impl Iterator<Item = f32>) {
    let mut graph = SvgGraph::new(Axis::Percent);

    let points = std::iter::once(data.global_cpu).chain(old_points).take(20);
    graph.add_series(points.clone(), "var(--green-6)");

    let content = html! {
        section .span-3
        {
            h2 { "CPU Graph" }
            (graph)
        }
    };

    (content, points)
}

pub fn temp_graph(
    data: &TempResponse,
    old_points: impl Iterator<Item = f32> + Clone,
) -> Option<(Markup, impl Iterator<Item = f32>)> {
    data.temp.map(|temp| {
        let mut graph = SvgGraph::new(Axis::Temp);

        let points = std::iter::once(temp).chain(old_points).take(20);
        graph.add_series(points.clone(), "light-dark(#000, #fff)");

        let content = html! {
                section .span-3
                {
                    h2 { "Temperature Graph" }
                    (graph)
                }
        };

        (content, points)
    })
}

pub fn mem_meters(data: &MemResponse) -> Markup {
    let pretty_ram_used = pretty_bytes_binary(data.ram.used, Some(2));
    let pretty_ram_total = pretty_bytes_binary(data.ram.total, Some(2));
    let ram_percent = calc_percent(data.ram.used, data.ram.total);

    let pretty_swap_used = pretty_bytes_binary(data.swap.used, Some(2));
    let pretty_swap_total = pretty_bytes_binary(data.swap.total, Some(2));
    let swap_percent = calc_percent(data.swap.used, data.swap.total);

    html! {
        section .span-2 {
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
    }
}

pub fn mem_graph(
    data: &MemResponse,
    ram_points: impl Iterator<Item = f32> + Clone,
    swap_points: impl Iterator<Item = f32> + Clone,
) -> (Markup, impl Iterator<Item = f32>, impl Iterator<Item = f32>) {
    let mut graph = SvgGraph::new(Axis::Percent);

    let ram_percent = calc_percent(data.ram.used, data.ram.total);
    let swap_percent = calc_percent(data.swap.used, data.swap.total);

    let ram_points = std::iter::once(ram_percent).chain(ram_points).take(20);
    let swap_points = std::iter::once(swap_percent).chain(swap_points).take(20);

    graph.add_series(ram_points.clone(), "var(--red-6)");
    graph.add_series(swap_points.clone(), "var(--blue-6)");

    let content = html! {
        section .span-3 {
            h2 { "Memory Graph" }
            (graph)
        }
    };

    (content, ram_points, swap_points)
}

pub fn disk_meters(data: &DiskResponse) -> Markup {
    let span = calc_grid_span(data.disks.len());

    html! {
        section .{"span-" (span)} {
            h2 { "Disk Usage" }

            @for disk in &data.disks {
                @let pretty_disk_used = pretty_bytes(disk.usage.used, Some(2));
                @let pretty_disk_total = pretty_bytes(disk.usage.total, Some(2));
                @let disk_percent = calc_percent(disk.usage.used, disk.usage.total);

                p { (disk.name) " (" (disk.mnt_point) "): " (pretty_disk_used) " / " (pretty_disk_total) }
                .meter-container {
                    .bar.-disk style={"--scale:"(disk_percent / 100.)} {}
                }
            }
        }
    }
}

pub fn net_graph(
    data: &NetworkResponse,
    sent_points: impl Iterator<Item = f32> + Clone,
    recv_points: impl Iterator<Item = f32> + Clone,
) -> (Markup, impl Iterator<Item = f32>, impl Iterator<Item = f32>) {
    let mut graph = SvgGraph::new(Axis::Bytes);

    let sent_points = std::iter::once(data.sent as f32)
        .chain(sent_points)
        .take(20);
    let recv_points = std::iter::once(data.recv as f32)
        .chain(recv_points)
        .take(20);

    graph.add_series(sent_points.clone(), "var(--purple-6)");
    graph.add_series(recv_points.clone(), "var(--pink-6)");

    let content = html! {
        section .span-3 {
            h2 { "Network Graph" }
            (graph)
        }
    };

    (content, sent_points, recv_points)
}
