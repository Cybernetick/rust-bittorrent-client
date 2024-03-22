pub mod tracker {
    use reqwest::{Client, Error, Method, Response};
    use serde::{Deserialize, Serialize};
    use crate::metainfo::Meta;
    use crate::peers::{Peer, Peers};
    // use crate::peers::Peers;

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