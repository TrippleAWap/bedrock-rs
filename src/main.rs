use std::cmp::min;
use proto::types::Packet;
use proto::{PacketT};
use std::env::args_os;
use std::process::exit;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use proto::conn::Conn;
use proto::messages::open_connection_request_1::OpenConnectionRequest1;
use proto::messages::unconnected_ping::UnconnectedPing;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = args_os();
    if args.len() < 2 {
        println!("Usage: {} <target_address>", args.next().unwrap().to_str().unwrap());
        exit(1);
    }
    // skip the first arg;
    args.next();

    let target_address = args.next().unwrap().into_string().unwrap();
    println!("Target address: {}", target_address);
    client(target_address).await?;
    Ok(())
}

async fn client(target_address: String) -> std::io::Result<()> {
    println!("Connecting to {}", target_address.clone());
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!("Bound to {}", socket.local_addr()?);
    let start_time = SystemTime::now();
    socket.connect(target_address.clone()).await?;
    println!("Connected to {} in {}ms", target_address, start_time.elapsed().unwrap().as_millis());
    let mut buf = [0u8; 4096];
    socket.send(&UnconnectedPing { client_guid_be: 0, client_send_time_be: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64 }.serialize()).await?;
    let socket = Arc::new(Mutex::new(socket));
    let mut conn = Conn::new(socket.clone(), 4096).await;
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
                            max_transmission_unit: 1492,
                            client_protocol: 11,
                        };
                        while request.max_transmission_unit != 0 {
                            for _ in 0..4 {
                                if max_mtu != 0 {
                                    println!("Breaking {}", max_mtu);
                                    break;
                                }
                                let sent_bytes= socket.lock().await.send(&request.serialize()).await;
                                if sent_bytes.is_err() {
                                    println!("Failed to send OpenConnectionRequest1");
                                    break;
                                }
                                println!("Sent OpenConnectionRequest1 with MTU {}", request.max_transmission_unit);
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }
                            if max_mtu != 0 {
                                break;
                            }
                            request.max_transmission_unit -= min(1024, request.max_transmission_unit);
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
