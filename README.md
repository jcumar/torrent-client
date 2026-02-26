# torrent-client

A minimal BitTorrent client written in Rust.

This project implements the core BitTorrent protocol stack, including:

- `.torrent` file parsing (bencode)
- Correct `info_hash` computation
- HTTP tracker communication (compact peer list)
- Peer TCP handshake
- Bitfield parsing and choke state tracking
- Length-prefixed message framing
- Block-based piece downloading (16KB blocks)
- SHA-1 piece integrity verification
- Sequential single-peer download orchestration

This is a minimal educational implementation and does not yet implement
full swarm behavior or advanced protocol features.

## Features

### Torrent Parsing
- Extracts metadata
- Computes SHA-1 hash of raw `info` dictionary
- Splits piece hashes correctly

### Tracker Support
- HTTP announce
- Compact peer response decoding

### Peer Protocol
- Handshake implementation
- Message serialization/deserialization
- Bitfield handling
- HAVE message updates
- Choke / Unchoke tracking

### Download Engine
- Requests pieces in 16KB blocks
- Handles last-piece sizing correctly
- Verifies piece integrity using SHA-1
- Writes output to file sequentially

## Current Limitations

This client currently:

- Uses a single peer
- Downloads pieces sequentially
- Supports HTTP trackers only
- Does not implement rarest-first selection
- Does not support multi-file torrents
- Does not implement DHT
- Does not support magnet links
- Does not upload data to peers
- Has minimal timeout and error recovery logic

## Usage

```bash
cargo run
```

## License 

Licensed under either of

- Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE)
- MIT License [LICENSE-MIT](LICENSE-MIT)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
