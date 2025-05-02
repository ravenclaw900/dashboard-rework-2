use bitcode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum FrontendMessage {
    Id(u16, IdFrontendMessage),
}

#[derive(Debug, Encode, Decode)]
pub enum IdFrontendMessage {
    Cpu,
    Temp,
    Mem,
    Disk,
    NetIO,
}
