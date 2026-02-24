use std::fs::File;
use std::io::Read;
use sha1::{Sha1, Digest};
use serde::{Serialize, Deserialize};
use serde_bencode::{de, ser};
use crate::{Error, Result};

#[derive(Debug, PartialEq, Deserialize)]
pub struct TorrentFile {
    pub announce: String,
    pub info_hash: [u8; 20],
    pub piece_hashes: Vec<[u8; 20]>,
    pub piece_length: u32,
    pub length: u64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct BencodeTorrent {
    announce: String,
    info: BencodeInfo,
}

impl TryFrom<BencodeTorrent> for TorrentFile {
    type Error = Error;

    fn try_from(bencode_torrent: BencodeTorrent) -> Result<TorrentFile> {
        let info_hash = bencode_torrent.info.hash()?;
        let piece_hashes = bencode_torrent.info.split_piece_hashes()?;
        
        Ok (TorrentFile {
            announce: bencode_torrent.announce,
            info_hash,
            piece_hashes,
            piece_length: bencode_torrent.info.piece_length,
            length: bencode_torrent.info.length,
            name: bencode_torrent.info.name,
        })
    }
}

impl TorrentFile {
    pub fn open(path: &str) -> Result<TorrentFile> {
        let mut buf = Vec::new();
        
        File::open(path)?
            .read_to_end(&mut buf)?;

        let bencode_torrent: BencodeTorrent = de::from_bytes(&buf)?;

        TorrentFile::try_from(bencode_torrent)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BencodeInfo {
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>, 
    #[serde(rename = "piece length")]
    piece_length: u32,
    length: u64,
    name: String,
}

impl BencodeInfo {
    fn hash(&self) -> Result<[u8; 20]> {
        let info_bytes = ser::to_bytes(self)?;
        let mut hasher = Sha1::new();

        hasher.update(&info_bytes);

        Ok(hasher.finalize().into())
    }

    fn split_piece_hashes(&self) -> Result<Vec<[u8; 20]>> {
        let hash_len = 20; 

        if !self.pieces.len().is_multiple_of(hash_len) {
            return Err(
                format!(
                    "Received malformed pieces of length {}", 
                    self.pieces.len()
                ).into()
            );
        } 

        let num_hashes = self.pieces.len() / hash_len;
        let mut hashes = Vec::with_capacity(num_hashes);

        for i in 0..num_hashes {
            hashes.push(
                self.pieces[i * hash_len..(i + 1) * hash_len].try_into()?
            );
        }

        Ok(hashes)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_archlinux_torrent() {
        let torrent_path = "src/testdata/archlinux-2019.12.01-x86_64.iso.torrent";
        let torrent = TorrentFile::open(torrent_path).unwrap(); 
        
        let golden_path = "src/testdata/archlinux-2019.12.01-x86_64.iso.torrent.golden.copy.json";
        let raw_json = fs::read_to_string(golden_path).unwrap();
        let golden_torrent: TorrentFile = serde_json::from_str(&raw_json).unwrap();

        assert_eq!(torrent, golden_torrent);
    }

    #[test] 
    fn from_bencode_to_torrent() {
        let input = BencodeTorrent {
            announce: "http://bttracker.debian.org:6969/announce".to_string(),
            info: BencodeInfo {
                pieces: "1234567890abcdefghijabcdefghij1234567890".to_string().as_bytes().to_vec(),
                piece_length: 262144,
                length: 351272960,
                name: "debian-10.2.0-amd64-netinst.iso".to_string(),
            },
        };

        let expected = TorrentFile {
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

        assert_eq!(TorrentFile::try_from(input).unwrap(), expected);
    }

    #[test] 
    #[should_panic]
    fn from_bencode_to_torrent_panics() {
        let input = BencodeTorrent {
            announce: "http://bttracker.debian.org:6969/announce".to_string(),
            info: BencodeInfo {
                pieces: "1234567890abcdef".to_string().as_bytes().to_vec(),
                piece_length: 262144,
                length: 351272960,
                name: "debian-10.2.0-amd64-netinst.iso".to_string(),
            },
        };

        TorrentFile::try_from(input).unwrap();
    }
}
    #[test] 
    fn from_bencode_to_torrent() {
        let input = BencodeTorrent {
            announce: "http://bttracker.debian.org:6969/announce".to_string(),
            info: BencodeInfo {
                pieces: "1234567890abcdefghijabcdefghij1234567890".to_string().as_bytes().to_vec(),
                piece_length: 262144,
                length: 351272960,
                name: "debian-10.2.0-amd64-netinst.iso".to_string(),
            },
        };

        let expected = TorrentFile {
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

        assert_eq!(TorrentFile::try_from(input).unwrap(), expected);
    }

    #[test] 
    #[should_panic]
    fn torrent_from_bencode_fail() {
        let input = BencodeTorrent {
            announce: "http://bttracker.debian.org:6969/announce".to_string(),
            info: BencodeInfo {
                pieces: "1234567890abcdef".to_string().as_bytes().to_vec(),
                piece_length: 262144,
                length: 351272960,
                name: "debian-10.2.0-amd64-netinst.iso".to_string(),
            },
        };

        TorrentFile::try_from(input).unwrap();
    }
