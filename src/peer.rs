use reqwest::Client;
use tokio::sync::{
    broadcast, mpsc, OwnedSemaphorePermit, Semaphore
};

#[derive(Debug)]
struct Peer {
    pub am_choking: bool,      // this client is choking the peer
    pub am_interested: bool,   // this client is interested in the peer
    pub peer_choking: bool,    // peer is choking this client
    pub peer_interested: bool, // peer is interested in this client
}

impl Peer {
    pub fn new() -> Self {
        Peer {
            am_choking: false,
            am_interested: false,
            peer_choking: false,
            peer_interested: false,
        }
    }
}


pub async fn run(
    addr: String,
    permit: OwnedSemaphorePermit,
    peer_id: String,
    http_client: Client,
    piece_tx: broadcast::Sender<usize>,
    piece_rx: broadcast::Receiver<usize>,
    data_tx: mpsc::Sender<(usize, Vec<u8>)>,
) {
    let peer = Peer::new();
    unimplemented!()
}