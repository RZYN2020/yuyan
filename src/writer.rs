//! Writer


use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

use crate::{bitmap::Bitmap, metainfo::MetaInfo};



pub async fn run(
    metainfo: Arc<MetaInfo>,
    dir: String,
    data_rx: mpsc::Receiver<(usize, Vec<u8>)>,
    bit_fields: Arc<Mutex<Bitmap>>,
) -> anyhow::Result<()> {
    unimplemented!()
}
