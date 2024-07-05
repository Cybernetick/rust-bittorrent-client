pub mod tracker {
    use std::io::{Error, ErrorKind, Read};
    use std::net::{IpAddr, SocketAddr};
    use std::path::PathBuf;

    use bytes::{Buf, BufMut, BytesMut};
    use reqwest::{Client};
    use serde::{Deserialize, Serialize};
    use sha1::Digest;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest};
    use tokio::net::TcpStream;
    use crate::metainfo::Meta;
    use crate::peers::{PeerMessage, PeerMessageTag, PeersContainer, Request};

    const MAX_BLOCK_SIZE: usize = 1 << 14;

    pub async fn connect_to_tracker(meta_data: &Meta) -> Result<PeersContainer, Error> {
        let hash = meta_data.calculate_info_hash();
        let tracker_request = TrackerRequest {
            peer_id: "00112233445566778899".to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: meta_data.info.length,
            compact: 1,
        };
        let url_params = serde_urlencoded::to_string(tracker_request).expect("url params encode failed");
        let tracker_request = format!(
            "{}?{}&info_hash={}",
            meta_data.announce,
            url_params,
            &url_encode(&hash)
        );
        let client = Client::new();
        //cannot use query() method since it does url-encode differently for hashes
        // let body = client.request(Method::GET, meta_data.announce)
        //     .query(&[("peer_id", String::from("00112233445566778899")), ("info_hash", url_encode(&hash)), ("port", String::from("6881")), ("uploaded", String::from("0")), ("downloaded", String::from("0")), ("left", meta_data.info.length.to_string()), ("compact", String::from("1"))])
        //     .build();
        let body = client.get(tracker_request).build();
        let result = client.execute(body.unwrap()).await;
        match result {
            Ok(response) => {
                let body = response.bytes().await.unwrap();
                let peers: PeersResponse = serde_bencode::from_bytes(body.as_ref()).expect("cannot deserialize response");
                for peer in &peers.peers_container.peers {
                    println!("{}:{}", peer.ip_address, peer.port)
                }
                Ok(peers.peers_container)
            }
            Err(error) => {
                eprintln!("error sending the request: {}", error);
                Err(Error::new(ErrorKind::NotConnected, error))
            }
        }
    }

    pub async fn connect_to_peer(peer_ip: IpAddr, port: u16, meta_data: &Meta) -> Result<TcpStream, Error> {
        let stream = TcpStream::connect(SocketAddr::new(peer_ip, port)).await;
        match stream {
            Ok(mut safe_stream) => {
                let mut buf: Vec<u8> = Vec::new();
                buf.push(19);
                buf.put_slice("BitTorrent protocol".as_bytes());
                let reserve: [u8; 8] = [0; 8];
                buf.put_slice(&reserve);
                buf.put_slice(meta_data.calculate_info_hash().as_slice());
                buf.put_slice("00112233445566778899".as_bytes());

                let write = safe_stream.write_all(buf.as_slice()).await?;
                let mut response_buf: Vec<u8> = vec![];
                response_buf.reserve_exact(buf.len());
                safe_stream.read_exact(&mut response_buf).await?;
                println!("{:?}", response_buf);
                println!();
                Ok(safe_stream)
            }
            Err(err) => {
                eprintln!("error handshaking {}", err);
                Err(err)
            }
        }
    }

    pub async fn download_piece(piece_file_path: &PathBuf, meta_data: Meta, piece_index: &usize) -> Result<(), Error> {
        let peers = connect_to_tracker(&meta_data).await?;
        if !peers.peers.is_empty() {
            let stream = TcpStream::connect(SocketAddr::new(peers.peers[0].ip_address, peers.peers[0].port)).await;
            match stream {
                Ok(mut safe_stream) => {
                    let mut buf: Vec<u8> = Vec::new();
                    buf.push(19);
                    buf.put_slice("BitTorrent protocol".as_bytes());
                    let reserve: [u8; 8] = [0; 8];
                    buf.put_slice(&reserve);
                    buf.put_slice(meta_data.calculate_info_hash().as_slice());
                    buf.put_slice("00112233445566778899".as_bytes());

                    let write = safe_stream.write_all(buf.as_slice()).await?;
                    loop {
                        let ready_to_read = safe_stream.ready(Interest::READABLE).await?;
                        if ready_to_read.is_readable() {
                            let mut response_buf: Vec<u8> = vec![];
                            response_buf.resize(buf.len(), 0);
                            safe_stream.read_exact(&mut response_buf).await?;
                            break;
                        }
                    }

                    let mut connection = FrameConnection::new(safe_stream);
                    let bitfield = connection.read_frame().await.expect("peer should response with bitfield")
                        .ok_or(Error::new(ErrorKind::InvalidData, "peer not responded with bitfield"))?;
                    assert_eq!(bitfield.tag, PeerMessageTag::Bitfield);
                    connection.write_frame(PeerMessage {
                        tag: PeerMessageTag::Interested,
                        payload: Vec::new(),
                    }).await.expect("our client should respond with INTERESTED");
                    let _ = connection.read_frame().await.expect("peer should respond to INTERESTED with UNCHOKE message")
                        .ok_or(Error::new(ErrorKind::InvalidData, "peer not responded with UNCHOKE"))?;

                    let block_count = (meta_data.info.piece_length + (MAX_BLOCK_SIZE - 1)) / MAX_BLOCK_SIZE;
                    let mut data: Vec<u8> = vec![];
                    for block in 0..block_count {
                        let block_size : usize = if block == block_count - 1 {
                            let remainder = meta_data.info.piece_length % MAX_BLOCK_SIZE;
                            if remainder == 0 {
                                MAX_BLOCK_SIZE
                            } else {
                                meta_data.info.piece_length % MAX_BLOCK_SIZE
                            }

                        } else {
                            MAX_BLOCK_SIZE
                        };
                        let request = Request::new(*piece_index as u32, (block * block_size) as u32, block_size as u32);
                        connection.write_frame(PeerMessage {
                            tag: PeerMessageTag::Request,
                            payload: request.as_bytes_mute(),
                        }).await?;
                        let piece = connection.read_frame().await.expect("peer should respond with PIECE")
                            .ok_or(Error::new(ErrorKind::InvalidData, "peer not responded with piece"))?;

                        assert_eq!(piece.tag, PeerMessageTag::Piece);
                        data.extend_from_slice(&piece.payload[8..]);
                    }
                    assert_eq!(data.len(), meta_data.info.piece_length);

                    tokio::fs::write(piece_file_path, data).await?;
                    println!("Piece {} downloaded to {:?}", piece_index, piece_file_path);
                    Ok(())
                }
                Err(_) => {
                    Err(Error::new(ErrorKind::ConnectionRefused, "error opening TCPStream"))
                }
            }
        } else {
            Err(Error::new(ErrorKind::InvalidData, "peers list is empty"))
        }
    }

    fn url_encode(input: &Vec<u8>) -> String {
        let mut encoded = String::with_capacity(3 * input.len());
        for &b in input.as_slice() {
            encoded.push('%');
            encoded.push_str(&hex::encode(&[b]))
        }
        encoded
    }

    #[derive(Debug, Clone, Serialize)]
    struct TrackerRequest {
        pub peer_id: String,
        pub port: u16,
        pub uploaded: usize,
        pub downloaded: usize,
        pub left: usize,
        pub compact: u8,
    }

    #[derive(Debug, Deserialize)]
    struct PeersResponse {
        pub interval: usize,
        #[serde(rename = "peers")]
        pub peers_container: PeersContainer,
    }

    struct FrameConnection {
        stream: TcpStream,
        buffer: BytesMut,
    }

    impl FrameConnection {
        pub fn new(stream: TcpStream) -> FrameConnection {
            FrameConnection {
                stream,
                buffer: BytesMut::with_capacity(1 << 16),
            }
        }

        pub async fn read_frame(&mut self) -> Result<Option<PeerMessage>, Error> {
            loop {
                if let Some(frame) = self.parse_frame()? {
                    return Ok(Some(frame));
                }
                if 0 == self.stream.read_buf(&mut self.buffer).await? {
                    // The remote closed the connection. For this to be
                    // a clean shutdown, there should be no data in the
                    // read buffer. If there is, this means that the
                    // peer closed the socket while sending a frame.
                    return if self.buffer.is_empty() {
                        Ok(None)
                    } else {
                        Err(Error::new(ErrorKind::NotConnected, "connection reset by peer"))
                    };
                }
            }
        }

        fn parse_frame(&mut self) -> Result<Option<PeerMessage>, Error> {
            // length prefix (4 bytes)
            if self.buffer.len() < 4 {
                return Ok(None);
            }

            let mut message_length = [0u8; 4];
            message_length.copy_from_slice(&self.buffer[0..4]);
            let message_length = u32::from_be_bytes(message_length) as usize;

            if message_length == 0 {
                self.buffer.advance(4);
                return self.parse_frame();
            }

            if self.buffer.len() < message_length {
                return Ok(None)
            }

            let message_tag = self.buffer[4];
            let tag = match message_tag {
                0 => PeerMessageTag::Heartbeat,
                1 => PeerMessageTag::Unchoke,
                2 => PeerMessageTag::Interested,
                5 => PeerMessageTag::Bitfield,
                6 => PeerMessageTag::Request,
                7 => PeerMessageTag::Piece,
                tag => {
                    return Err(Error::new(ErrorKind::InvalidData, format!("unknown message tag received {}", tag)));
                }
            };


            let data = if self.buffer.len() >= message_length - 1 {
                self.buffer[5..4 + message_length].to_vec()
            } else {
                vec![]
            };
            self.buffer.advance(message_length + 4);
            Ok(Some(PeerMessage { tag, payload: data.to_vec() }))
        }

        pub async fn write_frame(&mut self, frame: PeerMessage) -> Result<(), Error> {
            let mut complete: Vec<u8> = vec![];
            let len_slice = u32::to_be_bytes(frame.payload.len() as u32 + 1);
            complete.extend_from_slice(&len_slice);
            complete.push(frame.tag as u8);
            complete.extend_from_slice(frame.payload.as_slice());
            self.stream.write_all(complete.as_slice()).await?;
            Ok(())
        }
    }
}