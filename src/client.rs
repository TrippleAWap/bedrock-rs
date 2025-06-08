use std::cmp::min;
use std::process::exit;
use std::sync::Arc;
use std::time::SystemTime;
use rand::random;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use proto::conn::Conn;
use proto::messages::open_connection_request_1::OpenConnectionRequest1;
use proto::messages::unconnected_ping::UnconnectedPing;
use proto::{PacketT, DEFAULT_PROTOCOL_VERSION};
use proto::types::Packet;

pub async fn client(target_address: String) -> std::io::Result<()> {
    println!("Connecting to {}", target_address.clone());
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!("Bound to {}", socket.local_addr()?);
    let start_time = SystemTime::now();
    socket.connect(target_address.clone()).await?;
    println!("Connected to {} in {}ms", target_address, start_time.elapsed().unwrap().as_millis());
    let mut buf = [0u8; 1492];
    socket.send(&UnconnectedPing { client_guid_be: random(), client_send_time_be: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64 }.serialize()).await?;
    let socket = Arc::new(Mutex::new(socket));
    let mut conn = Conn::new(socket.clone(), 1492, false).await;
    let mut max_mtu = 0;
    loop {
        let (len, src) = socket.lock().await.recv_from(&mut buf).await?;
        let received_data = &buf[..len];
        match conn.ReceivePacket(received_data).await {
            Ok(packet_v) => {
                if packet_v.is_none() {
                    continue
                }
                let packet = packet_v.unwrap();
                println!("Received packet from {}    : {:?}", src, packet);
                match packet {
                    PacketT::OpenConnectionReply1(packet) => {
                        println!("Received OpenConnectionReply1 from {} with MTU {}", src, packet.max_transmission_unit_be);
                        max_mtu = packet.max_transmission_unit_be;
                    }
                    PacketT::UnconnectedPong(_) => {
                        let mut request = OpenConnectionRequest1 {
                            max_transmission_unit: 0, // this gets set in the loop below.
                            client_protocol: DEFAULT_PROTOCOL_VERSION,
                        };
                        for i in 0..3 {
                            if max_mtu != 0 {
                                break;
                            }
                            request.max_transmission_unit = match i {
                                0 => 1492,
                                1 => 1200,
                                _ => 576,
                            };
                            for _ in 0..4 {
                                if max_mtu != 0 {
                                    println!("Breaking {}", max_mtu);
                                    break;
                                }
                                conn.WritePacket(Box::new(&request), true).await?;
                                println!("Sent OpenConnectionRequest1 with MTU {}", request.max_transmission_unit);
                                tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
                            }
                        }
                        if max_mtu == 0 {
                            println!("Failed to negotiate MTU");
                            exit(1)
                        } else {
                            println!("Negotiated MTU: {}", max_mtu);
                        }
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
