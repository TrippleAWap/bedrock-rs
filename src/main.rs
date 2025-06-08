mod client;
mod server;

use proto::types::Packet;
use std::env::args_os;
use std::process::exit;
use crate::client::client;
use crate::server::server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut args = args_os();
    if args.len() < 3 {
        println!("Usage: {} <address> <server|client>", args.next().unwrap().to_str().unwrap());
        exit(1);
    }
    // skip the first arg;
    args.next();

    let addr = args.next().unwrap().into_string().unwrap();
    println!("Address: {}", addr);
    let run_server = args.next().unwrap().to_str().unwrap() == "server";
    if run_server {
        println!("Starting server");
        server(addr).await?;
    } else {
        println!("Starting client");
        client(addr).await?;
    }
    Ok(())
}

