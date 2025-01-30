use std::fmt::{Debug, Formatter};
use crate::types::{read_be_u64, Packet, PacketId, UNCONNECTED_MESSAGE_SEQUENCE};

pub struct UnconnectedPong {
    pub client_send_time_be: u64,
    pub server_guid_be: u64,
    pub data: Vec<u8>,
}
impl Debug for UnconnectedPong {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ client_send_time_be: {:?}, server_guid_be: {:?}, data: {:?} }}", self.client_send_time_be, self.server_guid_be, self.data)
    }
}

impl Packet for UnconnectedPong {

    fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.reserve_exact(35+self.data.len());

        serialized.extend_from_slice(&self.client_send_time_be.to_be_bytes());
        serialized.extend_from_slice(&self.server_guid_be.to_be_bytes());
        serialized.extend_from_slice(&UNCONNECTED_MESSAGE_SEQUENCE);
        serialized.extend(&u16::to_be_bytes(self.data.len() as u16));
        serialized.extend_from_slice(&self.data);

        serialized
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 32 {
            return Err("Invalid data length".to_string());
        }
        let client_send_time_be = read_be_u64(&data);
        let client_guid_be = read_be_u64(&data[24..]);

        Ok(Self {
            client_send_time_be,
            server_guid_be: client_guid_be,
            data: data[32..].to_vec(),
        })
    }

    fn new(_data: Vec<u8>) -> Self where Self: Sized {
        Self {
            client_send_time_be: 0,server_guid_be: 0,data: vec![]
        }
    }
}
