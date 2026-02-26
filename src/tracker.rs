use std::net::Ipv4Addr;
use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde_bytes::ByteBuf;
use reqwest::Url;
use url::form_urlencoded::byte_serialize;
use crate::{TorrentFile, Peer};

#[derive(Deserialize)]
#[allow(dead_code)]
struct TrackerResponse {
    interval: i64,
    peers: ByteBuf,
}

pub async fn request_peers(
    torrent: &TorrentFile,
    peer_id: [u8; 20],
    port: u16,
) -> Result<Vec<Peer>> {
    let url = build_tracker_url(torrent, peer_id, port)?;
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let tracker: TrackerResponse = serde_bencode::from_bytes(&bytes)?;
    
    parse_compact_peers(&tracker.peers)
}

fn build_tracker_url(
    torrent: &TorrentFile,
    peer_id: [u8; 20],
    port: u16,
) -> Result<Url> {
    let mut url = Url::parse(&torrent.announce)?;
    let info_hash = byte_serialize(&torrent.info_hash).collect::<String>();
    let peer_id = byte_serialize(&peer_id).collect::<String>();

    url.set_query(Some(&format!(
        "compact=1&downloaded=0&uploaded=0&left={}&port={}&info_hash={}&peer_id={}",
        torrent.length,
        port,
        info_hash,
        peer_id
    )));

    Ok(url)
}

fn parse_compact_peers(bytes: &[u8]) -> Result<Vec<Peer>> {
    if !bytes.len().is_multiple_of(6) {
        return Err(anyhow!("invalid compact peer list"));
    }
    
    let mut peers = Vec::new();

    for chunk in bytes.chunks(6) {
        let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
        
        peers.push(Peer {
            ip,
            port,
        });
    }

    Ok(peers)
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_debian_tracker_url() -> Result<()> {
    	let torrent_file = TorrentFile {
    		announce: "http://bttracker.debian.org:6969/announce".to_string(),
    		info_hash: [216, 247, 57, 206, 195, 40, 149, 108, 204, 91, 191, 31, 134, 217, 253, 207, 219, 168, 206, 182],
    		pieces: vec![
    			[49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106],
    			[97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48],
    		],
    		piece_length: 262144,
    		length: 351272960,
    		name: "debian-10.2.0-amd64-netinst.iso".to_string(),
    	};

        let peer_id = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
        let port: u16 = 6882;
        let url = build_tracker_url(&torrent_file, peer_id, port)?;
        let expected = "http://bttracker.debian.org:6969/announce?compact=1&downloaded=0&uploaded=0&left=351272960&port=6882&info_hash=%D8%F79%CE%C3%28%95l%CC%5B%BF%1F%86%D9%FD%CF%DB%A8%CE%B6&peer_id=%01%02%03%04%05%06%07%08%09%0A%0B%0C%0D%0E%0F%10%11%12%13%14";
        assert_eq!(url.to_string(), expected);

        Ok(())
    }

    #[test]
    fn parse_compact_pair_peers() {
        let peers =  [
            192, 0, 2, 123, 0x1A, 0xE1, // 6881
            127, 0, 0, 1, 0x1A, 0xE9,   // 6889
        ];

        let expected = vec![
            Peer { ip: Ipv4Addr::new(192, 0, 2, 123), port: 6881 },
            Peer { ip: Ipv4Addr::new(127, 0, 0, 1), port: 6889 },
        ];

        assert_eq!(parse_compact_peers(&peers).unwrap(), expected);
    }
}
