use crate::types::{Packet, PacketId};
use std::fmt::{Debug, Formatter};

pub struct ConnectedPong {
    pub client_send_time_be: u64,
}

impl Debug for ConnectedPong {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ client_send_time_be: {} }}", self.client_send_time_be)
    }
}

impl Packet for ConnectedPong {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(9);

        serialized[0] = PacketId::ConnectedPong as u8;
        serialized.extend_from_slice(&u64::to_be_bytes(self.client_send_time_be));

        serialized
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 8 {
            return Err("Invalid data length".to_string());
        }

        let client_send_time_be = u64::from_be_bytes(data[0..8].try_into().unwrap());
        Ok(ConnectedPong { client_send_time_be })


    }
}
