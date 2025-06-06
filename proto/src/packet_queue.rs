use std::fmt::Debug;
use crate::types::uint24;

/// PacketQueue is an ordered queue for reliable ordered packets.
pub struct PacketQueue {
    pub lowest: uint24,
    pub highest: uint24,
    pub queue: std::collections::HashMap<uint24, Vec<u8>>,
}

impl Default for PacketQueue {
    fn default() -> PacketQueue {
        PacketQueue::new()
    }
}

impl Debug for PacketQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PacketQueue")
            .field("lowest", &self.lowest)
            .field("highest", &self.highest)
            .field("queue", &self.queue)
            .finish()
    }
}

impl PacketQueue {
    pub(crate) fn new() -> PacketQueue {
        PacketQueue {
            lowest: 0,
            highest: 0,
            queue: std::collections::HashMap::new(),
        }
    }

    /// put puts a value at the index passed. If the index was already occupied
    /// once, false is returned.
    pub fn put(self: &mut PacketQueue, index: uint24, buffer: Vec<u8>) -> bool {
        if index < self.lowest {
            self.lowest = index;
        }
        if self.queue.contains_key(&index) {
            return false;
        }
        if index >= self.highest {
            self.highest = index + 1;
        }
        self.queue.insert(index, buffer);
        true
    }

    /// fetch attempts to take out as many values from the ordered queue as
    /// possible. Upon encountering an index that has no value yet, the function
    /// returns all values that it did find and takes them out.
    pub fn fetch(self: &mut PacketQueue) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        let mut i = self.lowest;
        while i < self.highest {
            let asasd = self.queue.clone();
            let buffer = match asasd.get(&i) {
                Some(buffer) => buffer,
                None => break,
            };
            self.queue.remove(&i);
            packets.push(buffer.clone());
            i += 1;
        }
        self.lowest = i;

        packets
    }

    /// window_size returns the size of the window held by the packet queue.
    pub fn window_size(self: &PacketQueue) -> uint24 {
        self.highest - self.lowest
    }
}