pub const MTU_SIZE: u16 = 1492;
pub const MAX_WINDOW_SIZE: u16 = 2048;
pub const UDP_HEADER_SIZE: u8 = 28;
pub const PUBLIC_KEY_SIZE: u16 = 294;
pub const REQUEST_CHALLENGE_SIZE: u8 = 64;
pub const RESPONDING_ENCRYPTION_KEY: u8 = 128;
pub const MAX_NUMBER_OF_LOCAL_ADDRESSES: u8 = 10;
pub const IDENTITY_PROOF_SIZE: u16 = 294;
pub const CLIENT_PROOF_SIZE: u8 = 32;
pub const DEFAULT_PROTOCOL_VERSION: u8 = 11;
pub const NUMBER_OF_ARRANGED_STREAMS: u8 = 32;

pub mod types;
pub mod address;
pub mod packet;
pub mod motd;
pub mod messages;
pub mod conn;
pub mod frame;
mod packet_queue;
mod dynamic_queue;

use std::net::Shutdown::Read;
use types::{Packet, PacketId};
use messages::unknown::UnknownPacket;
use messages::unconnected_ping::UnconnectedPing;
use messages::unconnected_pong::UnconnectedPong;
use messages::open_connection_request_1::OpenConnectionRequest1;
use messages::open_connection_request_2::OpenConnectionRequest2;
use messages::open_connection_reply_1::OpenConnectionReply1;
use messages::open_connection_reply_2::OpenConnectionReply2;
use messages::connected_ping::ConnectedPing;
use messages::connected_pong::ConnectedPong;
use crate::packet::{PacketBitFlags, handle_ack, handle_datagram, handle_nack};

#[derive(Debug)]
pub enum PacketT {
    ConnectedPing(ConnectedPing),
    ConnectedPong(ConnectedPong),

    UnconnectedPing(UnconnectedPing),
    UnconnectedPong(UnconnectedPong),

    OpenConnectionRequest1(OpenConnectionRequest1),
    OpenConnectionRequest2(OpenConnectionRequest2),

    OpenConnectionReply1(OpenConnectionReply1),
    OpenConnectionReply2(OpenConnectionReply2),

    Unknown(UnknownPacket),
}

#[allow(non_snake_case)]
/// Parses the packet data directly without handling the packet type and flags.
/// This should never be used in practice, instead use `ReceivePacket` which handles packet type and flags.
/// Example usage:
/// ```
///     let (len, src) = socket.recv_from(&mut buf).await?;
///
///     // wont correctly parse datagrams, ack or nack packets
///     let packet = ReadPacket(&buf[..len]);
///     match packet {
///         Ok(packet) => {
///             println!("Received packet: {:#?}", packet);
///         },
///         Err(err) => {
///             println!("Error parsing packet: {}", err);
///         }
///     }
/// ```
pub fn ReadPacket(data: &[u8]) -> Result<Option<PacketT>, String> {
    let packetId = &data[0].into();

    let packetData = &data[1..];
    match packetId {
        &PacketId::ConnectedPing => {
            ConnectedPing::deserialize(packetData)
                .map(|packet| PacketT::ConnectedPing(packet))
                .map_err(|err| format!("Error deserializing ConnectedPing packet: {:?}", err.to_string()))
        },
        &PacketId::UnconnectedPing => {
            UnconnectedPing::deserialize(packetData)
                .map(|packet| PacketT::UnconnectedPing(packet))
                .map_err(|err| format!("Error deserializing UnconnectedPing packet: {:?}", err.to_string()))
        },
        &PacketId::UnconnectedPong => {
            UnconnectedPong::deserialize(packetData)
                .map(|packet| PacketT::UnconnectedPong(packet))
                .map_err(|err| format!("Error deserializing UnconnectedPong packet: {:?}", err.to_string()))
        },
        &PacketId::OpenConnectionRequest1 => {
            OpenConnectionRequest1::deserialize(packetData)
                .map(|packet| PacketT::OpenConnectionRequest1(packet))
                .map_err(|err| format!("Error deserializing OpenConnectionRequest1 packet: {:?}", err.to_string()))
        }
        &PacketId::OpenConnectionRequest2 => {
            OpenConnectionRequest2::deserialize(packetData)
                .map(|packet| PacketT::OpenConnectionRequest2(packet))
                .map_err(|err| format!("Error deserializing OpenConnectionRequest2 packet: {:?}", err.to_string()))
        }
        _ => {
            UnknownPacket::deserialize(data)
                .map(|packet| PacketT::Unknown(packet))
                .map_err(|err| format!("Error deserializing Unknown packet: {:?}", err.to_string()))
        },
    }
}
/// Parses the packet data and handles the packet type and flags.
/// This is the `recommended` function in practice.
/// Example usage:
/// ```
///     let (len, src) = socket.recv_from(&mut buf).await?;
///
///     // will correctly parse datagrams, nack and acks.
///     let packet = ReceivePacket(&buf[..len]);
///     match packet {
///         Ok(packet) => {
///             println!("Received packet: {:#?}", packet);
///         },
///         Err(err) => {
///             println!("Error parsing packet: {}", err);
///         }
///     }
/// ```
#[allow(non_snake_case)]
pub fn ReceivePacket(data: &[u8]) -> Result<Option<PacketT>, String> {
    if data[0]&PacketBitFlags::ACK as u8 != 0 {
        handle_ack(&data)
    } else if data[0]&PacketBitFlags::NACK as u8 != 0 {
        handle_nack(&data)
    } else if data[0]&PacketBitFlags::Datagram as u8 != 0 {
        handle_datagram(&data)
    } else {
        ReadPacket(&data)
    }
}
