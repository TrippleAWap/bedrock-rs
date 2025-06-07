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
