use std::{io::prelude::*, sync::atomic::{AtomicU32, AtomicU64, Ordering}};
use std::net::TcpStream;

pub fn construct(addr: String) -> Result<TCP_Manager, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(&addr)?;
    Ok(TCP_Manager {
        address: addr,
        stream: stream,
        count: AtomicU32::new(0),
        bytes: AtomicU64::new(0)
    })
}
pub struct TCP_Manager {
    pub address: String,
    pub stream: TcpStream,
    pub count: AtomicU32,
    pub bytes: AtomicU64
}

impl TCP_Manager {
    pub fn send(&mut self, data: String) -> bool {
        self.bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        match self.stream.write(data.as_bytes()) {
            Ok(_) => true,
            Err(_) => false
        }
    }

    pub fn flood(mut self, data: String, count: u32) {
        for _ in 0..count {
            Self::send(&mut self, data.clone());
        }
    }
}