use std::sync::Arc;
use rand::random;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use proto::conn::Conn;
use proto::messages::connected_pong::ConnectedPong;
use proto::messages::open_connection_reply_1::OpenConnectionReply1;
use proto::messages::open_connection_reply_2::OpenConnectionReply2;
use proto::messages::unconnected_pong::UnconnectedPong;
use proto::PacketT;
use proto::types::Packet;

pub async fn server(local_addr: String) -> std::io::Result<()> {
    println!("Listening on {}", local_addr);
    let socket = UdpSocket::bind(local_addr).await?;
    let server_id: u64 = random();
    let mut buf = [0u8; 1492];
    let socket = Arc::new(Mutex::new(socket));
    let conn = Conn::new(socket.clone(), 1492, true).await;
    loop {
        let (len, src) = socket.lock().await.recv_from(&mut buf).await?;

        let received_data = &buf[..len];
        match conn.ReceivePacket(received_data).await {
            Ok(p) => {
                if p.is_none() {
                    continue;
                }
                let packet = p.unwrap();
                println!("Received {:?}", packet);
                match packet {
                    PacketT::ConnectedPing(packet) => {
                        let response = ConnectedPong{ client_send_time_be: packet.client_send_time_be };
                        conn.WritePacketTo(Box::new(&response), src, true).await?;
                        println!("Sent ConnectedPong to {}", src);
                    }
                    PacketT::UnconnectedPing(packet) => {
                        let response =  UnconnectedPong{server_guid_be: server_id, client_send_time_be: packet.client_send_time_be, data: "MCPE;Dedicated Server;786;1.21.73;0;10;11954621141260796043;Bedrock level;Survival;1;19132;19133;0;".to_string() };
                        conn.WritePacketTo(Box::new(&response), src, true).await?;
                    }
                    PacketT::OpenConnectionRequest1(packet) => {
                        if packet.max_transmission_unit as usize > buf.len() {
                            println!("Ignoring MTU {} as it is larger than buffer size", packet.max_transmission_unit);
                            continue;
                        }

                        let response = OpenConnectionReply1{
                            server_has_security: false,
                            server_guid_be: server_id,
                            max_transmission_unit_be: packet.max_transmission_unit,
                            cookie: 0,
                        };

                        conn.WritePacketTo(Box::new(&response), src, true).await?;
                    }
                    PacketT::OpenConnectionRequest2(packet) => {
                        if packet.max_transmission_unit as usize > buf.len() {
                            continue;
                        }

                        let response = OpenConnectionReply2{
                            client_address: src.into(),
                            do_security: packet.server_has_security,
                            max_transmission_unit_be: 19200,
                            server_guid_be: server_id,
                        };

                        conn.WritePacketTo(Box::new(&response), src, true).await?;
                    }
                    _ => {
                        println!("Received Unhandled packet id: 0x{:02X}", received_data[0]);
                    }
                }
            }
            Err(e) => {
                println!("{} | data: [{}]", e,  received_data.to_vec().iter().map(|&b| format!("0x{:02X}", b)).collect::<Vec<String>>().join(", "));
            }
        }
    }
}
