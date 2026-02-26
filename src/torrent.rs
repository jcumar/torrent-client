use std::fs;
use anyhow::Result;
use serde_bytes::ByteBuf;
use serde::{Serialize, Deserialize};
use sha1::{Sha1, Digest};

#[derive(Debug, PartialEq, Deserialize)]
pub struct TorrentFile {
    pub announce: String,
    pub info_hash: [u8; 20],
    pub name: String,
    pub piece_length: u32,
    pub pieces: Vec<[u8; 20]>,
    pub length: u64,
}

#[derive(Deserialize)]
struct Torrent {
    announce: String,
    info: Info,
}

#[derive(Serialize, Deserialize)]
struct Info {
    name: String,
    #[serde(rename = "piece length")]
    piece_length: u32,
    pieces: ByteBuf,
    length: u64,
}

impl TorrentFile {
    pub fn from_file(path: &str) -> Result<Self> {
        let bytes = fs::read(path)?;

        let torrent: Torrent = serde_bencode::from_bytes(&bytes)?;

        let info_bytes = serde_bencode::to_bytes(&torrent.info)?;

        let info_hash = hash(&info_bytes);

        let pieces = split_pieces(&torrent.info.pieces);

        Ok(TorrentFile {
            announce: torrent.announce,
            info_hash,
            name: torrent.info.name,
            piece_length: torrent.info.piece_length,
            pieces,
            length: torrent.info.length,
        })
    }
    
    pub fn piece_length(&self, index: usize) -> u32 {
        if index == self.pieces.len() - 1 {
            let remaining =
                self.length as u32 % self.piece_length;

            if remaining == 0 {
                self.piece_length
            } else {
                remaining
            }
        } else {
            self.piece_length
        }
    }
}

fn hash(info_bytes: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(info_bytes);
    hasher.finalize().into()
}

fn split_pieces(pieces: &[u8]) -> Vec<[u8; 20]> {
    pieces
        .chunks(20)
        .map(|chunk| {
            let mut hash = [0u8; 20];
            hash.copy_from_slice(chunk);
            hash
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_info_bytes() {
        let info = Info {
            pieces: ByteBuf::from("1234567890abcdefghijabcdefghij1234567890"),
            piece_length: 262144,
            length: 351272960,
            name: "debian-10.2.0-amd64-netinst.iso".to_string(),
        };
        let info_bytes = serde_bencode::to_bytes(&info).unwrap();
        let info_hash = [216, 247, 57, 206, 195, 40, 149, 108, 204, 91, 191, 31, 134, 217, 253, 207, 219, 168, 206, 182];

        assert_eq!(hash(&info_bytes), info_hash);
    }

    #[test]
    fn split_into_two_pieces() {
        let pieces = ByteBuf::from("1234567890abcdefghijabcdefghij1234567890");
        let expected = vec![
            [49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106],
            [97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48],
        ];

        assert_eq!(split_pieces(&pieces), expected);
    }
}
