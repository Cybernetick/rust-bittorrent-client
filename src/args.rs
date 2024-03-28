use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Decode { input: String },
    Info { file_path: PathBuf },
    Peers { file_path: PathBuf },
    Handshake { file_path: PathBuf, peer_address: String }
}