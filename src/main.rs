use std::any::Any;
use proto::types::{Packet, PacketId};
use proto::{PacketT, ReadPacket, unconnected_ping::UnconnectedPing, unconnected_pong::UnconnectedPong};
use std::env::args_os;
use tokio::net::UdpSocket;
use rand::random;

#[tokio::main]
async fn main()  -> std::io::Result<()> {
    let mut args = args_os();
    if args.len() < 2 {
        println!("Usage: {} <target_address>", args.next().unwrap().to_str().unwrap());
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid target address"));
    }
    let target_address = args.nth(1).unwrap().into_string().unwrap();
    println!("Listening on {}", target_address);

    let socket = UdpSocket::bind(target_address).await?;
    let server_id: u64 = random();
    let mut buf = [0u8; 1500];
    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let received_data = &buf[..len];
        // println!("Received data from {} | len: {} | data: {:?} ", src, received_data.len(), received_data.to_vec());

        match ReadPacket(received_data) {
            Ok(packet) => {
                println!("Received packet: {:?}", packet);
                match packet {
                    PacketT::UnconnectedPing(packet) => {
                        println!("Received unconnected ping packet, replying with unconnected pong");
                        let response =  UnconnectedPong{server_guid_be: server_id, client_send_time_be: packet.client_send_time_be, data: vec![] };
                        socket.send_to(&response.serialize(), src).await?;
                    }
                    _ => {
                        println!("Received unknown packet");
                    }
                }
            }
            Err(e) => {
                println!("{} | data: [{}]", e,  received_data.to_vec().iter().map(|&b| format!("0x{:02X}", b)).collect::<Vec<String>>().join(", "));
            }
        }

        let response = vec![PacketId::UnconnectedPong as u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F];
        socket.send_to(&response, &src).await?;
    }
}
