use std::fmt::{Debug, Formatter};
use crate::types::{Packet, PacketId, UNCONNECTED_MESSAGE_SEQUENCE};
pub struct OpenConnectionRequest1 {
    pub client_protocol: u8,
    pub max_transmission_unit: u16,
}

impl Debug for OpenConnectionRequest1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenConnectionRequest1 {{ client_protocol: {}, max_transmission_unit: {} }}", self.client_protocol, self.max_transmission_unit)
    }
}

impl Packet for OpenConnectionRequest1 {
    fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::with_capacity(self.max_transmission_unit as usize - 20 - 8);
        serialized.push(PacketId::OpenConnectionRequest1 as u8);

        serialized.extend_from_slice(&UNCONNECTED_MESSAGE_SEQUENCE);
        serialized.push(self.client_protocol);

        serialized
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 17 {
            return Err("Invalid data length".to_string());
        }
        // magic: 16 bytes
        let protocol_version = data[16];
        let mtu = data.len() + 20 + 8 + 1; // headers + packet id
        Ok(OpenConnectionRequest1 {
            client_protocol: protocol_version,
            max_transmission_unit: mtu as u16,
        })
    }
}