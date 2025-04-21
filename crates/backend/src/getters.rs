use sysinfo::{Components, System};

use proto::types::{CpuResponse, MemResponse, TempResponse, UsageData};

fn round_to_2(num: f32) -> f32 {
    (num * 100.).round() / 100.
}

pub fn cpu(sys: &mut System) -> CpuResponse {
    sys.refresh_cpu_usage();

    let global_cpu = round_to_2(sys.global_cpu_usage());
    let cpus: Vec<f32> = sys
        .cpus()
        .iter()
        .map(|x| round_to_2(x.cpu_usage()))
        .collect();

    CpuResponse { global_cpu, cpus }
}

pub fn temp() -> TempResponse {
    let components = Components::new_with_refreshed_list();
    let components = components.list();

    let known_sensor_names = ["coretemp Package"];

    let temp = components
        .iter()
        .find(|x| known_sensor_names.iter().any(|y| x.label().contains(y)))
        .or_else(|| components.first())
        .map(|x| round_to_2(x.temperature()));

    TempResponse { temp }
}

pub fn memory(sys: &mut System) -> MemResponse {
    // Refreshes both RAM and Swap
    sys.refresh_memory();

    let ram = UsageData {
        used: sys.used_memory(),
        total: sys.total_memory(),
    };

    let swap = UsageData {
        used: sys.used_swap(),
        total: sys.total_swap(),
    };

    MemResponse { ram, swap }
}
