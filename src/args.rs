use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Command {
    Decode { input: String },
    Info { file_path: PathBuf },
    Peers { file_path: PathBuf },
    Handshake { file_path: PathBuf, peer_address: String },
    DownloadPiece {
        piece_file_path: PathBuf,
        torrent_file_path: PathBuf,
        piece_index: usize,
    },
}