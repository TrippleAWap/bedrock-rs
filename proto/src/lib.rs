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

use types::{Packet, PacketId};
use unknown::UnknownPacket;
use unconnected_ping::UnconnectedPing;
use unconnected_pong::UnconnectedPong;

#[derive(Debug)]
pub enum PacketT {
    Unknown(UnknownPacket),
    UnconnectedPing(UnconnectedPing),
    UnconnectedPong(UnconnectedPong),
}

#[allow(non_snake_case)]
pub fn ReadPacket(data: &[u8]) -> Result<PacketT, String> {
    // let packet = match base_packet::BasePacket::read(data) {
    //     Ok(packet) => packet,
    //     Err(err) => return Err(format!("Error reading base packet: {}", err)),
    // };
    // let data = &packet.data;
    // if data.len() < 1 {
    //     return Err("Invalid packet data length".to_string());
    // }
    let packetId = &data[0].into();
    //
    // let address  = match  read_addr(&data[1..]) {
    //     Ok(addr) => addr,
    //     Err(err) => return Err(format!("Error reading address: {}", err)),
    // };
    let packetData = &data[1..];
    match packetId {
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
        &PacketId::Unknown => {
            UnknownPacket::deserialize(packetData)
                .map(|packet| PacketT::Unknown(packet))
                .map_err(|err| format!("Error deserializing Unknown packet: {:?}", err.to_string()))
        },
        _ => Err(format!("Unhandled packet type: {}", *packetId as u8)),
    }
}
