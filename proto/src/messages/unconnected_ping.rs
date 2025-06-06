use std::fmt::{Debug, Formatter};
use crate::types::{read_be_u64, Packet, PacketId};

pub struct UnconnectedPing {
    pub client_send_time_be: u64,
    pub client_guid_be: u64,
}
impl Debug for UnconnectedPing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ client_send_time_be: {:?}, client_guid_be: {:?} }}", self.client_send_time_be, self.client_guid_be)
    }
}

impl Packet for UnconnectedPing {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(33); // 32 bytes + packet id
        serialized.push(PacketId::UnconnectedPing as u8);

        serialized.extend_from_slice(&self.client_send_time_be.to_be_bytes());
        serialized.extend_from_slice(&crate::types::UNCONNECTED_MESSAGE_SEQUENCE);
        serialized.extend_from_slice(&self.client_guid_be.to_be_bytes());


        serialized
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 32 {
            return Err("Invalid data length".to_string());
        }
        let client_send_time_be = read_be_u64(&data);
        let client_guid_be = read_be_u64(&data[24..]);

        Ok(UnconnectedPing {
            client_send_time_be,
            client_guid_be,
        })
    }
}
