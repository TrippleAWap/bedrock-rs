use std::fmt::{Debug, Formatter};
use crate::types::Packet;

pub struct UnknownPacket {
    pub id: u8,
    pub data: Vec<u8>,
}

impl Debug for UnknownPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnknownPacket {{ id: {}, data: [{}] }}", self.id, self.data.iter().map(|x| format!("0x{:02X}", x)).collect::<Vec<_>>().join(", "))
    }
}

impl Packet for UnknownPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(1 + self.data.len());
        serialized.push(self.id);

        serialized.extend_from_slice(&self.data);

        serialized
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        let id = data[0];
        let data_vec = data[1..].to_vec();

        Ok(UnknownPacket { id, data: data_vec })
    }
}