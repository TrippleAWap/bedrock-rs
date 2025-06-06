use proto::types::Packet;
use proto::{PacketT, ReceivePacket};
use rand::random;
use std::env::args_os;
use std::process::exit;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::UdpSocket;
use proto::messages::open_connection_reply_1::OpenConnectionReply1;
use proto::messages::open_connection_request_1::OpenConnectionRequest1;
use proto::messages::open_connection_reply_2::OpenConnectionReply2;
use proto::messages::unconnected_ping::UnconnectedPing;
use proto::messages::unconnected_pong::UnconnectedPong;
use proto::messages::connected_pong::ConnectedPong;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = args_os();
    if args.len() < 3 {
        println!("Usage: {} <target_address> <client|server|ping>", args.next().unwrap().to_str().unwrap());
        exit(1);
    }

    // skip the first arg;
    args.next();

    let target_address = args.next().unwrap().into_string().unwrap();
    println!("Target address: {}", target_address);
    let run_mode = args.next().unwrap().into_string().unwrap();
    println!("Run mode: {}", run_mode);
    match run_mode.as_str() {
        "server" => {
            println!("Starting server");
            Ok(server(target_address).await?)
        }
        "client" => {
            println!("Starting client");
            Ok(client(target_address, false).await?)
        }
        "ping" => {
            Ok(client(target_address, true).await?)
        }
        _ => {
            println!("Invalid argument: {}", run_mode);
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid argument"))
        }
    }
}
#[allow(dead_code)]
struct RateStats {
    pub tick: u64,
    pub packets_sent: u64,
}

async fn server(target_address: String) -> std::io::Result<()> {
    println!("Listening on {}", target_address);
    let socket = UdpSocket::bind(target_address).await?;
    let server_id: u64 = random();
    let mut buf = [0u8; 1492];
    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;

        let received_data = &buf[..len];
        match ReceivePacket(received_data) {
            Ok(p) => {
                if p.is_none() {
                    continue;
                }
                let packet = p.unwrap();
                println!("Received {:?}", packet);
                match packet {
                    PacketT::ConnectedPing(packet) => {
                        let response = ConnectedPong{ client_send_time_be: packet.client_send_time_be };
                        socket.send_to(&response.serialize(), src).await?;
                        println!("Sent ConnectedPong to {}", src);
                    }
                    PacketT::UnconnectedPing(packet) => {
                        let response =  UnconnectedPong{server_guid_be: server_id, client_send_time_be: packet.client_send_time_be, data: "MCPE;Dedicated Server;786;1.21.73;0;10;11954621141260796043;Bedrock level;Survival;1;19132;19133;0;".to_string() };
                        socket.send_to(&response.serialize(), src).await?;
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

                        socket.send_to(&response.serialize(), src).await?;
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

                        socket.send_to(&response.serialize(), src).await?;
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

async fn client(target_address: String, ping_only: bool) -> std::io::Result<()> {
    println!("Connecting to {}", target_address.clone());
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!("Bound to {}", socket.local_addr()?);
    let start_time = SystemTime::now();
    socket.connect(target_address.clone()).await?;
    println!("Connected to {} in {}ms", target_address, start_time.elapsed().unwrap().as_millis());
    let mut buf = [0u8; 4096];
    socket.send(&UnconnectedPing { client_guid_be: 0, client_send_time_be: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64 }.serialize()).await?;
    let (len, src) = socket.recv_from(&mut buf).await?;
    let received_data = &buf[..len];
    let mut max_mtu = 0;
    let socket = Arc::new(tokio::sync::Mutex::new(socket));
    loop {
        match ReceivePacket(received_data) {
            Ok(packet_v) => {
                if packet_v.is_none() {
                    continue
                }
                let packet = packet_v.unwrap();
                //println!("Received packet from {}    : {:?}", src, packet);
                match packet {
                    PacketT::OpenConnectionReply1(packet) => {
                        println!("Received OpenConnectionReply1 from {} with MTU {}", src, packet.max_transmission_unit_be);
                        max_mtu = packet.max_transmission_unit_be;
                    }
                    PacketT::UnconnectedPong(packet) => {
                        if ping_only {
                            println!("Received UnconnectedPong from {} [\x1b[32m{}\x1b[0m]", src, &packet.data);
                            return Ok(());
                        }
                        let socket_clone = Arc::clone(&socket);

                        _ = tokio::spawn(async move {
                            let mut response = OpenConnectionRequest1 {
                                max_transmission_unit: 1492,
                                client_protocol: 255,
                            };
                            max_mtu = response.max_transmission_unit;
                            match socket_clone.lock().await.send(&response.serialize()).await {
                                Ok(_) => {
                                    // println!("Sent OpenConnectionRequest1 with MTU {}", response.max_transmission_unit);
                                }
                                Err(e) => {
                                    println!("Failed to send OpenConnectionRequest1: {}", e);
                                }
                            }
                            for _ in 0..4 {
                                for _ in 0..4 {
                                    if max_mtu != 0 {
                                        println!("Breaking {}", max_mtu);
                                        break;
                                    }
                                    let sent_bytes= socket_clone.lock().await.send(&response.serialize()).await;
                                    if sent_bytes.is_err() {
                                        println!("Failed to send OpenConnectionRequest1");
                                        break;
                                    }
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                    // println!("Sent OpenConnectionRequest1 with MTU {}", response.max_transmission_unit);
                                }
                                response.max_transmission_unit -= 1024;
                            }
                        });
                    }
                    _ => {
                        println!("Received Unsupported packet id: 0x{:02X}", received_data[0]);
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}
