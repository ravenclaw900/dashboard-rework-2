use bitcode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub enum FrontendMessage {
    Id(u16, IdFrontendMessage),
    NoId(NoIdFrontendMessage),
}

#[derive(Debug, Encode, Decode)]
pub enum IdFrontendMessage {
    Cpu,
    Temp,
    Mem,
    Disk,
    NetIO,
}

#[derive(Debug, Encode, Decode)]
pub enum NoIdFrontendMessage {
    Terminal(Vec<u8>),
}
