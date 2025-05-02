use std::path::PathBuf;

use proto::backend::{
    CpuResponse, DiskInfo, DiskResponse, MemResponse, NetworkResponse, TempResponse, UsageData,
};

use crate::client::BackendContext;

fn round_to_2(num: f32) -> f32 {
    (num * 100.).round() / 100.
}

pub fn cpu(mut ctx: BackendContext) -> CpuResponse {
    let sys = &mut ctx.system().system;

    sys.refresh_cpu_usage();

    let global_cpu = round_to_2(sys.global_cpu_usage());
    let cpus: Vec<f32> = sys
        .cpus()
        .iter()
        .map(|x| round_to_2(x.cpu_usage()))
        .collect();

    CpuResponse { global_cpu, cpus }
}

pub fn temp(mut ctx: BackendContext) -> TempResponse {
    let components = &mut ctx.system().components;
    components.refresh();
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
    let sys = &mut ctx.system().system;

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

pub fn disks(mut ctx: BackendContext) -> DiskResponse {
    let mnt_points = &ctx.config().disks;
    let mnt_points: Vec<_> = mnt_points.iter().map(PathBuf::from).collect();

    let disks = &mut ctx.system().disks;
    disks.refresh();
    let disks = disks.list();

    let disks: Vec<_> = disks
        .iter()
        .filter(|disk| mnt_points.iter().any(|path| path == disk.mount_point()))
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

pub fn network_io(mut ctx: BackendContext) -> NetworkResponse {
    let networks = &mut ctx.system().networks;
    networks.refresh();
    let networks = networks.list();

    let mut resp = NetworkResponse { sent: 0, recv: 0 };

    for net in networks.values() {
        resp.recv += net.received();
        resp.sent += net.transmitted();
    }

    resp
}
