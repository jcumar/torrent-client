use torrent_client::{tracker, TorrentFile, Peer};
use httpmock::MockServer;
use std::net::Ipv4Addr;

#[tokio::test]
async fn test_request_peers() {
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
        pieces: vec![],
        piece_length: 262144,
        length: 351272960,
        name: "debian.iso".into(),
    };

    let peer_id = [
        1,2,3,4,5,6,7,8,9,10,
        11,12,13,14,15,16,17,18,19,20
    ];

    let peers = tracker::request_peers(&tf, peer_id, 6882)
        .await
        .unwrap();

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
