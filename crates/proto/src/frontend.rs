use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

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
    Processes,
    Host,
    Software,
}

#[derive(Debug, Encode, Decode)]
pub enum NoIdFrontendMessage {
    Terminal(Vec<u8>),
    Signal(SignalAction),
}

#[derive(Debug, Encode, Decode, Deserialize)]
pub struct SignalAction {
    pub pid: u32,
    pub signal: Signal,
}

#[derive(Debug, Encode, Decode, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Signal {
    Term,
    Pause,
    Resume,
    Kill,
}
