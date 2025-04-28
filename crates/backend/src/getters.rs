use std::path::Path;

use sysinfo::{Components, Disks, System};

use proto::types::{CpuResponse, DiskInfo, DiskResponse, MemResponse, TempResponse, UsageData};

use crate::client::BackendContext;

fn round_to_2(num: f32) -> f32 {
    (num * 100.).round() / 100.
}

pub fn cpu(mut ctx: BackendContext) -> CpuResponse {
    let mut sys = ctx.system();

    sys.refresh_cpu_usage();

    let global_cpu = round_to_2(sys.global_cpu_usage());
    let cpus: Vec<f32> = sys
        .cpus()
        .iter()
        .map(|x| round_to_2(x.cpu_usage()))
        .collect();

    CpuResponse { global_cpu, cpus }
}

pub fn temp(_ctx: BackendContext) -> TempResponse {
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

pub fn memory(mut ctx: BackendContext) -> MemResponse {
    let mut sys = ctx.system();

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

pub fn disks(ctx: BackendContext) -> DiskResponse {
    let mnt_points = &ctx.config().disks;
    let mnt_points: Vec<_> = mnt_points.iter().map(Path::new).collect();

    let disks = Disks::new_with_refreshed_list();
    let disks = disks.list();

    let disks: Vec<_> = disks
        .iter()
        .filter(|disk| mnt_points.contains(&disk.mount_point()))
        .map(|disk| DiskInfo {
            name: disk.name().to_str().unwrap_or("unknown").into(),
            mnt_point: disk.mount_point().to_str().unwrap_or("unknown").into(),
            usage: UsageData {
                used: disk.total_space() - disk.available_space(),
                total: disk.total_space(),
            },
        })
        .collect();

    DiskResponse { disks }
}
