use std::collections::HashMap;
use std::ops::{Add, Div};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use lazy_static::lazy_static;
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, Mutex};
use crate::address::Address;
use crate::dynamic_queue::DynamicQueue;
use crate::messages::connected_ping::ConnectedPing;
use crate::messages::unknown::UnknownPacket;
use crate::types::{read_u24, uint24, Packet};
use crate::frame::Window;
use crate::packet_queue::PacketQueue;
use crate::{PacketT, ReadPacket};
use crate::packet::PacketBitFlags;

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

    pub window: Mutex<Window>,

    pub ack_slice: Mutex<Vec<uint24>>,

    pub packet_queue: *mut PacketQueue,
    pub packets: DynamicQueue<Vec<u8>>,

    pub last_packet_time: Arc<*mut SystemTime>,

    pub limits_enabled: bool,
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

            window: Mutex::new(Window::new()),
            packet_queue: Box::into_raw(Box::new(PacketQueue::new())),
            close_conn: |socket| {
                drop(socket)
            },

            limits_enabled: true,
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
                self.check_resend(system_time).await;
            }
            if tick_count%5 == 0 {
                _ = self.conn.send(&ConnectedPing{client_send_time_be: system_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64}.serialize());
            }
        }
    }
    pub async fn send_nack(&self, missing: &[uint24]) -> Result<(), Box<dyn std::error::Error>> {
        let res = self.send_ack(missing, PacketBitFlags::NACK, self.nack_buf.lock().await.as_mut()).await;
        self.nack_buf.lock().await.clear();
        res
    }
    /// this function itself resends? (atleast in go-raknet, i might change that because that's weird)
    pub async fn check_resend(&mut self, now: SystemTime) {
        // we calculate this.. im lazy i'll do this later.
        // self.round_trip_time = ;
    }

    #[allow(non_snake_case)]
    pub fn ReceivePacket(&self, data: &[u8]) -> Result<Option<PacketT>, String> {
        if data[0]&PacketBitFlags::ACK as u8 != 0 {
            self.handle_ack(&data)
        } else if data[0]&PacketBitFlags::NACK as u8 != 0 {
            self.handle_nack(&data)
        } else if data[0]&PacketBitFlags::Datagram as u8 != 0 {
            self.handle_datagram(&data)
        } else {
            ReadPacket(&data)
        }
    }

    pub async fn handle_datagram(&self, data: &[u8]) -> Result<Option<PacketT>, String> {
        let mut window = self.window.lock().await;
        let sequence_number = read_u24(&data[0..3]);
        if !window.add(sequence_number) {
            return Ok(None)
        }
        self.ack_slice.lock().await.push(sequence_number);

        if window.shift() == 0 {
            let round_trip_time = self.round_trip_time.clone();
            let missing = match window.missing(Duration::from_millis(round_trip_time.clone().add(round_trip_time.div(2)))) {
                Ok(missing) => missing,
                Err(e) => {
                    return Err(format!("Failed to get missing data from packet queue: {}", e))
                }
            };
            if missing.len() > 0 {
                match self.send_nack(&missing).await {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(format!("Failed to send nack: {}", e))
                    }
                }
            }
        }
        if self.window.lock().await.len() > MAX_WINDOW_SIZE as usize && self.limits_enabled {
            return Err(format!("receive datagram: queue window size is too big ({}->{})", window.lowest, window.highest).to_string())
        }

        self.handle_datagram(&data[3..])
    }

    pub fn handle_nack(data: &[u8]) -> Result<Option<PacketT>, String> {
        ReadPacket(data)
    }

    pub fn handle_ack(data: &[u8]) -> Result<Option<PacketT>, String> {
        ReadPacket(data)
    }

}

