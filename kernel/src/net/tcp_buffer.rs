// XPARQ OS - Phase 18: Pluggable TCP Buffers
// Defines buffer abstraction for standard copying or zero-copy future upgrades.

pub trait TcpBuffer {
    fn write(&mut self, data: &[u8]) -> usize;
    fn read(&mut self, data: &mut [u8]) -> usize;
    fn peek(&self, offset: usize, data: &mut [u8]) -> usize;
    fn advance(&mut self, count: usize);
    fn available_space(&self) -> usize;
    fn available_data(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn is_full(&self) -> bool;
}

/// A standard static ring buffer for kernel-space buffering.
#[derive(Clone, Copy)]
pub struct RingBuffer<const N: usize> {
    buffer: [u8; N],
    head: usize,
    tail: usize,
    count: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }
}

impl<const N: usize> TcpBuffer for RingBuffer<N> {
    fn write(&mut self, data: &[u8]) -> usize {
        let space = N - self.count;
        let to_write = data.len().min(space);
        
        for i in 0..to_write {
            self.buffer[self.tail] = data[i];
            self.tail = (self.tail + 1) % N;
        }
        
        self.count += to_write;
        to_write
    }

    fn read(&mut self, data: &mut [u8]) -> usize {
        let to_read = data.len().min(self.count);
        
        for i in 0..to_read {
            data[i] = self.buffer[self.head];
            self.head = (self.head + 1) % N;
        }
        
        self.count -= to_read;
        to_read
    }

    fn peek(&self, offset: usize, data: &mut [u8]) -> usize {
        if offset >= self.count {
            return 0;
        }
        
        let to_read = data.len().min(self.count - offset);
        let mut curr = (self.head + offset) % N;
        
        for i in 0..to_read {
            data[i] = self.buffer[curr];
            curr = (curr + 1) % N;
        }
        
        to_read
    }

    fn advance(&mut self, count: usize) {
        let to_adv = count.min(self.count);
        self.head = (self.head + to_adv) % N;
        self.count -= to_adv;
    }

    fn available_space(&self) -> usize {
        N - self.count
    }

    fn available_data(&self) -> usize {
        self.count
    }

    fn is_empty(&self) -> bool {
        self.count == 0
    }

    fn is_full(&self) -> bool {
        self.count == N
    }
}
