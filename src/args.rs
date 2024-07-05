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
    Handshake { file_path: PathBuf, peer_address: String },
    #[command(rename_all = "snake_case")]
    DownloadPiece { piece_file_path: PathBuf, torrent_file_path: PathBuf, piece_index: usize },
}