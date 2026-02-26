use anyhow::{Result, anyhow};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use torrent_client::TorrentFile;
use torrent_client::tracker;
use torrent_client::peer::PeerConnection;
use torrent_client::piece;

pub async fn download_torrent(path: &str) -> Result<()> {
    // 1️⃣ Parse torrent
    let torrent = TorrentFile::from_file(path)?;

    // 2️⃣ Generate peer_id
    let peer_id = *b"-RU0001-123456789012";

    // 3️⃣ Get peers
    let peers = tracker::request_peers(&torrent, peer_id, 6881).await?;

    if peers.is_empty() {
        return Err(anyhow!("no peers found"));
    }

    // 4️⃣ Try connecting to peers
    let mut peer_connection = None;

    for peer in peers {
        let addr = format!("{}:{}", peer.ip, peer.port);

        match PeerConnection::connect(&addr, torrent.info_hash, peer_id).await {
            Ok(conn) => {
                peer_connection = Some(conn);
                break;
            }
            Err(_) => continue,
        }
    }

    let mut peer = peer_connection
        .ok_or_else(|| anyhow!("could not connect to any peer"))?;

    // 5️⃣ Interested
    peer.send_interested().await?;

    // 6️⃣ Wait unchoke
    loop {
        peer.read_message_and_update().await?;
        if !peer.is_choked() {
            break;
        }
    }

    // 7️⃣ Create file
    let mut file = File::create(&torrent.name).await?;

    // 8️⃣ Download pieces
    for (i, expected_hash) in torrent.pieces.iter().enumerate() {

        if !peer.has_piece(i as u32) {
            println!("Peer does not have piece {}", i);
            continue;
        }

        let piece_length = torrent.piece_length(i);

        println!("Downloading piece {}", i);

        let piece_data = piece::download_piece(
            &mut peer,
            i as u32,
            piece_length,
            *expected_hash,
        ).await?;

        file.write_all(&piece_data).await?;
    }

    println!("Download complete.");

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let path = "tests/testdata/archlinux-2019.12.01-x86_64.iso.torrent";
    download_torrent(path).await
}
