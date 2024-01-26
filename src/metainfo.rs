pub struct Meta {
    pub announce: String,
    pub info: Info
}

pub struct Info {
    length: usize,
    name: String,
    piece_length: usize,
    pieces: Vec<u8>
}

impl Meta {
    pub fn create(announce_url: String, info: Info) -> Meta {
        Meta {
            announce: announce_url,
            info
        }
    }

    // pub fn from(json: serde_json::Value) -> Meta {
    //
    // }
}

impl Info {
    pub fn create(length: usize, name: String, pieces_length: usize, pieces: &[u8]) -> Info {
        Info {
            length,
            name,
            piece_length: pieces_length,
            pieces: Vec::from(pieces),
        }
    }
}