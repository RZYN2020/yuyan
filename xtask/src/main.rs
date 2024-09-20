use clap::{Parser, Subcommand};
use libyuyan::*;
use std::fs::read;
use std::process::Command;
use std::env; 

#[derive(Parser)]
#[clap(name = "yuyan")]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}


#[derive(Subcommand)]
enum Commands {
    /// Read torrent file
    Read {
        #[clap(short, long)]
        file: String,
    },
    /// Read bencode file
    Bencode {
        #[clap(short, long)]
        file: String,
    },
    /// Run UI
    Ui,
}

fn main() { 
    let cli = Cli::parse();

    match &cli.command {
        Commands::Read { file } => {
            let metainfo = metainfo::MetaInfo::from(file).unwrap();
            println!("{:?}", metainfo);
        },
        Commands::Bencode { file } => {
            let bytes = read(file).unwrap();
            let bencode = bencode::BItem::deseri_cons(&bytes).unwrap();
            println!("{:?}", bencode);
        }
        Commands::Ui => {
           env::set_current_dir("./desktop").unwrap();
           Command::new("npm.cmd").arg("run").arg("tauri").arg("dev").spawn().unwrap();
        }
    }
}