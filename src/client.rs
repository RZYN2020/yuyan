//! yuyan-torrent Client

use crate::{bitmap::Bitmap, metainfo::MetaInfo, peer, tracker, writer};
use anyhow::{anyhow, bail};
use rand::Rng;
use reqwest::Url;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex, Semaphore};

#[inline]
fn gen_id() -> String {
    let mut id = String::from("-YY0000-");
    for _ in 0..12 {
        let ch: u8 = rand::thread_rng().gen_range(0..=9);
        id.push((('0' as u8) + ch) as char);
    }
    id
}

static MAX_PEER_COUNT: usize = 50;

#[derive(Clone, Debug)]
pub struct Client {
    pub peer_id: String,
    pub length: usize, // piece num
    pub bit_fields: Arc<Mutex<Bitmap>>,
    pub uploaded: Arc<Mutex<usize>>,
    pub http_client: reqwest::Client,
}

impl Client {
    pub fn new(size: usize, length: usize) -> Self {
        Client {
            peer_id: gen_id(),
            length,
            bit_fields: Arc::new(Mutex::new(Bitmap::new(size))),
            uploaded: Arc::new(Mutex::new(0)),
            http_client: reqwest::Client::new(),
        }
    }
}

pub async fn download(metainfo: MetaInfo, port: u128, dst: &str) -> anyhow::Result<()> {
    let length = metainfo.pieces.len() / 20;
    let size = metainfo.piece_length * length;
    let client = Client::new(size, length);

    let metainfo = Arc::new(metainfo);

    let (piece_tx, _) = broadcast::channel::<usize>(length);
    let (data_tx, data_rx) = mpsc::channel::<(usize, Vec<u8>)>(length);
    let (peer_tx, mut peer_rx) = mpsc::channel::<String>(length);

    tokio::spawn(writer::run(
        Arc::clone(&metainfo),
        dst.to_owned(),
        data_rx,
        client.clone(),
    ));

    tokio::spawn(tracker::run(
        Arc::clone(&metainfo),
        port,
        peer_tx,
        client.clone(),
    ));

    let peer_permit = Arc::new(Semaphore::new(MAX_PEER_COUNT));

    while client.bit_fields.lock().await.left() != 0 {
        let permit = peer_permit.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let piece_tx = piece_tx.clone();
        let piece_rx = piece_tx.subscribe();
        let data_tx = data_tx.clone();

        let peer_addr = peer_rx
            .recv()
            .await
            .ok_or(anyhow!("Failed to get peer address"))?;

        tokio::spawn(peer::run(
            peer_addr, permit, client, piece_tx, piece_rx, data_tx,
        ));
    }

    for i in 0..length {
        piece_tx.send(i).unwrap();
    }

    Ok(())
}
