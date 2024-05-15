//! yuyan-torrent commandLine download
//! Leecher mode only

use clap::{Args, Parser, Subcommand};
use yuyan_torrent::bencode::BItem;
use yuyan_torrent::client::Client;
use yuyan_torrent::{client, tracker};
use yuyan_torrent::metainfo::MetaInfo;
use std::fs::read;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    download: bool,

    #[arg(short, long)]
    read: bool,

    #[arg(short, long)]
    bencode: bool,
    
    file: String,

    dst: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    if cli.read {
        let meta_info = MetaInfo::from(&cli.file)?;
        println!("{meta_info}");
    }
    if cli.bencode {
        let bytes = read(&cli.file)?;
        let bencode = BItem::deseri_cons(&bytes)?;
        println!("{}", &bencode);
    }
    if cli.download {
        download(&cli.file, &cli.dst.unwrap_or(".".to_string())).await?;
    }

    Ok(())
}


async fn download(torrent: &str, dst: &str) -> anyhow::Result<()> {
    let meta_info = MetaInfo::from(torrent)?;
    client::download(meta_info, 10006, dst).await
}
