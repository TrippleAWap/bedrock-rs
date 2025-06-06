use crate::types::{read_u24, uint24};
use std::cmp::min;
use lazy_static::lazy_static;
use crate::{PacketT, ReadPacket};
use crate::frame::Window;

pub enum Reliability {
    Unreliable,
    UnreliableSequenced,
    Reliable,
    ReliableOrdered,
    ReliableSequenced,

    // last value in enum, used to evaluate size of enum ( we use this to check the validity of reliability type)
    ReliabilitySize,
}

impl From<u8> for Reliability {
    fn from(value: u8) -> Self {
        match value {
            0 => Reliability::Unreliable,
            1 => Reliability::UnreliableSequenced,
            2 => Reliability::Reliable,
            3 => Reliability::ReliableOrdered,
            4 => Reliability::ReliableSequenced,
            _ => panic!("Invalid reliability type"),
        }
    }
}
pub const SPLIT_FLAG: u8 = 0x10;

pub enum PacketBitFlags {
    Datagram = 0x80,
    ACK = 0x40,
    NACK = 0x20,
    NeedsBAndAS = 0x04,
}

// BasePacket is an encapsulation around every packet sent after the connection is
// established.
pub struct Packet {
    pub reliability: Reliability,

    pub message_index: uint24,
    pub sequence_index: uint24,
    pub order_index: uint24,

    pub data: Vec<u8>,
    pub split: bool,
    pub split_count: u32,
    pub split_index: u32,
    pub split_id: u16,
}

impl Default for Packet {
    fn default() -> Self {
        Self {
            reliability: Reliability::Unreliable,
            message_index: 0,
            sequence_index: 0,
            order_index: 0,
            data: Vec::new(),
            split: false,
            split_count: 0,
            split_index: 0,
            split_id: 0,
        }
    }
}
impl Packet {
    pub fn reliable(&self) -> bool {
        match self.reliability {
            Reliability::Reliable | Reliability::ReliableOrdered | Reliability::ReliableSequenced => true,
            _ => false,
        }
    }

    pub fn sequenced(&self) -> bool {
        match self.reliability {
            Reliability::ReliableSequenced | Reliability::UnreliableSequenced => true,
            _ => false,
        }
    }

    pub fn sequenced_or_ordered(&self) -> bool {
        match self.reliability {
            Reliability::ReliableOrdered | Reliability::ReliableSequenced | Reliability::UnreliableSequenced => true,
            _ => false,
        }
    }
}

lazy_static! {
    pub static ref DATAGRAMS_WINDOW: tokio::sync::Mutex<Window> = tokio::sync::Mutex::new(Window::new());
}

// TODO: handle datagrams
// this raises an issue because datagrams are chunked packets meaning we need a new array per connection.
// how do we do this!? ( might be time to make a new struct for connections :c )
pub fn handle_datagram(data: &[u8]) -> Result<PacketT, String> {
    println!("Received datagram");

    Err("Datagram handling not implemented".to_string())
}

pub fn handle_nack(data: &[u8]) -> Result<PacketT, String> {
    ReadPacket(data)
}

pub fn handle_ack(data: &[u8]) -> Result<PacketT, String> {
    ReadPacket(data)
}

pub fn read(buf: &[u8]) -> Result<Packet, String> {
    let mut packet = Packet {
        reliability: Reliability::Unreliable,
        message_index: 0,
        sequence_index: 0,
        order_index: 0,
        data: Vec::new(),
        split: false,
        split_count: 0,
        split_index: 0,
        split_id: 0,
    };

    let header = buf[0];
    packet.split = header & SPLIT_FLAG != 0;
    packet.reliability = ((header & 224) >> 5).into();

    let packet_length = u16::from_be_bytes([buf[1], buf[2]]) >> 3;
    let mut offset = 3;

    if packet.reliable() {
        if buf.len() - offset < 3 {
            return Err("Packet too short".to_string());
        }
        packet.message_index = read_u24(&buf[offset..]);
        offset += 3;
    }
    if packet.sequenced() {
        if buf.len() - offset < 3 {
            return Err("Packet too short".to_string());
        }
        packet.sequence_index = read_u24(&buf[offset..]);
        offset += 3;
    }
    if packet.sequenced_or_ordered() {
        if buf.len() - offset < 4 {
            return Err("Packet too short".to_string());
        }
        packet.order_index = read_u24(&buf[offset..]);
        // order channel ( 1 byte, discarded )
        offset += 4;
    }

    if packet.split {
        if buf.len() - offset < 10 {
            return Err("Packet too short".to_string());
        }

        packet.split_count = u32::from_be_bytes(buf[offset..].try_into().unwrap());
        packet.split_id = u16::from_be_bytes(buf[offset + 4..].try_into().unwrap());
        packet.split_index = u32::from_be_bytes(buf[offset + 6..].try_into().unwrap());
        offset += 10;
    }

    packet.data = Vec::with_capacity(packet_length as usize);
    if buf[offset..].len() < packet_length as usize {
        return Err("Packet too short".to_string());
    }

    println!("Packet data length: {}", packet_length);
    println!("Offset: {}, packet length: {}", offset, packet_length);
    packet.data = buf[offset..offset + packet_length as usize].to_vec();

    Ok(packet)
}


// Datagram header +
// Datagram sequence number +
// Packet header +
// Packet content length +
// Packet message index +
// Packet order index +
// Packet order channel
const PACKET_ADDITIONAL_SIZE: u8 = 1 + 3 + 1 + 2 + 3 + 3 + 1;
// Packet split count +
// Packet split ID +
// Packet split index
const SPLIT_ADDITIONAL_SIZE: u8 = 4 + 2 + 4;


pub fn split_packet(data: &[u8], mtu: u16) -> Vec<Vec<u8>> {
    let size = data.len();

    let mut max_size = mtu as usize - PACKET_ADDITIONAL_SIZE as usize;
    if size > max_size {
        max_size -= SPLIT_ADDITIONAL_SIZE as usize;
    }

    let total_fragments = size / max_size + min(size % max_size, 1);

    let mut fragments = Vec::with_capacity(total_fragments);
    for i in 0..total_fragments {
        let start = i * max_size;
        let end = min(start + max_size, size);
        fragments.push(data[start..end].to_vec())
    }

    fragments
}
