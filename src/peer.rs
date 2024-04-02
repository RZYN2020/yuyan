use reqwest::Url;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::{
    broadcast, mpsc, OwnedSemaphorePermit, Semaphore
}};
use tracing::info;

use crate::{bitmap::Bitmap, client::Client};

#[derive(Debug)]
struct Peer {
    pub am_choking: bool,      // this client is choking the peer
    pub am_interested: bool,   // this client is interested in the peer
    pub peer_choking: bool,    // peer is choking this client
    pub peer_interested: bool, // peer is interested in this client
    bitfields: Bitmap,
    stream: TcpStream,
}

impl Peer {
    // pub fn new(addr: String) -> Self {
    //     let mut stream = TcpStream::connect(addr).await.unwrap();
    //     Peer {
    //         am_choking: true,
    //         am_interested: false,
    //         peer_choking: true,
    //         peer_interested: false,
    //         stream,
    //     }
    // }

    async fn send(msg: Message) -> anyhow::Result<()> {
        unimplemented!()
    }
    async fn sends(msgs: Vec<Message>) -> anyhow::Result<()> {
        unimplemented!()
    }

    async fn recv() -> anyhow::Result<Vec<Message>> {
        unimplemented!()
    }
}

struct Handshake {

}


enum Message {
    // handshake: <pstrlen><pstr><reserved><info_hash><peer_id>
    HandShake,
    // normal messages: <length prefix><message ID><payload>

}


pub async fn run(
    addr: String,
    permit: OwnedSemaphorePermit,
    client: Client,
    piece_tx: broadcast::Sender<usize>,
    piece_rx: broadcast::Receiver<usize>,
    data_tx: mpsc::Sender<(usize, Vec<u8>)>,
) {
    info!("running peer: {}", addr.to_string());

    // let peer = Peer::new();

    // todo: drop permit when existing

    let handshake = Message::HandShake;

    // peer.send(handshake).await;

    // for msg in peer.recv().await {
        // build bitmap

        // match Bitfield/Have/Have All ...
    // }


    // send Interesting

    // loop:
    //    1. retrive piece id
    //    2. check if have have 
    //      no  -> put it back
    //      yes -> request
    //    3. receive piece
    //    4. put it into data channel
}