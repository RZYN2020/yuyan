//! yuyan-torrent Client

use crate::{bitmap::Bitmap, metainfo::MetaInfo, peer, tracker, writer};
use anyhow::{anyhow, bail};
use rand::Rng;
// https://stackoverflow.com/questions/73840520/what-is-the-difference-between-stdsyncmutex-vs-tokiosyncmutex
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::{broadcast, mpsc, Semaphore};

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

// torrent client
#[derive(Clone, Debug)]
pub struct TClient {
    pub peer_id: String,
    pub length: usize, // piece num
    pub bit_fields: Arc<Mutex<Bitmap>>,
    pub uploaded: Arc<Mutex<usize>>,
    pub http_client: reqwest::Client,
    pub dest: String,
    pub metainfo: Arc<MetaInfo>,
}

impl TClient {
    pub fn new(metainfo: MetaInfo, port: u128, dst: &str) -> Self {
        let client = TClient {
            peer_id: gen_id(),
            length: metainfo.pieces.len() / 20,
            bit_fields: Arc::new(Mutex::new(Bitmap::new(
                metainfo.piece_length * metainfo.pieces.len() / 20,
            ))),
            uploaded: Arc::new(Mutex::new(0)),
            http_client: reqwest::Client::new(),
            dest: dst.to_owned(),
            metainfo: Arc::new(metainfo),
        };

        let client_ = client.clone();
        let dst = dst.to_owned();

        let _ = thread::spawn(move || {
            let client = client_;
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let (piece_tx, _) = broadcast::channel::<usize>(client.length);
                let (data_tx, data_rx) = mpsc::channel::<(usize, Vec<u8>)>(client.length);
                let (peer_tx, mut peer_rx) = mpsc::channel::<String>(client.length);

                tokio::spawn(writer::run(
                    client.metainfo.clone(),
                    dst,
                    data_rx,
                    client.clone(),
                ));

                tokio::spawn(tracker::run(
                    client.metainfo.clone(),
                    port,
                    peer_tx,
                    client.clone(),
                ));

                let client_ = client.clone();
                let peer_permit = Arc::new(Semaphore::new(MAX_PEER_COUNT));

                while client_.bit_fields.lock().unwrap().left() != 0 {
                    let permit = peer_permit.clone().acquire_owned().await.unwrap();
                    let client = client_.clone();
                    let piece_tx = piece_tx.clone();
                    let piece_rx = piece_tx.subscribe();
                    let data_tx = data_tx.clone();

                    let peer_addr = peer_rx
                        .recv()
                        .await
                        .ok_or(anyhow!("Failed to get peer address"))
                        .unwrap();

                    tokio::spawn(peer::run(
                        peer_addr, permit, client, piece_tx, piece_rx, data_tx,
                    ));
                }
            });
        });

        client
    }
}
