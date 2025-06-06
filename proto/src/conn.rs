use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, Mutex};
use crate::address::Address;
use crate::dynamic_queue::DynamicQueue;
use crate::messages::connected_ping::ConnectedPing;
use crate::messages::unknown::UnknownPacket;
use crate::types::{uint24, Packet};
use crate::frame::Window;
use crate::packet_queue::PacketQueue;

// Current RakNet protocol version for Minecraft
const PROTOCOL_VERSION: u8 = 11;

const MIN_TRANSMISSION_UNIT_SIZE: u16    = 576;
const MAX_TRANSMISSION_UNIT_SIZE: u16    = 1492;
const MAX_WINDOW_SIZE: u16 = 2048;


pub struct Conn {
    pub round_trip_time: Arc<u64>,
    pub closing: Arc<bool>,

    pub conn: UdpSocket,
    pub remote_address: Address,

    pub connected_rx: Option<oneshot::Receiver<()>>,
    pub connected_tx: Option<oneshot::Sender<()>>,

    pub buf: Mutex<Vec<u8>>,
    pub close_conn: fn(UdpSocket),
    pub ack_buf: Mutex<Vec<u8>>,
    pub nack_buf: Mutex<Vec<u8>>,

    pub packet: Box<dyn Packet>,

    pub sequence_number: uint24,
    pub order_index: uint24,
    pub message_index: uint24,

    pub split_id: u32,

    pub max_transmission_unit: u16,

    pub splits: HashMap<u16, Vec<Vec<u8>>>,

    pub window: *mut Window,

    pub ack_slice: Mutex<Vec<uint24>>,

    pub packet_queue: *mut PacketQueue,
    pub packets: DynamicQueue<Vec<u8>>,

    pub last_packet_time: Arc<*mut SystemTime>,
}

impl Conn {
    pub fn new(socket: UdpSocket, remote_address: Address, max_transmission_unit: u16) -> Self {
        let (tx, rx) = oneshot::channel::<()>();
        Self {
            conn: socket,
            remote_address,
            max_transmission_unit,

            round_trip_time: Arc::new(0),
            closing: Arc::new(false),

            connected_rx: Some(rx),
            connected_tx: Some(tx),

            packets: DynamicQueue::new(4, 4096),

            buf: Mutex::new(Vec::with_capacity(max_transmission_unit as usize - 28)),
            ack_buf: Mutex::new(Vec::with_capacity(128)),
            nack_buf: Mutex::new(Vec::with_capacity(64)),

            packet: Box::new(UnknownPacket{id: 0, data: Vec::new()}),
            splits: HashMap::new(),
            ack_slice: Mutex::new(Vec::new()),
            last_packet_time: Arc::new(Box::into_raw(Box::new(SystemTime::now()))),

            sequence_number: 0,
            order_index: 0,
            message_index: 0,
            split_id: 0,

            window: Box::into_raw(Box::new(Window::new())),
            packet_queue: Box::into_raw(Box::new(PacketQueue::new())),
            close_conn: |socket| {
                drop(socket)
            },
        }
    }
    pub fn effective_mtu(&self) -> u16 {
        self.max_transmission_unit - 28
    }
    pub async fn start_ticking(&mut self) {
        const INTERVAL: Duration = Duration::from_millis(100);
        let mut tick_count: i64 = 0;
        let mut acks_left: i32 = 0;
        loop {
            tokio::time::sleep(INTERVAL).await;
            let system_time =  SystemTime::now();
            tick_count += 1;
            if *self.closing.as_ref() {
                break;
            }
            if tick_count%3 == 0 {
                _ = Conn::check_resend(system_time).await;
            }
            if tick_count%5 == 0 {
                _ = self.conn.send(&ConnectedPing{client_send_time_be: system_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64}.serialize());
            }
        }
    }
    pub async fn check_resend(timestamp: SystemTime) {
        println!("Checking resend");
    }
    pub async fn write() {}
    pub async fn read() {}
}