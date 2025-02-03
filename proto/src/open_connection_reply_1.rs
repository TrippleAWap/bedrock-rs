use std::fmt::{Debug, Formatter};
use crate::types::{Packet, PacketId, UNCONNECTED_MESSAGE_SEQUENCE};

pub struct OpenConnectionReply1 {
    pub server_guid_be: u64,
    pub server_has_security: bool,
    pub cookie: u32,
    pub max_transmission_unit_be: usize,
}

impl Debug for OpenConnectionReply1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenConnectionReply1 {{ server_guid_be: {}, server_has_security: {}, cookie: {}, max_transmission_unit_be: {} }}", self.server_guid_be, self.server_has_security, self.cookie, self.max_transmission_unit_be)
    }
}

impl Packet for OpenConnectionReply1 {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity((28 + (self.server_has_security as u8) * 4) as usize);
        result.push(PacketId::OpenConnectionReply1 as u8);

        result.extend_from_slice(&UNCONNECTED_MESSAGE_SEQUENCE);
        result.extend_from_slice(&(self.server_guid_be).to_be_bytes());
        result.push(self.server_has_security as u8);
        if self.server_has_security {
            result.extend_from_slice(&self.cookie.to_be_bytes());
        }
        result.extend_from_slice(&(self.max_transmission_unit_be as u64).to_be_bytes());

        result
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 27 || data.len() < 27+data[24] as usize*4 {
            return Err("Invalid OpenConnectionReply1 packet".to_string());
        }
        let server_guid_be = u64::from_be_bytes(data[16..24].try_into().expect("Slice with incorrect length"));
        let server_has_security = data[24] != 0;
        let mut cookie = 0;
        if server_has_security {
            cookie = u32::from_be_bytes(data[25..29].try_into().unwrap());
        }
        let max_transmission_unit_be = usize::from_be_bytes(data[25+((server_has_security as u8) * 4) as usize..data.len()-1].try_into().unwrap());

        Ok(OpenConnectionReply1 {
            server_guid_be,
            server_has_security,
            cookie,
            max_transmission_unit_be,
        })
    }
}