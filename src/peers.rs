use std::fmt::Formatter;
use std::net::{IpAddr, Ipv4Addr};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct PeersContainer {
    pub peers: Vec<Peer>,
}

struct PeersVisitor;

#[derive(Debug)]
pub struct Peer {
    pub ip_address: IpAddr,
    pub port: u16,
}

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = PeersContainer;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("list of peers. 6 bytes per peer, 4 bytes for IP address, 2 bytes for port number")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
        let items: Vec<Peer> = v.chunks_exact(6).map(|chunk| {
            Peer {
                ip_address: IpAddr::from(Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3])),
                port: u16::from_be_bytes([chunk[4], chunk[5]]),
            }
        }).collect();
        Ok(PeersContainer {
            peers: items
        })
    }
}

impl<'de> Deserialize<'de> for PeersContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

/*
    Peer messages consist of a message length prefix (4 bytes), message id (1 byte) and a payload (variable size).
*/
#[derive(Eq, PartialEq, Debug)]
pub enum PeerMessageTag {
    Heartbeat = 0,
    Unchoke = 1,
    Interested = 2,
    Bitfield = 5,
    Request = 6,
    Piece = 7
}

pub struct PeerMessage {
    pub tag: PeerMessageTag,
    pub payload: Vec<u8>
}


pub struct Request {
    index: [u8; 4],
    begin_offset: [u8; 4],
    length: [u8; 4]
}

impl Request {
    pub fn new(index: u32, begin_offset: u32, length: u32) -> Self {
        Request {
            index: index.to_be_bytes(),
            begin_offset: begin_offset.to_be_bytes(),
            length: length.to_be_bytes()
        }
    }

    pub fn as_bytes_mute(&self) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];
        result.extend(self.index);
        result.extend(self.begin_offset);
        result.extend(self.length);
        result
    }
}