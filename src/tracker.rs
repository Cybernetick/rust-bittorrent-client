pub mod tracker {
    use reqwest::{Client, Error, Method, Response};
    use crate::metainfo::Meta;

    // const PEER_ID: String = String::from("00112233445566778899");
    pub async fn connect_to_tracker(meta_data: Meta) {
        let hash = meta_data.calculate_info_hash();
        let client = Client::new();
        let body = client.request(Method::GET, meta_data.announce)
            .query(&[("peer_id", String::from("00112233445566778899")), ("port", String::from("6881")), ("uploaded", String::from("0")), ("downloaded", String::from("0")), ("left", meta_data.info.length.to_string()), ("compact", String::from("1"))])
            .build();

        let result = client.execute(body.unwrap()).await;
        match result {
            Ok(response) => {
                println!("response body is {}", response.text().await.unwrap())
            }
            Err(error) => {
                eprintln!("error sending the request: {}", error)
            }
        }

        // tokio::spawn(async move {
        //
        // });
    }
}