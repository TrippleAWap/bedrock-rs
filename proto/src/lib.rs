#[allow(dead_code)]
const MTU_SIZE: u16 = 1492;
#[allow(dead_code)]
const UDP_HEADER_SIZE: u8= 	28;
#[allow(dead_code)]
const PUBLIC_KEY_SIZE: u16= 	294;
#[allow(dead_code)]
const REQUEST_CHALLENGE_SIZE: u8= 	64;
#[allow(dead_code)]
const RESPONDING_ENCRYPTION_KEY: u8= 	128;
#[allow(dead_code)]
const MAX_NUMBER_OF_LOCAL_ADDRESSES: u8= 	10;
#[allow(dead_code)]
const IDENTITY_PROOF_SIZE: u16= 	294;
#[allow(dead_code)]
const CLIENT_PROOF_SIZE: u8= 	32;
#[allow(dead_code)]
const DEFAULT_PROTOCOL_VERSION: u8= 	6;
#[allow(dead_code)]
const NUMBER_OF_ARRANGED_STREAMS: u8= 	32;

pub mod types;
pub mod unknown;
pub mod unconnected_ping;
pub mod unconnected_pong;
pub mod address;
pub mod base_packet;
pub mod motd;
pub mod open_connection_request_1;
pub mod open_connection_reply_1;
pub mod open_connection_request_2;
pub mod open_connection_reply_2;
pub mod connected_ping;
pub mod connected_pong;

use std::ops::BitAnd;
use types::{Packet, PacketId};
use unknown::UnknownPacket;
use unconnected_ping::UnconnectedPing;
use unconnected_pong::UnconnectedPong;
use open_connection_request_1::OpenConnectionRequest1;
use open_connection_request_2::OpenConnectionRequest2;
use open_connection_reply_1::OpenConnectionReply1;
use open_connection_reply_2::OpenConnectionReply2;
use connected_ping::ConnectedPing;
use connected_pong::ConnectedPong;
use crate::base_packet::PacketBitFlags;

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
pub fn ReadPacket(data: &[u8]) -> Result<PacketT, String> {
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


pub fn Recieve(data: &[u8]) -> Result<PacketT, String> {
    if data[0]&PacketBitFlags::ACK as u8 != 0 {
        println!("ACK packet received");
    } else if data[0]&PacketBitFlags::NACK as u8 != 0 {
        println!("Data packet received");
    } else if data[0]&PacketBitFlags::Datagram as u8 != 0 {
        println!("Datagram packet received");
    } else {
        println!("Unknown packet received");
    }
    ReadPacket(data)
}
