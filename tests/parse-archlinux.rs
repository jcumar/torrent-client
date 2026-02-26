use torrent_client::TorrentFile;
use std::fs;

#[test]
fn parse_archlinux_torrent() {
    let torrent_path = "tests/testdata/archlinux-2019.12.01-x86_64.iso.torrent";
    let torrent = TorrentFile::from_file(torrent_path).unwrap(); 
    
    let golden_path = "tests/testdata/archlinux-2019.12.01-x86_64.iso.torrent.golden.json";
    let raw_json = fs::read_to_string(golden_path).unwrap();
    let golden_torrent: TorrentFile = serde_json::from_str(&raw_json).unwrap();

    assert_eq!(torrent, golden_torrent);
}
