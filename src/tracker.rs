use reqwest::blocking::Client;
use crate::torrentfile::TorrentFile;
use crate::Result;
use crate::peers::{self, Peer};
use serde::Deserialize;
use serde_bencode::de;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct BencodeTrackerResp {
	interval: i64,
    #[serde(with = "serde_bytes")]
    peers: Vec<u8>,
}

pub fn build_tracker_url(
    torrent_file: &TorrentFile, 
    peer_id: [u8; 20], 
    port: u16) -> Result<String> 
{
    let url = format!(
        "{}?compact=1&downloaded=0&info_hash={}&left={}&peer_id={}&port={}&uploaded=0",
        &torrent_file.announce,
        percent_encode_bytes(&torrent_file.info_hash),
        torrent_file.length,
        percent_encode_bytes(&peer_id),
        port,
    );

    Ok(url)
}

fn percent_encode_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            match b {
                b'A'..=b'Z'
                | b'a'..=b'z'
                | b'0'..=b'9'
                | b'-'
                | b'_'
                | b'.'
                | b'~' => (b as char).to_string(),
                _ => format!("%{:02X}", b),
            }
        })
        .collect()
}

pub fn request_peers(
    torrent_file: &TorrentFile, 
    peer_id: [u8; 20], 
    port: u16) -> Result<Vec<Peer>> 
{
    let url = build_tracker_url(torrent_file, peer_id, port)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    let resp = client.get(&url).send()?;

    let bytes = resp.bytes()?;
    	
    let tracker_resp: BencodeTrackerResp = de::from_bytes(&bytes)?;
    let result = peers::unmarshal(&tracker_resp.peers)?;

    Ok(result) 
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use std::net::Ipv4Addr;
    use crate::Result;

    #[test]
    fn build_debian_tracker_url() -> Result<()> {
    	let torrent_file = TorrentFile {
    		announce: "http://bttracker.debian.org:6969/announce".to_string(),
    		info_hash: [216, 247, 57, 206, 195, 40, 149, 108, 204, 91, 191, 31, 134, 217, 253, 207, 219, 168, 206, 182],
    		piece_hashes: vec![
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
        let expected = "http://bttracker.debian.org:6969/announce?compact=1&downloaded=0&info_hash=%D8%F79%CE%C3%28%95l%CC%5B%BF%1F%86%D9%FD%CF%DB%A8%CE%B6&left=351272960&peer_id=%01%02%03%04%05%06%07%08%09%0A%0B%0C%0D%0E%0F%10%11%12%13%14&port=6882&uploaded=0";
        assert_eq!(url, expected);

        Ok(())
    }
    #[test]
    fn test_request_peers() {
        let server = MockServer::start();

        let response = {
            let mut data = Vec::new();
            data.extend(b"d8:intervali900e5:peers12:");

            data.extend(&[
                192, 0, 2, 123, 0x1A, 0xE1, // 6881
                127, 0, 0, 1, 0x1A, 0xE9,   // 6889
            ]);

            data.extend(b"e");
            data
        };

        let mock = server.mock(|when, then| {
            when.method("GET");
            then.status(200)
                .body(response);
        });

        let tf = TorrentFile {
            announce: server.url("/"),
            info_hash: [216, 247, 57, 206, 195, 40, 149, 108, 204, 91,
                        191, 31, 134, 217, 253, 207, 219, 168, 206, 182],
            piece_hashes: vec![],
            piece_length: 262144,
            length: 351272960,
            name: "debian.iso".into(),
        };

        let peer_id = [
            1,2,3,4,5,6,7,8,9,10,
            11,12,13,14,15,16,17,18,19,20
        ];

        let peers = request_peers(&tf, peer_id, 6882).unwrap();

        let expected = vec![
            Peer {
                ip: Ipv4Addr::new(192, 0, 2, 123),
                port: 6881,
            },
            Peer {
                ip: Ipv4Addr::new(127, 0, 0, 1),
                port: 6889,
            },
        ];

        assert_eq!(peers, expected);

        mock.assert();
    }
}
