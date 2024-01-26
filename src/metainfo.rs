#![allow(unused_variables, dead_code)]
use std::fmt::{Debug, Display, Formatter};
use serde::Deserialize;

#[derive(Debug)]
pub struct Meta {
    pub announce: String,
    pub info: Info
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub length: usize,
    pub name: String,
    #[serde(rename= "piece length")] pub piece_length: usize,
    //TODO pieces: Vec<u8>
}

impl Meta {
    pub fn create(announce_url: String, info: Info) -> Meta {
        Meta {
            announce: announce_url,
            info
        }
    }

    pub fn from(json: &serde_json::Value) -> Meta {
        let a = &json["info"];
        let info: Info = serde_json::from_value(a.clone()).expect("");
        Meta {
            announce: String::from(json["announce"].as_str().expect("missing \"announce\" key in source JSON")),
            info
        }
    }
}

impl Info {
    pub fn create(length: usize, name: String, pieces_length: usize, pieces: &[u8]) -> Info {
        Info {
            length,
            name,
            piece_length: pieces_length,
            //TODO pieces: Vec::from(pieces),
        }
    }
}

impl Display for Meta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(announce: {}, info: {})", self.announce, self.info)
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "lenght: {}, name: {}, pieces length: {}", self.length, self.name, self.piece_length)
    }
}