use std::fmt::Debug;
use crate::types::read_be_u16;

pub const SIZEOF_ADDR4: u8 = 1 + 4 + 2;
pub const SIZEOF_ADDR6: u8 = 1 + 2 + 2 + 4 + 16 + 4;

#[derive(PartialEq, Debug)]
pub enum AddrType {
    IPv4,
    IPv6,
    Zero,
}

pub type Addr4 = [u8; 4];
pub type Addr6 = [u8; 16];

#[derive(Debug, PartialEq)]
pub enum Addr {
    Addr4(Addr4),
    Addr6(Addr6),
}

impl Addr {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Addr::Addr4(addr) => addr.to_vec(),
            Addr::Addr6(addr) => addr.to_vec(),
        }
    }
}
#[derive(Debug, PartialEq)]
pub struct Address {
    pub addr: Addr,
    pub port: u16,
    pub addr_type: AddrType,
}


impl Address {
    pub fn fmt(&self) -> String {
        format!("{}:{}", self.addr.to_bytes().iter().map(|&b| format!("{}", b as char)).collect::<String>(), self.port)
    }
    pub fn size(&self) -> u8 {
        match self.addr_type {
            AddrType::IPv4 => SIZEOF_ADDR4,
            AddrType::IPv6 => SIZEOF_ADDR6,
            AddrType::Zero => 5,
        }
    }
}

pub fn serialize_addr(addr: &Address) -> Vec<u8> {
    if addr.addr_type == AddrType::IPv4 {
        // IPv4 address.
        let addr_bytes = addr.addr.to_bytes();
       vec![4, !addr_bytes[0], !addr_bytes[1], !addr_bytes[2], !addr_bytes[3]];
    } else if addr.addr_type == AddrType::IPv6 {
        // IPv6 address.
        let mut ret = vec![6];
        ret.extend_from_slice(&23u16.to_be_bytes());
        ret.extend_from_slice(&addr.port.to_be_bytes());
        ret.extend_from_slice(&addr.addr.to_bytes());
        return ret;
    } else {
        // Special case for zero addresses.
        vec![4, 255, 255, 255, 255];
    }
    vec![]
}

pub fn read_addr(buf: &[u8]) -> Result<Address, String> {
    if buf.len() < 5 {
        return Err("Invalid address length".to_string());
    }
    let addr_type = match buf[0] {
        0 => AddrType::Zero,
        4 => AddrType::IPv4,
        _ => AddrType::IPv6,
    };
    if addr_type == AddrType::IPv6 {
        let port = read_be_u16(buf[3..].try_into().map_err(|_| "Failed to read port")?);
        let mut ip = [0u8; 16];
        ip.copy_from_slice(&buf[9..]);
        Ok(Address {
            addr: Addr::Addr6(ip),
            port,
            addr_type,
        })
    } else {
        let mut ip = [0u8; 4];
        ip[0] = !buf[1];
        ip[1] = !buf[2];
        ip[2] = !buf[3];
        ip[3] = !buf[4];
        let port = read_be_u16(buf[5..].try_into().map_err(|_| "Failed to read port")?);
        Ok(Address {
            addr: Addr::Addr4(ip),
            port,
            addr_type,
        })
    }
}

pub fn addr_size(b: &[u8]) -> u8 {
    if b.len() == 0 || b[0] == 4 || b[0] == 0 {
        return SIZEOF_ADDR4
    }
    SIZEOF_ADDR6
}