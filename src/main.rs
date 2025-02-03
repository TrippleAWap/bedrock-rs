use proto::types::Packet;
use proto::{unconnected_pong::UnconnectedPong, PacketT, ReadPacket};
use rand::random;
use std::env::args_os;
use std::time::SystemTime;
use tokio::net::UdpSocket;
use proto::open_connection_reply_1::OpenConnectionReply1;
use proto::open_connection_request_1::OpenConnectionRequest1;
use proto::unconnected_ping::UnconnectedPing;

#[tokio::main]
async fn main()  -> std::io::Result<()> {
    let mut args = args_os();
    if args.len() < 3 {
        println!("Usage: {} <target_address> <client|server>", args.next().unwrap().to_str().unwrap());
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid argument"));
    }
    let target_address = args.nth(1).unwrap().into_string().unwrap();
    if !args.any(|a| a == "client") {
        println!("Starting server");
        Ok(server(target_address).await?)
    } else {
        println!("Starting client");
        Ok(client(target_address).await?)
    }
}

async fn server(target_address: String) -> std::io::Result<()> {
    println!("Listening on {}", target_address);

    let socket = UdpSocket::bind(target_address).await?;
    let server_id: u64 = random();
    let mut buf = [0u8; 1492];

    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let received_data = &buf[..len];

        match ReadPacket(received_data) {
            Ok(packet) => {
                println!("Received {:?}", packet);
                match packet {
                    PacketT::UnconnectedPing(packet) => {
                        let response =  UnconnectedPong{server_guid_be: server_id, client_send_time_be: packet.client_send_time_be, data: Vec::from("MCPE;Dedicated Server;766;1.21.51;0;10;13253860892328930865;Bedrock level;Survival;1;19132;19133;") };
                        socket.send_to(&response.serialize(), src).await?;
                    }
                    PacketT::OpenConnectionRequest1(packet) => {
                        if packet.max_transmission_unit > buf.len() {
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

async fn client(target_address: String) -> std::io::Result<()> {
    println!("Connecting to {}", target_address);

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!("Bound to {}", socket.local_addr()?);
    socket.connect(target_address).await?;

    let mut buf = [0u8; 1492];

    socket.send(&UnconnectedPing { client_guid_be: 0, client_send_time_be: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64 }.serialize()).await?;

    let (len, src) = socket.recv_from(&mut buf).await?;
    let received_data = &buf[..len];
    match ReadPacket(received_data) {
        Ok(packet) => {
            println!("Received packet from {}: {:?}", src, packet);
            match packet {
                PacketT::UnconnectedPong(_packet) => {
                    println!("Received UnconnectedPong flooding with OpenConnectionRequest1");
                    let response = OpenConnectionRequest1 {
                        max_transmission_unit: 4096,
                        client_protocol: 11,
                    }.serialize();
                    
                    const X: usize = 300;
                    println!("Sending OpenConnectionRequest1 to {} {} times", src, X);
                    for i in 0..X {
                        println!("Sending OpenConnectionRequest1 to {} ({}/{})", src, i+1, X);
                        let _ = socket.send(&response);
                    }
                    println!("Done");
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
    Ok(())
}