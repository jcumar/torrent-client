use anyhow::{Result, anyhow};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(Vec<u8>),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>,
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    }
}

impl Message {
    pub async fn read(stream: &mut TcpStream) -> Result<Self> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let length = u32::from_be_bytes(len_buf);

        if length == 0 {
            return Ok(Message::KeepAlive);
        }

        let mut payload = vec![0u8; length as usize];
        stream.read_exact(&mut payload).await?;
        let id = payload[0];
        let data = &payload[1..];

        match id {
            0 => Ok(Message::Choke),
            1 => Ok(Message::Unchoke),
            2 => Ok(Message::Interested),
            3 => Ok(Message::NotInterested),
            4 => {
                let index = u32::from_be_bytes(data[..4].try_into()?);
                Ok(Message::Have(index))
            }, 
            5 => Ok(Message::Bitfield(data.to_vec())),
            6 => {
                let index = u32::from_be_bytes(data[..4].try_into()?);
                let begin = u32::from_be_bytes(data[4..8].try_into()?);
                let length = u32::from_be_bytes(data[8..12].try_into()?);
                Ok(Message::Request { index, begin, length })
            },
            7 => {
                let index = u32::from_be_bytes(data[..4].try_into()?);
                let begin = u32::from_be_bytes(data[4..8].try_into()?);
                let block = data[8..].to_vec();
                Ok(Message::Piece { index, begin, block })
            },
            8 => {
                let index = u32::from_be_bytes(data[..4].try_into()?);
                let begin = u32::from_be_bytes(data[4..8].try_into()?);
                let length = u32::from_be_bytes(data[8..12].try_into()?);
                Ok(Message::Cancel { index, begin, length })
            },
            _ => Err(anyhow!("unknown message id {}", id)),
        }
    }

    pub async fn write(&self, stream: &mut TcpStream) -> Result<()> {
        let mut buf: Vec<u8> = vec![];

        match self {
            Message::KeepAlive => {
                stream.write_all(&0u32.to_be_bytes()).await?;
                return Ok(());
            }, 
            Message::Choke => buf.push(0),
            Message::Unchoke => buf.push(1),
            Message::Interested => buf.push(2),
            Message::NotInterested => buf.push(3),
            Message::Have(index) => {
                buf.push(4);
                buf.extend(index.to_be_bytes());
            },
            Message::Bitfield(bits) => {
                buf.push(5);
                buf.extend(bits);
            },
            Message::Request { index, begin, length } => {
                buf.push(6);
                buf.extend(index.to_be_bytes());
                buf.extend(begin.to_be_bytes());
                buf.extend(length.to_be_bytes());
            },
            Message::Piece { index, begin, block } => {
                buf.push(7);
                buf.extend(index.to_be_bytes());
                buf.extend(begin.to_be_bytes());
                buf.extend(block);
            },
            Message::Cancel { index, begin, length } => {
                buf.push(8);
                buf.extend(index.to_be_bytes());
                buf.extend(begin.to_be_bytes());
                buf.extend(length.to_be_bytes());
            },
        }

        let length = buf.len();
        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&buf).await?;

        Ok(())
    }
}
