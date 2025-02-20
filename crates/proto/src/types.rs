use bitcode::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub struct Handshake {
    pub nickname: String,
}

// #[derive(Debug, Clone, Copy)]
// pub enum ExpectedResponseType {
//     NoResponse,
//     Oneshot,
// }

#[derive(Debug, Encode, Decode)]
pub enum Request {
    System,
}

// impl Request {
//     pub fn expected_response_type(&self) -> ExpectedResponseType {
//         match self {
//             Self::System => ExpectedResponseType::Oneshot,
//         }
//     }
// }

#[derive(Debug, Encode, Decode)]
pub struct SystemResponse {
    pub cpu: f32,
    pub ram: UsageData,
    pub swap: UsageData,
}

#[derive(Debug, Encode, Decode)]
pub struct UsageData {
    pub used: u64,
    pub total: u64,
    pub percent: f32,
}
