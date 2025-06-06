use std::fmt::{Debug, Formatter};
use crate::address::{addr_size, read_addr, Address};
use crate::types::{read_be_u16, Packet, PacketId, UNCONNECTED_MESSAGE_SEQUENCE};

pub struct OpenConnectionReply2 {
    pub server_guid_be: u64,
    pub client_address: Address,
    pub max_transmission_unit_be: u16,
    pub do_security: bool,
}

impl Debug for OpenConnectionReply2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenConnectionReply2 {{ server_guid_be: {:?}, client_address: {:?}, max_transmission_unit_be: {:?}, do_security: {:?} }}", self.server_guid_be, self.client_address, self.max_transmission_unit_be, self.do_security)
    }
}

impl Packet for OpenConnectionReply2 {
    fn serialize(&self) -> Vec<u8> {
        let offset = self.client_address.size() as usize;
        let mut result = Vec::with_capacity(28 + offset);
        result.push(PacketId::OpenConnectionReply2 as u8);

        result.extend_from_slice(&UNCONNECTED_MESSAGE_SEQUENCE);
        result.extend_from_slice(&(self.server_guid_be).to_be_bytes());
        result.extend_from_slice(&self.client_address.serialize());

        result.extend_from_slice(&self.max_transmission_unit_be.to_be_bytes());
        if self.do_security {
            result[27+offset] = 1;
        }
        result
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        if data.len() < 24 || data.len() < 27+addr_size(&data[24..]) as usize {
            return Err("Invalid OpenConnectionReply2 packet".to_string());
        }
        let server_guid_be = u64::from_be_bytes(data[16..24].try_into().expect("Slice with incorrect length"));
        let client_address = read_addr(&data[24..])?;
        let offset = addr_size(&data[24..]) as usize;
        let max_transmission_unit_be = read_be_u16(&data[24+offset..]);
        let do_security = data[26+offset] != 0;
        Ok(OpenConnectionReply2 {
            server_guid_be,
            client_address,
            do_security,
            max_transmission_unit_be,
        })
    }
}