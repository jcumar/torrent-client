use std::io::{self, Read};

#[derive(Debug, PartialEq)]
pub struct Handshake {
    pub pstr: String,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Handshake {
        Handshake {
            pstr: String::from("BitTorrent protocol"),
            info_hash,
            peer_id,
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut length_buf = vec![0u8; 1];

        reader.read_exact(&mut length_buf)?;

        let pstrlen = length_buf[0] as usize;

        if pstrlen == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "pstrlen cannot be 0",
            ));
        }

        let mut handshake_buf = vec![0u8; pstrlen + 48];

        reader.read_exact(&mut handshake_buf)?;

        let pstr = String::from_utf8_lossy(
            &handshake_buf[0..pstrlen]
        ).to_string();

        let mut info_hash = [0u8; 20];

        info_hash.copy_from_slice(
            &handshake_buf[pstrlen + 8..pstrlen + 8 + 20]
        );

        let mut peer_id = [0u8; 20];

        peer_id.copy_from_slice(
            &handshake_buf[pstrlen + 8 + 20..]
        );

        Ok(Handshake {
            pstr,
            info_hash,
            peer_id,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.pstr.len() + 49);

        buf.push(self.pstr.len() as u8);
        buf.extend_from_slice(self.pstr.as_bytes());
        buf.extend_from_slice(&[0; 8]);
        buf.extend_from_slice(&self.info_hash);
        buf.extend_from_slice(&self.peer_id);

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample_info_hash() -> [u8; 20] {
        [134, 212, 200, 0, 36, 164, 105, 190, 76, 80,
         188, 90, 16, 44, 247, 23, 128, 49, 0, 116]
    }

    fn sample_peer_id() -> [u8; 20] {
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
         11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
    }

    #[test]
    fn test_new() {
        let info_hash = sample_info_hash();
        let peer_id = sample_peer_id();

        let h = Handshake::new(info_hash, peer_id);

        let expected = Handshake {
            pstr: "BitTorrent protocol".to_string(),
            info_hash,
            peer_id,
        };

        assert_eq!(expected, h);
    }

    #[test]
    fn test_serialize() {
        let tests = vec![
            (
                Handshake {
                    pstr: "BitTorrent protocol".to_string(),
                    info_hash: sample_info_hash(),
                    peer_id: sample_peer_id(),
                },
                vec![
                    19, 66, 105, 116, 84, 111, 114, 114, 101, 110,
                    116, 32, 112, 114, 111, 116, 111, 99, 111, 108,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    134, 212, 200, 0, 36, 164, 105, 190, 76, 80,
                    188, 90, 16, 44, 247, 23, 128, 49, 0, 116,
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
                    13, 14, 15, 16, 17, 18, 19, 20
                ],
            ),
            (
                Handshake {
                    pstr: "BitTorrent protocol, but cooler?".to_string(),
                    info_hash: sample_info_hash(),
                    peer_id: sample_peer_id(),
                },
                vec![
                    32, 66, 105, 116, 84, 111, 114, 114, 101, 110,
                    116, 32, 112, 114, 111, 116, 111, 99, 111, 108,
                    44, 32, 98, 117, 116, 32, 99, 111, 111, 108,
                    101, 114, 63,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    134, 212, 200, 0, 36, 164, 105, 190, 76, 80,
                    188, 90, 16, 44, 247, 23, 128, 49, 0, 116,
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
                    13, 14, 15, 16, 17, 18, 19, 20
                ],
            ),
        ];

        for (input, expected) in tests {
            let buf = input.serialize();
            assert_eq!(expected, buf);
        }
    }

    #[test]
    fn test_read() {
        let tests = vec![
            (
                vec![
                    19, 66, 105, 116, 84, 111, 114, 114, 101, 110,
                    116, 32, 112, 114, 111, 116, 111, 99, 111, 108,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    134, 212, 200, 0, 36, 164, 105, 190, 76, 80,
                    188, 90, 16, 44, 247, 23, 128, 49, 0, 116,
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
                    13, 14, 15, 16, 17, 18, 19, 20
                ],
                Some(Handshake {
                    pstr: "BitTorrent protocol".to_string(),
                    info_hash: sample_info_hash(),
                    peer_id: sample_peer_id(),
                }),
            ),
            (vec![], None),
            (
                vec![
                    19, 66, 105, 116, 84, 111, 114, 114, 101,
                    110, 116, 32, 112, 114, 111, 116, 111, 99, 111
                ],
                None,
            ),
            (vec![0, 0, 0], None),
        ];

        for (input, expected) in tests {
            let mut reader = Cursor::new(input);
            let result = Handshake::read(&mut reader);

            match expected {
                Some(expected_handshake) => {
                    assert!(result.is_ok());
                    assert_eq!(expected_handshake, result.unwrap());
                }
                None => {
                    assert!(result.is_err());
                }
            }
        }
    }
}
