use std::{env, vec};
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use serde_json;
use serde_json::Number;
use clap::Parser;
use crate::metainfo::Meta;

mod metainfo;
mod args;
mod tracker;
mod peers;

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

#[tokio::main]
async fn main() {
    let formatted = args::Args::parse();
    match &formatted.command {
        args::Command::Decode { input } => {
            let decoded_value = decode_bencoded_string(input);
            println!("{}", decoded_value.0.to_string());
        }
        args::Command::Info { file_path } => {
            let result = read_meta_from_args_filepath(file_path);
            match result {
                Ok(content) => {
                    let hash_hexed = content.calculate_info_hash_hexed();
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
        args::Command::Peers { file_path } => {
            use tracker::tracker::connect_to_tracker;
            let result = read_meta_from_args_filepath(file_path);
            match result {
                Ok(meta_data) => {
                    connect_to_tracker(&meta_data).await.expect("Failed to load peers list for tracker");
                }
                Err(err) => {
                    panic!("failed to parse torrent file. error: {}", err)
                }
            }
        }

        args::Command::Handshake { file_path, peer_address } => {
            use tracker::tracker::connect_to_peer;
            let file = read_meta_from_args_filepath(file_path);
            match file {
                Ok(meta_data) => {
                    let mut address_iterator = peer_address.as_str().to_socket_addrs().expect("invalid address supplied");
                    let address = address_iterator.next();
                    match address {
                        None => {
                            eprintln!("address iterator is empty")
                        }
                        Some(addr) => {
                            connect_to_peer(addr.ip(), addr.port(), &meta_data).await.expect("handshake failed");
                        }
                    }
                }
                Err(_) => {}
            }
        }

        args::Command::DownloadPiece { piece_file_path, torrent_file_path, piece_index } => {
            use tracker::tracker::download_piece;
            let file = read_meta_from_args_filepath(torrent_file_path);

            match file {
                Ok(meta_data) => {
                    download_piece(piece_file_path, meta_data, piece_index).await.expect("failed downloading");
                }
                Err(_err) => {
                }
            }
        }
    }
}

fn get_current_dir_path() -> PathBuf {
    let path = if env::args().any(|item| item == "--directory") {
        PathBuf::from(env::args().last().unwrap())
    } else {
        env::current_dir().unwrap_or(PathBuf::new())
    };
    path
}

fn read_meta_from_args_filepath(file_name: &PathBuf) -> Result<Meta, anyhow::Error> {
    let mut path = get_current_dir_path();
    path = path.join(Path::new(file_name));
    Meta::read_from_file(&path)
}
