use std::net::Ipv4Addr;
use crate::Result;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Peer {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

pub fn unmarshal(peers_bin: &[u8]) -> Result<Vec<Peer>> {
    let peer_size = 6;
    let num_peers = peers_bin.len() / peer_size;

    if !peers_bin.len().is_multiple_of(peer_size) {
        return Err("Received malformed peers".into());
    }

    let mut peers: Vec<Peer> = vec![];

    for i in 0..num_peers {
        let offset = i * peer_size; 
        
        let peer = Peer {
            ip: Ipv4Addr::new(
                peers_bin[offset], 
                peers_bin[offset + 1], 
                peers_bin[offset + 2], 
                peers_bin[offset + 3]
            ),
            port: u16::from_be_bytes(
                [peers_bin[offset + 4], peers_bin[offset + 5]]
            ),
        };
        
        peers.push(peer);
    }

    Ok(peers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmarshal_peers() -> Result<()> {
        let peers_bin = [127, 0, 0, 1, 0x00, 0x50, 1, 1, 1, 1, 0x01, 0xbb];
		let expected = vec![
            Peer { ip: Ipv4Addr::new(127, 0, 0, 1), port: 80},
            Peer { ip: Ipv4Addr::new(1, 1, 1, 1), port: 443},
        ];

        assert_eq!(unmarshal(&peers_bin)?, expected);

        Ok(())
    }

    #[test]
    fn peer_to_string() {
        let peer = Peer {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 8080,
        };

        assert_eq!(peer.to_string(), "127.0.0.1:8080");
    }
}
