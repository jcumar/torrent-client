use std::net::{TcpStream, ToSocketAddrs};
use crate::bitfield::Bitfield;
use crate::handshake::Handshake;
use std::io::{self, Write};
use std::time::Duration;
use crate::message::{Message, MessageID};
use crate::peers::Peer;

pub struct Client {
    pub conn: TcpStream,
    pub choked: bool,
    pub bitfield: Bitfield,
    pub peer: Peer,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Client {
    pub fn new(peer: Peer, peer_id: [u8; 20], info_hash: [u8; 20]) -> io::Result<Self> {
        let mut conn = TcpStream::connect_timeout(
            &peer.to_string().to_socket_addrs()?.next().ok_or(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "Could not resolve peer address",
            ))?,
            Duration::from_secs(3),
        )?;

        complete_handshake(&mut conn, info_hash, peer_id)?;

        let bf = recv_bitfield(&mut conn)?;

        Ok(Client {
            conn,
            choked: true,
            bitfield: bf,
            peer,
            info_hash,
            peer_id,
        })
    }

    pub fn read(&self, conn: &mut TcpStream) -> io::Result<Message> {
        let msg = Message::read(conn)?;

        let Some(msg) = msg else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected bitfield but got {:?}", msg),
            ));
        };

        Ok(msg)
    }

    pub fn send_request(&self, conn: &mut TcpStream, index: u32, begin: u32, length: u32) -> io::Result<()> {
        let req = Message::format_request(index, begin, length);
        conn.write_all(&Message::serialize(Some(&req)))
    }
    
    pub fn send_interested(&self, conn: &mut TcpStream) -> io::Result<()> {
        let msg = Message { id: MessageID::Interested, payload: vec![], };
        conn.write_all(&Message::serialize(Some(&msg)))
    }

    pub fn send_not_interested(&self, conn: &mut TcpStream) -> io::Result<()> {
        let msg = Message { id: MessageID::NotInterested, payload: vec![], };
        conn.write_all(&Message::serialize(Some(&msg)))
    }

    pub fn send_unchoke(&self, conn: &mut TcpStream) -> io::Result<()> {
        let msg = Message { id: MessageID::Unchoke, payload: vec![], };
        conn.write_all(&Message::serialize(Some(&msg)))
    }

    pub fn send_have(&self, conn: &mut TcpStream, index: u32) -> io::Result<()> {
        let req = Message::format_have(index);
        conn.write_all(&Message::serialize(Some(&req)))
    }
}

fn complete_handshake(conn: &mut TcpStream, info_hash: [u8; 20], peer_id: [u8; 20]) -> io::Result<Handshake> {
    conn.set_read_timeout(Some(Duration::from_secs(3)))?;
    conn.set_write_timeout(Some(Duration::from_secs(3)))?;

    let req = Handshake::new(info_hash, peer_id);

    conn.write_all(&req.serialize())?;
    
    let res = Handshake::read(conn)?;

    if res.info_hash != info_hash {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expected infohash {:x?} but got {:x?}", info_hash, res.info_hash),
        ));
    }
    
    conn.set_read_timeout(None)?;
    conn.set_write_timeout(None)?;

    Ok(res)
}

fn recv_bitfield(conn: &mut TcpStream) -> io::Result<Bitfield> {
    conn.set_read_timeout(Some(Duration::from_secs(3)))?;
    conn.set_write_timeout(Some(Duration::from_secs(3)))?;

    let msg = Message::read(conn)?;

    let Some(msg) = msg else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expected bitfield but got {:?}", msg),
        ));
    };

    if msg.id != MessageID::Bitfield {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Expected bitfield but got ID {:?}", msg.id),
        ));
    }

    Ok(Bitfield(msg.payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    use std::io::Write;
    use std::thread;

    fn create_client_and_server() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind to local addr");
        let addr = listener.local_addr().expect("Could not get local addr");

        let server_handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("Could not accept connection");
            stream
        });

        let client_stream = TcpStream::connect(addr).expect("Could not connect to server");
        let server_stream = server_handle.join().expect("Server thread panicked");

        (client_stream, server_stream)
    }

    #[test]
    fn test_recv_bitfield() {
        let cases = vec![
            (
                vec![0x00, 0x00, 0x00, 0x06, 5, 1, 2, 3, 4, 5],
                Some(Bitfield(vec![1, 2, 3, 4, 5])),
                false, 
            ),
            (
                vec![0x00, 0x00, 0x00, 0x06, 99, 1, 2, 3, 4, 5],
                None,
                true,
            ),
        ];

        for (msg, expected, should_fail) in cases {
            let (mut client, mut server) = create_client_and_server();
            server.write_all(&msg).unwrap();

            let result = recv_bitfield(&mut client);
            if should_fail {
                assert!(result.is_err());
            } else {
                assert_eq!(result.unwrap(), expected.unwrap());
            }
        }
    }
}
