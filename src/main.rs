use std::{env, vec};
use anyhow;
use std::hash::Hash;
use std::io::{Read};
use std::path::{Path, PathBuf};
use anyhow::Context;
use serde::de::Error;
use serde_json;
use serde_json::Number;
use sha1::Digest;
use crate::metainfo::Meta;

mod metainfo;

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
        Ok(Meta) => { Ok(Meta) }
        Err(body) => { Err(body) }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = &args[1];
    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_string(encoded_value);
            println!("{}", decoded_value.0.to_string());
        }
        "info" => {
            let mut path = if env::args().any(|item| item == "--directory") {
                PathBuf::from(env::args().last().unwrap())
            } else {
                env::current_dir().unwrap_or(PathBuf::new())
            };
            path = path.join(Path::new(&args[2]));
            let result = read_sample_file(path.as_path());
            match result {
                Ok(content) => {
                    println!("Tracker URL: {}\n Length: {}", content.announce, content.info.length);
                }
                Err(err) => {
                    panic!("failed to parse torrent file. error: {}", err)
                }
            }
        }
        _ => {
            panic!("unknown command provided: {}", command)
        }
    }
}
