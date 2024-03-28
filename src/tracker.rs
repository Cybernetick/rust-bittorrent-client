pub mod tracker {
    use std::net::{IpAddr, SocketAddr};
    use bytes::{Buf, BufMut};
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use tokio::io::{AsyncWriteExt, Interest};
    use tokio::net::TcpStream;
    use crate::metainfo::Meta;
    use crate::peers::Peers;

    // const PEER_ID: String = String::from("00112233445566778899");
    pub async fn connect_to_tracker(meta_data: Meta) {
        let hash = meta_data.calculate_info_hash();
        let tracker_request = TrackerRequest{
            peer_id: "00112233445566778899".to_string(),
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: meta_data.info.length,
            compact: 1
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
                for peer in peers.peers.peers {
                    println!("{}:{}", peer.ip_address, peer.port)
                }
            }
            Err(error) => {
                eprintln!("error sending the request: {}", error)
            }
        }
    }

    pub async fn connect_to_peer(peer_ip: IpAddr, port: u16, meta_data: Meta) {
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

                let write_result = safe_stream.write_all(buf.as_slice()).await;
                match write_result {
                    Ok(_) => {
                        let ready_to_read = safe_stream.ready(Interest::READABLE).await;
                        match ready_to_read {
                            Ok(_) => {
                                let mut response_buf: [u8; 1024] = [0; 1024];
                                let read_size = safe_stream.try_read(&mut response_buf).expect("handshake happened");

                                let peer_hash = &response_buf.take(read_size).into_inner()[read_size - 20..read_size];
                                println!("Peer ID: {}", base16::encode_lower(peer_hash))
                            }
                            Err(err) => {
                                eprintln!("did not receive READABLE state of stream. error: {}", err);
                            }
                        }

                    }
                    Err(err) => {
                        eprintln!("error writing handshake into stream: {}", err);
                    }
                }
                //
            }
            Err(err) => {
                eprintln!("error handshaking {}", err)
            }
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
        pub compact: u8
    }

    #[derive(Debug, Deserialize)]
    struct PeersResponse {
        pub interval: usize,
        pub peers: Peers
    }
}