use serde_json;
use std::env;
use serde_json::Number;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let delimiter = encoded_value.find(':');
    return match delimiter {
        Some(delimited_safe) => {
            let key = encoded_value[..delimited_safe].parse::<usize>().expect("unable to parse key as digit");
            let value = &encoded_value[delimited_safe + 1..=delimited_safe + key];
            serde_json::Value::String(String::from(value))
        }
        None => {
            if encoded_value.starts_with('i') {
                let delimiter = encoded_value.find('e').expect("supplied string starts like integer, but missing enclosing symbol");
                let parsed_number = encoded_value[1..delimiter].parse::<i64>().expect("unable to parse integer");
                serde_json::Value::Number(Number::from(parsed_number))
            } else {
                panic!("unknown input {}", encoded_value)
            }
        }
    };
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        decode_bencoded_value("7:sevenoo");
    } else {
        let command = &args[1];

        if command == "decode" {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value);
            println!("{}", decoded_value.to_string());
        } else {
            println!("unknown command: {}", args[1])
        }
    }
}
