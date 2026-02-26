use std::net::Ipv4Addr;
use anyhow::{Result, anyhow};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::Message;

#[derive(Debug, PartialEq)]
pub struct Peer {
    pub ip: Ipv4Addr, 
    pub port: u16,
}

pub struct PeerConnection {
    pub stream: TcpStream,
    pub bitfield: Vec<u8>,
    pub choked: bool,
}

impl PeerConnection {
    pub async fn connect(
        addr: &str, 
        info_hash: [u8; 20], 
        peer_id: [u8; 20]
    ) -> Result<Self> {
        let mut stream = TcpStream::connect(addr).await?;

        send_handshake(&mut stream, info_hash, peer_id).await?;
        receive_handshake(&mut stream, info_hash).await?;

        Ok(PeerConnection { 
            stream, 
            bitfield: vec![], 
            choked: true,
        })
    }

    pub async fn send_interested(&mut self) -> Result<()> {
        Message::Interested.write(&mut self.stream).await
    }

    pub async fn read_message(&mut self) -> Result<Message> {
        Message::read(&mut self.stream).await
    }

    pub async fn send_request(
        &mut self,
        index: u32,
        begin: u32,
        length: u32,
    ) -> Result<()> {
        Message::Request { index, begin, length }
            .write(&mut self.stream)
            .await
    }

    pub async fn read_message_and_update(&mut self) -> Result<Message> {
        let msg = Message::read(&mut self.stream).await?;

        match &msg {
            Message::Bitfield(bits) => {
                self.bitfield = bits.clone();
            }, 
            Message::Have(index) => {
                self.set_piece(*index);
            }, 
            Message::Choke => {
                self.choked = true;
            },
            Message::Unchoke => {
                self.choked = false;
            },
            _ => {},
        }

        Ok(msg)
    }

    fn set_piece(&mut self, index: u32) {
        let byte_index = (index / 8) as usize;
        let bit_index = (index % 8) as u8;

        if byte_index >= self.bitfield.len() {
            return;
        }

        self.bitfield[byte_index] |= 1 << (7 - bit_index);
    }
    
    pub fn has_piece(&self, index: u32) -> bool {
        let byte_index = (index / 8) as usize;
        let bit_index = (index % 8) as u8;

        if byte_index >= self.bitfield.len() {
            return false;
        }

        let mask = 1 << (7 - bit_index);
        self.bitfield[byte_index] & mask != 0
    }

    pub fn is_choked(&self) -> bool {
        self.choked
    }
}

async fn send_handshake(
    stream: &mut TcpStream,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
) -> Result<()> {
    let mut buf = vec![];

    buf.push(19);
    buf.extend(b"BitTorrent protocol");
    buf.extend([0u8; 8]);
    buf.extend(info_hash);
    buf.extend(peer_id);

    stream.write_all(&buf).await?;

    Ok(())
}

async fn receive_handshake(
    stream: &mut TcpStream, 
    expected_info_hash: [u8; 20]
) -> Result<()> {
    let mut buf = [0u8; 68];

    stream.read_exact(&mut buf).await?;

    if &buf[1..20] != b"BitTorrent protocol" {
        return Err(anyhow!("invalid protocol string"));
    }

    let received_info_hash = &buf[28..48];

    if received_info_hash != expected_info_hash {
        return Err(anyhow!("info_hash mismatch"));
    }

    Ok(())
}
