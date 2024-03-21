use std::{env, vec};
use anyhow;
use std::path::{Path, PathBuf};
use anyhow::Context;
use serde_json;
use serde_json::Number;
use sha1::{Digest, Sha1};
use clap::Parser;
use crate::metainfo::Meta;

mod metainfo;
mod args;

fn decode_bencoded_string(encoded_string: &str) -> (serde_json::Value, usize) {
    return match encoded_string.chars().nth(0).expect("fail to create iterator over input string") {
        'i' => {
            let delimiter = encoded_string.find('e').expect("supplied string starts like integer, but missing enclosing symbol");
            let parsed_number = encoded_string[1..delimiter].parse::<i64>().expect("unable to parse integer");
            (serde_json::Value::Number(Number::from(parsed_number)), delimiter + 1)
        }
        item if item.is_digit(10) => {
            let delimiter = encoded_string.find(':');
            match delimiter {
                None => {
                    panic!("malformed input. expecting a string, but cannot find a semicolon delimiter for {}", encoded_string)
                }
                Some(delimiter_safe) => {
                    let key = encoded_string[..delimiter_safe].parse::<usize>().expect("unable to parse key as digit");
                    let mut bytes: Vec<u8> = vec![];
                    let mut size = 0;
                    let _ = &encoded_string[delimiter_safe + 1..].char_indices().take(key).for_each(|item| {
                        bytes.push(item.1 as u8);
                        size = item.0
                    });
                    size += 1;
                    let safe_string = String::from_utf8(bytes.clone());
                    return match safe_string {
                        Ok(formatted) => { (serde_json::Value::String(formatted), (delimiter_safe + size + 1)) }
                        Err(_) => { (serde_json::Value::from(bytes), (delimiter_safe + size + 1)) }
                    };
                }
            }
        }
        'l' => {
            let mut total: Vec<serde_json::Value> = vec![];
            let mut cursor: usize = 1;
            while encoded_string.chars().nth(cursor).unwrap() != 'e' {
                let (result, size) = decode_bencoded_string(&encoded_string[cursor..encoded_string.len()]);
                total.push(result);
                cursor += size;
            }

            (serde_json::Value::Array(total), cursor + 1)
        }
        'd' => {
            let mut result = serde_json::Map::new();
            let mut cursor: usize = 1;
            while encoded_string.chars().nth(cursor).is_some() && encoded_string.chars().nth(cursor).unwrap() != 'e' {
                let (key, size) = decode_bencoded_string(&encoded_string[cursor..encoded_string.len()]);
                cursor += size;
                let (value, size) = decode_bencoded_string(&encoded_string[cursor..encoded_string.len()]);
                cursor += size;
                result.insert(key.as_str().expect("cannot unwrap key").to_string(), value);
            }
            (result.into(), cursor + 1)
        }
        'e' => {
            (serde_json::Value::Null, 0)
        }
        _ => {
            panic!("unknown input {}", encoded_string)
        }
    };
}

fn read_sample_file(file: &Path) -> anyhow::Result<Meta, anyhow::Error> {
    let torrent_file = std::fs::read(file).context("parse torrent file")?;
    let parsed = serde_bencode::from_bytes(&torrent_file).context("parse torrent file");
    match parsed {
        Ok(meta) => { Ok(meta) }
        Err(body) => { Err(body) }
    }
}

fn calculate_info_hash(info: &metainfo::Info) -> String {
    let encoded = serde_bencode::to_bytes(info).expect("failed to serialize info");

    return calculate_hash_hexed(&encoded)
}

fn calculate_hash_hexed(input: &Vec<u8>) -> String {
    let output = Sha1::digest(input);
    base16::encode_lower(&output)
}

fn main() {
    let formatted = args::Args::parse();
    match &formatted.command {
        args::Command::Decode { input } => {
            let decoded_value = decode_bencoded_string(input);
            println!("{}", decoded_value.0.to_string());
        }
        args::Command::Info { file_path } => {
            let mut path = if env::args().any(|item| item == "--directory") {
                PathBuf::from(env::args().last().unwrap())
            } else {
                env::current_dir().unwrap_or(PathBuf::new())
            };
            path = path.join(Path::new(file_path));
            let result = read_sample_file(path.as_path());
            match result {
                Ok(content) => {
                    let hash_hexed = calculate_info_hash(&content.info);
                    println!("Tracker URL: {}\n Length: {}\n Info Hash: {}", content.announce, content.info.length, hash_hexed);
                    println!("Piece Length: {}", content.info.piece_length);
                    let mut iterator = content.info.pieces.chunks_exact(20);
                    iterator.clone().for_each(|chunk| println!("{}", base16::encode_lower(&chunk.to_vec())));
                    if !iterator.next().is_none() {
                        println!("{}", base16::encode_lower(&iterator.remainder().to_vec()))
                    }
                }
                Err(err) => {
                    panic!("failed to parse torrent file. error: {}", err)
                }
            }
        }
    }
}
