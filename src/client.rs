//! yuyan-torrent Client

use crate::{bitmap::Bitmap, metainfo::MetaInfo, peer, tracker, writer};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex, Semaphore};

#[inline]
fn gen_id() -> String {
    format!("-YY00{:2}-", rand::thread_rng().gen::<u8>())
}

static MAX_PEER_COUNT: usize = 50;

pub async fn download(metainfo: MetaInfo, port: u128, dst: &str) -> anyhow::Result<()> {
    let peer_id = gen_id();

    let metainfo = Arc::new(metainfo);
    let http_client = reqwest::Client::new();

    let length = metainfo.pieces.len() / 20;
    let size = metainfo.piece_length * length;
    let bit_fields = Arc::new(Mutex::new(Bitmap::new(size)));

    let (piece_tx, _) = broadcast::channel::<usize>(length);
    let (data_tx, data_rx) = mpsc::channel::<(usize, Vec<u8>)>(length);

    tokio::spawn(writer::run(
        Arc::clone(&metainfo),
        dst.to_owned(),
        data_rx,
        Arc::clone(&bit_fields),
    ));

    let mut peers = Vec::new();
    let peer_permit = Arc::new(Semaphore::new(MAX_PEER_COUNT));

    for i in 0..length {
        piece_tx.send(i).unwrap();
    }

    while bit_fields.lock().await.size() != 0 {
        let permit = peer_permit.clone().acquire_owned().await.unwrap();
        let peer_id = peer_id.clone();
        let http_client = http_client.clone();
        let piece_tx = piece_tx.clone();
        let piece_rx = piece_tx.subscribe();
        let data_tx = data_tx.clone();

        let peer_addr = match peers.pop() {
            None => {
                peers.extend(tracker::get_peers(&metainfo, port, http_client.clone()).await?);
                peers.pop().unwrap()
            }
            Some(addr) => addr,
        };

        tokio::spawn(peer::run(
            peer_addr,
            permit,
            peer_id,
            http_client,
            piece_tx,
            piece_rx,
            data_tx,
        ));
    }
    Ok(())
}
