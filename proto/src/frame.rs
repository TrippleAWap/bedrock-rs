use std::cmp::max;
use std::collections::HashMap;
use crate::types::uint24;

pub struct Window {
    pub lowest: uint24,
    pub highest: uint24,
    pub queue: HashMap<uint24, std::time::SystemTime>,
}

impl Window {
    pub fn new() -> Self {
        Self {highest: 0, lowest: 0, queue: HashMap::new()}
    }

    pub fn add(&mut self, index: uint24) -> bool {
        if self.seen(index) {
            return false
        }
        self.highest = max(index+1, self.highest);
        self.queue.insert(index, std::time::SystemTime::now());
        true
    }

    pub fn seen(&self, index: uint24) -> bool {
        if index < self.lowest {
            return true
        }
        self.queue.contains_key(&index)
    }

    pub fn shift(&mut self) -> usize {
        let mut index: uint24 = self.lowest;
        let mut n = 0;
        while index < self.highest {
            index += 1;
            if !self.queue.contains_key(&index) {
                break
            }
            self.queue.remove(&index);
            n += 1;
        }
        self.lowest = index;
        n
    }

    pub fn missing(&mut self, since: std::time::Duration) -> Result<Vec<uint24>, Box<dyn std::error::Error>> {
        let mut missing = false;
        let mut indecies: Vec<uint24> = Vec::new();
        let mut index = self.highest as isize - 1;
        while index >= self.lowest as isize {
            index -= 1;
            let i = index as uint24;
            match self.queue.get_key_value(&i) {
                Some((_, v)) => {
                    let prev = std::time::SystemTime::now().duration_since(*v)?;
                    if prev >= since {
                        missing = true;
                    }
                    continue
                },
                _ => {
                    if missing {
                        indecies.push(i);
                        self.queue.remove(&i);
                    }
                },
            }
        }
        self.shift();
        Ok(indecies)
    }

    pub fn len(&self) -> usize {
        (self.highest - self.lowest) as usize
    }
}