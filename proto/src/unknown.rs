use std::fmt::{Debug, Formatter};
use crate::types::Packet;

pub struct UnknownPacket {
    id: u8,
    data: Vec<u8>,
}

impl Debug for UnknownPacket {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnknownPacket {{ id: {}, data: [{:?}] }}", self.id, self.data.iter().map(|x| format!("{:04x}", x)).collect::<Vec<_>>().join(", "))
    }
}

impl Packet for UnknownPacket {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];

        serialized.push(self.id);
        serialized.extend_from_slice(&self.data);

        serialized
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.is_empty() {
            return Err("Data is empty".to_string());
        }

        let id = data[0];
        let data_vec = data[1..].to_vec();

        Ok(UnknownPacket { id, data: data_vec })
    }

    fn new(data: Vec<u8>) -> Self where Self: Sized {
        if data.len() < 2 {
            return UnknownPacket { id: 0, data: vec![] };
        }
        UnknownPacket { id: data[0], data: data[1..].to_vec() }
    }
}