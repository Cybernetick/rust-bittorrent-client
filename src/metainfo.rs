use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Serialize};

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