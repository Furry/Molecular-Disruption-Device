use std::{io::prelude::*, sync::atomic::{AtomicU32, AtomicU64, Ordering}};
use std::net::UdpSocket;

pub fn construct(addr: String) -> Result<UDP_Manager, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind(&addr)?;
    Ok(UDP_Manager {
        address: addr,
        socket: socket,
        count: AtomicU32::new(0),
        bytes: AtomicU64::new(0)
    })
}

pub struct UDP_Manager {
    pub address: String,
    pub socket: UdpSocket,
    pub count: AtomicU32,
    pub bytes: AtomicU64
}

impl UDP_Manager {
    pub fn send(&mut self, data: String) -> bool {
        self.bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        match self.socket.send(data.as_bytes()) {
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