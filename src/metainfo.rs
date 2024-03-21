use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub announce: String,
    pub info: Info
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub name: String,
    pub length: usize,
    #[serde(rename= "piece length")] pub piece_length: usize,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>
}

impl Display for Meta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(announce: {}, info: {})", self.announce, self.info)
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "length: {}, name: {}, pieces length: {}", self.length, self.name, self.piece_length)
    }
}

impl Meta {
    pub fn read_from_file(file_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let torrent_file = std::fs::read(file_path).context("parse torrent file")?;
        let parsed = serde_bencode::from_bytes(&torrent_file).context("parse torrent file");
        match parsed {
            Ok(meta) => { Ok(meta) }
            Err(body) => { Err(body) }
        }
    }

    pub fn calculate_info_hash_hexed(&self) -> String {
        let encoded = serde_bencode::to_bytes(&self.info).expect("failed to serialize info");
        let output = Sha1::digest(encoded);
        base16::encode_lower(&output)
    }

    pub fn calculate_info_hash(&self) -> Vec<u8> {
        let encoded = serde_bencode::to_bytes(&self.info).expect("failed to serialize info");
        Vec::from(Sha1::digest(encoded).as_slice())
    }
}