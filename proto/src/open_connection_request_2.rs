use crate::address::{addr_size, read_addr, Address};
use crate::types::{read_be_u16, read_be_u32, read_be_u64, Packet};
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;


pub struct OpenConnectionRequest2 {
    pub server_address: Address,
    pub max_transmission_unit: u16,
    pub client_guid: u64,
    pub server_has_security: bool,
    pub cookie: u32,
}

impl Debug for OpenConnectionRequest2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenConnectionRequest2 {{ server_address: {:?}, max_transmission_unit: {}, client_guid: {}, server_has_security: {}, cookie: {} }}", self.server_address, self.max_transmission_unit, self.client_guid, self.server_has_security, self.cookie)
    }
}
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref CACHED_OCR1: Mutex<HashMap<usize, Vec<u8>>> = Mutex::new(HashMap::new());
}
const SERVER_HAS_SECURITY: bool = false;

impl Packet for OpenConnectionRequest2 {
    fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized {
        let mut offset = 0usize;
        if SERVER_HAS_SECURITY {
            offset = 5;
        }
        if data.len() < 16+ offset ||
            data.len() < 26 + offset + addr_size(&data[16 + offset..]) as usize {
            return Err("invalid size".to_string());
        }
        // Magic: 16 bytes.
        let cookie = if SERVER_HAS_SECURITY {
            read_be_u32(data[16..].try_into().expect("slice with incorrect length"))
        } else {
            0
        };
        let server_address = match read_addr(&data[16 + offset..]) {
            Ok(addr) => addr,
            Err(e) => return Err(e),
        };
        offset += addr_size(&data[16 + offset..]) as usize;
        let mtu = read_be_u16(data[16 + offset..].try_into().expect("slice with incorrect length"));
        let client_guid = read_be_u64(data[18 + offset..].try_into().expect("slice with incorrect length"));
        Ok(OpenConnectionRequest2 {
            server_address,
            max_transmission_unit: mtu,
            client_guid,
            server_has_security: SERVER_HAS_SECURITY,
            cookie,
        })
    }
}