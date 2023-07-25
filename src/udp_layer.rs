use std::io;
use tokio::net::{UdpSocket};


const MAX_PACKET_SIZE :usize = 1024;

pub struct UdpListener {
    socket: UdpSocket,
}

impl UdpListener {
    pub async fn bind(host_addr: &str) -> io::Result<UdpListener> {
        let socket = UdpSocket::bind(host_addr).await?;
        Ok(UdpListener { socket })
    }

    pub async fn receive(&mut self) -> io::Result<UdpPacket> {
        let mut buf = vec![0u8; MAX_PACKET_SIZE];
        let (size, src) = self.socket.recv_from(&mut buf).await?;
        buf.truncate(size);
        Ok(UdpPacket { data: buf, src })
    }
}

pub struct UdpPacket {
    data: Vec<u8>,
    src: std::net::SocketAddr,
}

impl UdpPacket {
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn src(&self) -> std::net::SocketAddr {
        self.src
    }
}

/*
pub struct UdpSocketConnection {
    socket: UdpSocket,
    remote: std::net::SocketAddr,
}

impl UdpSocketConnection {
    pub async fn new(host_addr: &str, remote: &str) -> Result<UdpSocketConnection, Box<dyn std::error::Error> >  {
        let socket = UdpSocket::bind(host_addr).await?;
        let remote_addr = remote.parse()?;
        Ok(UdpSocketConnection { socket, remote: remote_addr })
    }

    pub async fn send(&self, data: &[u8]) -> io::Result<usize> {
        self.socket.send_to(data, &self.remote).await
    }
}
*/


#[derive(Clone)]
pub struct UdpMessage {
    pub message: String
}
