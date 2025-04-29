use bitcode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode)]
pub struct Handshake {
    pub nickname: String,
    pub version: u32,
}

#[derive(Debug, Encode, Decode)]
pub struct FrontendMessage {
    pub id: Option<u16>,
    pub data: FrontendMessageType,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum FrontendMessageType {
    Cpu,
    Temp,
    Mem,
    Disk,
    NetIO,
}

#[derive(Debug, Encode, Decode)]
pub struct BackendMessage {
    pub id: Option<u16>,
    pub data: BackendMessageType,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum BackendMessageType {
    Handshake(Handshake),
    Cpu(CpuResponse),
    Temp(TempResponse),
    Mem(MemResponse),
    Disk(DiskResponse),
    NetIO(NetworkResponse),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CpuResponse {
    pub global_cpu: f32,
    pub cpus: Vec<f32>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TempResponse {
    pub temp: Option<f32>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MemResponse {
    pub ram: UsageData,
    pub swap: UsageData,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UsageData {
    pub used: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DiskResponse {
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DiskInfo {
    pub name: String,
    pub mnt_point: String,
    pub usage: UsageData,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct NetworkResponse {
    pub sent: u64,
    pub recv: u64,
}

#[derive(Debug, Encode, Decode)]
pub enum OneshotResponseType {}
