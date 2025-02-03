use std::fmt::Debug;
pub trait Packet: Debug {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<Self, String> where Self: Sized;
}

// sequence of bytes used to identify unconnected messages
pub const UNCONNECTED_MESSAGE_SEQUENCE: [u8; 16] = [0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum PacketId {
    #[default]
    ConnectedPing                   = 0x00,
    UnconnectedPing                 = 0x01,
    UnconnectedPingOpenConnections  = 0x02,
    ConnectedPong                   = 0x03,
    DetectLostConnections           = 0x04,
    OpenConnectionRequest1          = 0x05,
    OpenConnectionReply1            = 0x06,
    OpenConnectionRequest2          = 0x07,
    OpenConnectionReply2            = 0x08,
    ConnectionRequest               = 0x09,
    ConnectionRequestAccepted       = 0x10,
    NewIncomingConnection           = 0x13,
    DisconnectNotification          = 0x15,

    IncompatibleProtocolVersion     = 0x19,

    UnconnectedPong                 = 0x1C,

    Unknown                         = 0xFF,
}

impl From<u8> for PacketId {
    fn from(value: u8) -> Self {
        match value {
            0x00 => PacketId::ConnectedPing,
            0x01 => PacketId::UnconnectedPing,
            0x02 => PacketId::UnconnectedPingOpenConnections,
            0x03 => PacketId::ConnectedPong,
            0x04 => PacketId::DetectLostConnections,
            0x05 => PacketId::OpenConnectionRequest1,
            0x06 => PacketId::OpenConnectionReply1,
            0x07 => PacketId::OpenConnectionRequest2,
            0x08 => PacketId::OpenConnectionReply2,
            0x09 => PacketId::ConnectionRequest,
            0x10 => PacketId::ConnectionRequestAccepted,
            0x13 => PacketId::NewIncomingConnection,
            0x15 => PacketId::DisconnectNotification,

            0x19 => PacketId::IncompatibleProtocolVersion,

            0x1C => PacketId::UnconnectedPong,
            
            _ => PacketId::Unknown,
        }
    }
}

pub type U24 = u32;

pub fn read_u24(data: &[u8]) -> U24 {
    (data[0] as u32) | ((data[1] as u32) << 8) | ((data[2] as u32) << 16)
}

pub fn write_u24(value: U24) -> [u8; 3] {
    [
        (value & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        ((value >> 16) & 0xff) as u8,
    ]
}

pub fn inc_u24(value: &mut U24) -> U24 {
    let result = *value;
    let _ = value.wrapping_add(1);
    result
}

pub fn read_be_u64(input: &[u8]) -> u64 {
    let (int_bytes, _) = input.split_at(size_of::<u64>());
    u64::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn read_be_u32(input: &[u8]) -> u32 {
    let (int_bytes, _) = input.split_at(size_of::<u32>());
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn read_be_u16(input: &[u8]) -> u16 {
    let (int_bytes, _) = input.split_at(size_of::<u16>());
    u16::from_be_bytes(int_bytes.try_into().unwrap())
}
