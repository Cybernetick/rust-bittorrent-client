use std::{env, vec};
use std::fs::File;
use std::hash::Hash;
use std::io::{Read};
use std::path::{Path, PathBuf};
use serde_json;
use serde_json::Number;
use sha1::{Sha1, Digest};
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
                    //iterating over chars since the "pieces" key contains non-utf8 characters, so we cannot convert result into String right away.
                    //non-utf8 characters have different size, so need to pay attention to char_indices value when iterating and counting offset of "handled" string
                    let _ = &encoded_string[delimiter_safe + 1..].char_indices().take(key).for_each(|item| {
                        bytes.push(item.1 as u8);
                        size = item.0
                    });
                    size += 1;
                    let safe_string = String::from_utf8(bytes.clone());
                    return match safe_string {
                        Ok(formatted) => { (serde_json::Value::String(formatted), (delimiter_safe + size + 1))  }
                        Err(_) => { (serde_json::Value::from(bytes), (delimiter_safe + size + 1)) }
                    }

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
            (result.into(), cursor +1)
        }
        'e' => {
            (serde_json::Value::Null, 0)
        }
        _ => {
            panic!("unknown input {}", encoded_string)
        }
    };
}

fn read_sample_file(mut file: &File) {
    let mut file_content: Vec<u8> = vec![];
    match file.read_to_end(&mut file_content) {
        Ok(_) => {
            let file_parsed = core::str::from_utf8(file_content.as_slice());
            match file_parsed {
                Ok(safe) => {
                    let (result, _count) = decode_bencoded_string(&safe);
                    parse_file(result)
                }
                Err(err) => {
                    let limit = err.valid_up_to();
                    let result = decode_bencoded_string(core::str::from_utf8(&file_content[..limit]).unwrap());
                    println!("result is {}", result.0);
                    eprintln!("error parsing file {}", err)
                }
            }
        }
        Err(body) => {
            panic!("error happended while reading file: {}", body);
        }
    }
}

fn parse_file(input_string: serde_json::Value){
    let metainfo = Meta::from(&input_string);
    println!("Tracker URL: {}\n Length: {}", metainfo.announce, metainfo.info.length);
    let encoded = serde_bencode::ser::to_bytes(&metainfo.info);
    match encoded {
        Ok(bytes_array) => {
            let mut digest = Sha1::new();
            digest.update(bytes_array);
            let result = digest.finalize();
            println!("Info Hash: {}", hex::encode(result))
        }
        Err(_) => {}
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
            let file = File::open(path);
            match file {
                Ok(actual) => {
                    read_sample_file(&actual);
                }
                Err(err) => {
                    panic!("error opening file - {}", err)
                }
            }
        }
        _ => {
            panic!("unknown command provided: {}", command)
        }
    }
}
