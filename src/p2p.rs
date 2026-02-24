use sha1::{Sha1, Digest};
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::sync::Arc;
use crate::client::Client;
use crate::peers::Peer;

const MAX_BLOCK_SIZE: usize = 16384;
const MAX_BACKLOG: usize = 5;

#[derive(Clone)]
pub struct Torrent {
    pub peers: Vec<Peer>,
    pub peer_id: [u8; 20],
    pub info_hash: [u8; 20],
    pub piece_hashes: Vec<[u8; 20]>,
    pub piece_length: usize,
    pub length: usize,
    pub name: String,
}

struct PieceWork {
    index: usize,
    hash: [u8; 20],
    length: usize,
}

struct PieceResult {
    index: usize,
    buf: Vec<u8>,
}

struct PieceProgress<'a> {
    index: usize,
    client: &'a mut Client,
    buf: Vec<u8>,
    downloaded: usize,
    requested: usize,
    backlog: usize,
}

impl Torrent {
    // pub async fn download(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    //     println!("Starting download for {}", self.name);
    //
    //     let (work_tx, mut work_rx) = mpsc::channel(self.piece_hashes.len());
    //     let (res_tx, mut res_rx) = mpsc::channel(self.piece_hashes.len());
    //
    //     // Queue up work
    //     for (index, &hash) in self.piece_hashes.iter().enumerate() {
    //         let length = self.calculate_piece_size(index);
    //         work_tx.send(PieceWork { index, hash, length }).await?;
    //     }
    //
    //     // Start workers (using Arc to share Torrent metadata safely)
    //     let torrent_ref = Arc::new(self.clone());
    //     for peer in self.peers.clone() {
    //         let t = Arc::clone(&torrent_ref);
    //         let w_tx = work_tx.clone();
    //         let r_tx = res_tx.clone();
    //
    //         tokio::spawn(async move {
    //             t.start_download_worker(peer, w_tx, r_tx).await;
    //         });
    //     }
    //
    //     let mut final_buf = vec![0u8; self.length];
    //     let mut done_pieces = 0;
    //
    //     while done_pieces < self.piece_hashes.len() {
    //         if let Some(res) = res_rx.recv().await {
    //             let (begin, _) = self.calculate_bounds_for_piece(res.index);
    //             final_buf[begin..begin + res.buf.len()].copy_from_slice(&res.buf);
    //             done_pieces += 1;
    //
    //             let percent = (done_pieces as f64 / self.piece_hashes.len() as f64) * 100.0;
    //             println!("({:.2}%) Downloaded piece #{}", percent, res.index);
    //         }
    //     }
    //
    //     Ok(final_buf)
    // }

    fn calculate_bounds_for_piece(&self, index: usize) -> (usize, usize) {
        let begin = index * self.piece_length;
        let mut end = begin + self.piece_length;
        if end > self.length {
            end = self.length;
        }
        (begin, end)
    }

    fn calculate_piece_size(&self, index: usize) -> usize {
        let (begin, end) = self.calculate_bounds_for_piece(index);
        end - begin
    }
}

// async fn attempt_download_piece(client: &mut Client, pw: &PieceWork) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
//     let mut state = PieceProgress {
//         index: pw.index,
//         client,
//         buf: vec![0u8; pw.length],
//         downloaded: 0,
//         requested: 0,
//         backlog: 0,
//     };
//
//     // 30 second timeout as in Go code
//     while state.downloaded < pw.length {
//         if !state.client.choked {
//             while state.backlog < MAX_BACKLOG && state.requested < pw.length {
//                 let mut block_size = MAX_BLOCK_SIZE;
//                 if pw.length - state.requested < block_size {
//                     block_size = pw.length - state.requested;
//                 }
//
//                 state.client.send_request(conn, pw.index, state.requested, block_size).await?;
//                 state.backlog += 1;
//                 state.requested += block_size;
//             }
//         }
//
//         // read_message would be implemented to handle MsgPiece and update state
//         timeout(Duration::from_secs(30), state.read_message()).await??;
//     }
//
//     Ok(state.buf)
// }

fn check_integrity(pw: &PieceWork, buf: &[u8]) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(buf);
    let result = hasher.finalize();
    result.as_slice() == pw.hash
}
