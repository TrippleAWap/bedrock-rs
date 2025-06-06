use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct DynamicQueue<T> {
    pub inner: Mutex<Inner<T>>,
    pub condvar: Condvar,
    pub max_capacity: usize,
}

pub struct Inner<T> {
    pub buffer: VecDeque<T>,
    pub capacity: usize,
}

impl<T> DynamicQueue<T> {
    pub fn new(initial_capacity: usize, max_capacity: usize) -> Self {
        let buffer = VecDeque::with_capacity(initial_capacity);
        let capacity = buffer.capacity();
        DynamicQueue {
            inner: Mutex::new(Inner { buffer, capacity }),
            condvar: Condvar::new(),
            max_capacity,
        }
    }

    pub fn send(&self, value: T) {
        let mut inner = self.inner.lock().unwrap();
        if inner.buffer.len() == inner.capacity && inner.capacity < self.max_capacity {
            // Grow the buffer
            let new_capacity = (inner.capacity * 2).min(self.max_capacity);
            let mut new_buffer = VecDeque::with_capacity(new_capacity);
            while let Some(val) = inner.buffer.pop_front() {
                new_buffer.push_back(val);
            }
            inner.buffer = new_buffer;
            inner.capacity = new_capacity;
        }
        inner.buffer.push_back(value);
        self.condvar.notify_one();
    }

    pub fn recv(&self) -> T {
        let mut inner = self.inner.lock().unwrap();
        loop {
            if let Some(value) = inner.buffer.pop_front() {
                return value;
            }
            inner = self.condvar.wait(inner).unwrap();
        }
    }
}