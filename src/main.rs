use serde_json;
use std::env;
use regex::{Captures, Regex};

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    let reg = Regex::new("(?<length>[0-9]+):(?<value>.+)$").expect("failed to compile RegEx pattern");

    let captures = reg.captures(encoded_value);
    return match captures {
        Some(caps) => {
            let length = usize::from_str_radix(&caps["length"], 10).unwrap_or(0);
            let value = &caps["value"];
            serde_json::Value::String(String::from(value))
        }
        None => {
            serde_json::Value::String(String::new())
        }
    }

    // If encoded_value starts with a digit, it's a number
    // if encoded_value.chars().next().unwrap().is_digit(10) {
    //     // Example: "5:hello" -> "hello"
    //     let colon_index = encoded_value.find(':').unwrap();
    //     let number_string = &encoded_value[..colon_index];
    //     let number = number_string.parse::<i64>().unwrap();
    //     let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];

    // } else {
    //     panic!("Unhandled encoded value: {}", encoded_value)
    // }
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
