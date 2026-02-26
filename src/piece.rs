use anyhow::{Result, anyhow};
use sha1::{Sha1, Digest};
use crate::peer::PeerConnection;
use crate::Message;

const BLOCK_SIZE: u32 = 16 * 1024;

pub async fn download_piece(
    peer: &mut PeerConnection,
    piece_index: u32,
    piece_length: u32,
    expected_hash: [u8; 20],
) -> Result<Vec<u8>> {
    if !peer.has_piece(piece_index) {
        return Err(anyhow!("peer doesn't have piece"));
    }


    let mut piece = vec![0u8; piece_length as usize];
    let mut offset = 0;

    while offset < piece_length {
        let block_size = std::cmp::min(BLOCK_SIZE, piece_length - offset);
        peer.send_request(piece_index, offset, block_size).await?;

        loop {
            let msg = peer.read_message().await?; 

            match msg {
                Message::Piece { index, begin, block } => {
                    if index != piece_index {
                        continue;
                    }

                    piece[begin as usize..(begin as usize + block.len())]
                        .copy_from_slice(&block);

                    break;
                },
                Message::Choke => {
                    return Err(anyhow!("peer choked us"));
                },
                _ => {},
            }
        }

        offset += block_size;
    }

    verify_piece(&piece, expected_hash)?;

    Ok(piece)
}

fn verify_piece(data: &[u8], expected_hash: [u8; 20]) -> Result<()> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let hash = hasher.finalize();

    if hash[..] != expected_hash {
        return Err(anyhow!("piece hash mismatch"));
    }

    Ok(())
}
