use std::io::{self, Read};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MessageID {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl TryFrom<u8> for MessageID {
    type Error = ();

    fn try_from(v: u8) -> Result<MessageID, Self::Error> {
        match v {
            0 => Ok(MessageID::Choke),
            1 => Ok(MessageID::Unchoke),
            2 => Ok(MessageID::Interested),
            3 => Ok(MessageID::NotInterested),
            4 => Ok(MessageID::Have),
            5 => Ok(MessageID::Bitfield),
            6 => Ok(MessageID::Request),
            7 => Ok(MessageID::Piece),
            8 => Ok(MessageID::Cancel),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub id: MessageID,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn format_request(index: u32, begin: u32, length: u32) -> Message {
        let mut payload = Vec::with_capacity(12);
        
        payload.extend_from_slice(&index.to_be_bytes());
        payload.extend_from_slice(&begin.to_be_bytes());
        payload.extend_from_slice(&length.to_be_bytes());

        Message {
            id: MessageID::Request,
            payload,
        }
    }
    
    pub fn format_have(index: u32) -> Message {
        let mut payload = Vec::with_capacity(4);

        payload.extend_from_slice(&index.to_be_bytes());

        Message {
            id: MessageID::Have, 
            payload,
        }
    }

    pub fn parse_piece(&self, expected_index: u32, buf: &mut [u8]) -> crate::Result<usize> {
        if self.id != MessageID::Piece {
            return Err(format!("Expected PIECE (ID {:?}), got ID {:?}", MessageID::Piece, self.id).into());
        }

        if self.payload.len() < 8 {
            return Err(format!("Payload too short. {} < 8", self.payload.len()).into());
        }

        let parsed_index = u32::from_be_bytes(self.payload[0..4].try_into().unwrap());

        if parsed_index != expected_index {
            return Err(format!("Expected index {}, got {}", expected_index, parsed_index).into());
        }

        let begin = u32::from_be_bytes(self.payload[4..8].try_into().unwrap()) as usize;

        if begin >= buf.len() {
            return Err(format!("Begin offset too high. {} >= {}", begin, buf.len()).into());
        }

        let data = &self.payload[8..];

        if begin + data.len() > buf.len() {
            return Err(format!(
                "Data too long [{}] for offset {} with length {}",
                data.len(), begin, buf.len()
            ).into());
        }

        buf[begin..begin + data.len()].copy_from_slice(data);

        Ok(data.len())
    }

    pub fn parse_have(&self) -> crate::Result<u32> {
        if self.id != MessageID::Have {
            return Err(format!("Expected HAVE (ID {:?}), got ID {:?}", MessageID::Have, self.id).into());
        }

        if self.payload.len() != 4 {
            return Err(format!("Expected payload length 4, got length {}", self.payload.len()).into());
        }

        let index = u32::from_be_bytes(self.payload[..].try_into().unwrap());

        Ok(index)
    }

    pub fn serialize(message: Option<&Message>) -> Vec<u8> {
        match message {
            None => vec![0, 0, 0, 0],
            Some(m) => {
                let length = (m.payload.len() + 1) as u32;

                let mut buf = Vec::with_capacity(4 + length as usize);
                buf.extend_from_slice(&length.to_be_bytes());
                buf.push(m.id as u8);
                buf.extend_from_slice(&m.payload);

                buf
            }
        }
    }
    
    pub fn read<R: Read>(r: &mut R) -> io::Result<Option<Self>> {
        let mut length_buf = [0u8; 4];

        r.read_exact(&mut length_buf)?;

        let length = u32::from_be_bytes(length_buf);

        if length == 0 {
            return Ok(None);
        }

        let mut message_buf = vec![0u8; length as usize];

        r.read_exact(&mut message_buf)?;

        let id = MessageID::try_from(message_buf[0])
            .map_err(|_| io::Error::new(
                io::ErrorKind::InvalidData, "Unknown message ID"
            ))?;

        Ok(Some(Message {
            id,
            payload: message_buf[1..].to_vec(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_format_request() {
        let msg = Message::format_request(4, 567, 4321);
        let expected_payload = vec![
            0x00, 0x00, 0x00, 0x04, // Index
            0x00, 0x00, 0x02, 0x37, // Begin
            0x00, 0x00, 0x10, 0xe1, // Length
        ];
        assert_eq!(msg.id, MessageID::Request);
        assert_eq!(msg.payload, expected_payload);
    }

    #[test]
    fn test_format_have() {
        // Assuming you have a Message::format_have helper
        let msg = Message::format_have(4); 
        assert_eq!(msg.id, MessageID::Have);
        assert_eq!(msg.payload, vec![0x00, 0x00, 0x00, 0x04]);
    }

    #[test]
    fn test_parse_piece() {
        struct TestCase {
            name: &'static str,
            input_index: u32,
            input_buf: Vec<u8>,
            input_msg: Message,
            output_n: usize,
            output_buf: Vec<u8>,
            fails: bool,
        }

        let tests = vec![
            TestCase {
                name: "parse valid piece",
                input_index: 4,
                input_buf: vec![0u8; 10],
                input_msg: Message {
                    id: MessageID::Piece,
                    payload: vec![
                        0x00, 0x00, 0x00, 0x04, // Index
                        0x00, 0x00, 0x00, 0x02, // Begin
                        0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // Block
                    ],
                },
                output_n: 6,
                output_buf: vec![0x00, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x00],
                fails: false,
            },
            TestCase {
                name: "wrong message type",
                input_index: 4,
                input_buf: vec![0u8; 10],
                input_msg: Message { id: MessageID::Choke, payload: vec![] },
                output_n: 0,
                output_buf: vec![0u8; 10],
                fails: true,
            },
            // ... (other cases follow same pattern)
        ];

        for mut test in tests {
            let result = test.input_msg.parse_piece(test.input_index, &mut test.input_buf);
            if test.fails {
                assert!(result.is_err(), "Test '{}' should have failed", test.name);
            } else {
                assert_eq!(result.unwrap(), test.output_n, "Test '{}' failed length", test.name);
                assert_eq!(test.input_buf, test.output_buf, "Test '{}' failed buffer match", test.name);
            }
        }
    }

    #[test]
    fn test_serialize() {
        let msg = Message { id: MessageID::Have, payload: vec![1, 2, 3, 4] };
        
        // Test normal message
        let buf = Message::serialize(Some(&msg));
        assert_eq!(buf, vec![0, 0, 0, 5, 4, 1, 2, 3, 4]);

        // Test keep-alive
        let keep_alive = Message::serialize(None);
        assert_eq!(keep_alive, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_read() {
        let input = vec![0, 0, 0, 5, 4, 1, 2, 3, 4];
        let mut reader = Cursor::new(input);
        let m = Message::read(&mut reader).unwrap().unwrap();
        
        assert_eq!(m.id, MessageID::Have);
        assert_eq!(m.payload, vec![1, 2, 3, 4]);

        // Test keep-alive
        let mut ka_reader = Cursor::new(vec![0, 0, 0, 0]);
        let ka = Message::read(&mut ka_reader).unwrap();
        assert!(ka.is_none());
    }
}
