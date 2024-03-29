use std::fmt::Formatter;
use std::net::{IpAddr, Ipv4Addr};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Peers {
    pub peers: Vec<Peer>,
}

struct PeersVisitor;

#[derive(Debug)]
pub struct Peer {
    pub ip_address: IpAddr,
    pub port: u16,
}

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("list of peers")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
        let items: Vec<Peer> = v.chunks_exact(6).map(|chunk| {
            Peer {
                ip_address: IpAddr::from(Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3])),
                port: u16::from_be_bytes([chunk[4], chunk[5]]),
            }
        }).collect();
        Ok(Peers {
            peers: items
        })
    }
}

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}