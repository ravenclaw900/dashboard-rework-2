use bitcode::{Decode, Encode};

use crate::FrameData;

#[derive(Debug, Encode, Decode)]
pub struct Handshake {
    pub nickname: String,
    pub version: u32,
}

pub struct DataRequest {
    pub id: u16,
    pub data: DataRequestType,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum DataRequestType {
    Cpu,
    Temp,
    Mem,
    Disk,
}

pub struct DataResponse {
    pub id: u16,
    pub data: DataResponseType,
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum DataResponseType {
    Cpu(CpuResponse),
    Temp(TempResponse),
    Mem(MemResponse),
    Disk(DiskResponse),
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

impl FrameData for DataRequest {
    fn from_data(id: u16, data: &[u8]) -> Result<Self, bitcode::Error> {
        let data: DataRequestType = bitcode::decode(data)?;

        Ok(Self { id, data })
    }

    fn to_data(&self) -> (u16, Vec<u8>) {
        let data = bitcode::encode(&self.data);

        (self.id, data)
    }
}

impl FrameData for DataResponse {
    fn from_data(id: u16, data: &[u8]) -> Result<Self, bitcode::Error> {
        let data: DataResponseType = bitcode::decode(data)?;

        Ok(Self { id, data })
    }

    fn to_data(&self) -> (u16, Vec<u8>) {
        let data = bitcode::encode(&self.data);

        (self.id, data)
    }
}

impl FrameData for Handshake {
    fn from_data(_: u16, data: &[u8]) -> Result<Self, bitcode::Error> {
        let data: Self = bitcode::decode(data)?;

        Ok(data)
    }

    fn to_data(&self) -> (u16, Vec<u8>) {
        let data = bitcode::encode(self);

        (0, data)
    }
}
