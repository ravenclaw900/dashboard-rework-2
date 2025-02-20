use sysinfo::System;

use proto::types::{SystemResponse, UsageData};

fn round_percent(percent: f32) -> f32 {
    (percent * 100.).round() / 100.
}

pub fn system(sys: &mut System) -> SystemResponse {
    let cpu = cpu(sys);
    let (ram, swap) = memory(sys);

    SystemResponse { cpu, ram, swap }
}

fn cpu(sys: &mut System) -> f32 {
    sys.refresh_cpu_usage();
    round_percent(sys.global_cpu_usage())
}

fn memory(sys: &mut System) -> (UsageData, UsageData) {
    // Refreshes both RAM and Swap
    sys.refresh_memory();

    let ram_used = sys.used_memory();
    let ram_total = sys.total_memory();
    let ram_percent = round_percent((ram_used as f32) / (ram_total as f32) * 100.);
    let ram = UsageData {
        used: ram_used,
        total: ram_total,
        percent: ram_percent,
    };

    let swap_used = sys.used_swap();
    let swap_total = sys.total_swap();
    let swap_percent = round_percent((swap_used as f32) / (swap_total as f32) * 100.);
    let swap = UsageData {
        used: swap_used,
        total: swap_total,
        percent: swap_percent,
    };

    (ram, swap)
}
