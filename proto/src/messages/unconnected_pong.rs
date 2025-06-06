use std::fmt::{Debug, Formatter};
use crate::types::{read_be_u64, Packet, PacketId, UNCONNECTED_MESSAGE_SEQUENCE};

pub struct UnconnectedPong {
    pub client_send_time_be: u64,
    pub server_guid_be: u64,
    pub data: String,
}
impl Debug for UnconnectedPong {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ client_send_time_be: {:?}, server_guid_be: {:?}, data: {:?} }}", self.client_send_time_be, self.server_guid_be, self.data)
    }
}

impl Packet for UnconnectedPong {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(35+self.data.len()); // 34 bytes for header + packet id + data
        serialized.push(PacketId::UnconnectedPong as u8);

        serialized.extend_from_slice(&self.client_send_time_be.to_be_bytes());
        serialized.extend_from_slice(&self.server_guid_be.to_be_bytes());
        serialized.extend_from_slice(&UNCONNECTED_MESSAGE_SEQUENCE);
        serialized.extend(&u16::to_be_bytes(self.data.len() as u16));
        serialized.extend_from_slice(&self.data.as_bytes());

        serialized
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 32 {
            return Err("Invalid data length".to_string());
        }
        let client_send_time_be = read_be_u64(&data);
        let client_guid_be = read_be_u64(&data[24..]);
        let data_size = u16::from_be_bytes(data[32..34].try_into().unwrap()) as usize;
        if data.len() < data_size {
            return Err("Invalid data length".to_string());
        }
        Ok(Self {
            client_send_time_be,
            server_guid_be: client_guid_be,
            data: data[34..34+data_size].to_vec().into_iter().map(|b| b as char).collect(),
        })
    }
}
