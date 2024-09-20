use crate::{
    bencode::{BDict, BItem},
    client::TClient,
    metainfo::{self, MetaInfo},
};
use anyhow::Ok;
use futures::{future::join_all, lock::Mutex, StreamExt};
use reqwest::Url;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    net::Ipv4Addr,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::mpsc::{self, Sender},
    time::sleep,
};
use tracing::{info, instrument, warn};

// encoded in a bencoded dictionay with keys...
// struct TrackerResponse {
//     failure_reason: Option<String>,
//     warning_message: Option<String>,
//     interval: usize,
//     min_interval: Option<usize>,
//     tracker_id: String,
//     complete: usize,
//     incomplete: usize,
//     // peers (dict of peers)
// }

enum Event {
    Started,
    Stopped,
    Completed,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Event::Started => write!(f, "started"),
            Event::Stopped => write!(f, "stopped"),
            Event::Completed => write!(f, "completed"),
        }
    }
}

async fn form_requst_params(
    metainfo: &MetaInfo,
    port: u128,
    client: &TClient,
    event: Event,
) -> String {
    let downloaded = client.bit_fields.lock().unwrap().len();
    let left = client.bit_fields.lock().unwrap().left();

    let uploaded = client.uploaded.lock().unwrap().to_string();
    let port = port.to_string();
    let downloaded = downloaded.to_string();
    let left = left.to_string();
    let event = event.to_string();

    let params = [
        ("info_hash", metainfo.info_hash.as_str()),
        ("peer_id", client.peer_id.as_str()),
        ("port", port.as_str()),
        ("uploaded", uploaded.as_str()),
        ("downloaded", downloaded.as_str()),
        ("left", left.as_str()),
        ("event", event.as_str()),
        ("compact", "1"),
    ];
    let query = params
        .iter()
        .map(|(param, value)| format!("{}={}", param, value))
        .collect::<Vec<String>>()
        .join("&");
    query
}

#[instrument]
async fn request_tracker(url: &str, params: &str, client: &TClient) -> anyhow::Result<Vec<Url>> {
    let mut url = Url::parse(url)?;
    url.set_query(Some(params));

    info!("Requesting tracker {}", url.to_string());
    let response = client.http_client.get(url).send().await?;
    let raw_rep = response.bytes().await?.to_vec();
    let rep_dict = BItem::deseri_cons(&raw_rep)?;
    let mut d: BDict = rep_dict.try_into()?;
    let peers_chunks = d.remove::<Vec<u8>>("peers")?;
    let results = peers_chunks
        .chunks_exact(6)
        .map(|chunk| {
            let ip_bytes: [u8; 4] = chunk[..4].try_into().unwrap();
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            let ip = Ipv4Addr::from(ip_bytes);
            let addr = format!("http://{}:{}", ip, port);
            Url::parse(&addr).unwrap()
        })
        .collect::<Vec<Url>>();
    info!("got {} results", results.len());
    Ok(results)
}

pub async fn run(
    metainfo: Arc<MetaInfo>,
    port: u128,
    peer_tx: mpsc::Sender<String>,
    client: TClient,
) -> anyhow::Result<()> {
    for tracker in metainfo.trackers() {
        let tracker = tracker.to_owned();
        let metainfo = Arc::clone(&metainfo);
        let peer_tx = peer_tx.clone();
        let client = client.clone();

        tokio::spawn(async move {
            let url = Url::parse(tracker.as_str()).unwrap();
            let mut interval = Duration::from_secs(1800);

            while client.bit_fields.lock().unwrap().left() != 0 {
                let params = form_requst_params(&*metainfo, port, &client, Event::Started).await;
                let mut url = url.clone();
                url.set_query(Some(params.as_str()));
                info!("Requesting tracker {:?}", url);
                let response = client.http_client.get(url).send().await;
                if let std::result::Result::Ok(response) = response {
                    let raw_rep = response.bytes().await.unwrap().to_vec();
                    let rep_dict = BItem::deseri_cons(&raw_rep).unwrap();
                    let mut d: BDict = rep_dict.try_into().unwrap();
                    let peers_chunks = d.remove::<Vec<u8>>("peers").unwrap();
                    let interval_ = d.remove::<i64>("interval").unwrap();
                    interval = Duration::from_secs(interval_ as u64);
                    let results = peers_chunks
                        .chunks_exact(6)
                        .map(|chunk| {
                            let ip_bytes: [u8; 4] = chunk[..4].try_into().unwrap();
                            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                            let ip = Ipv4Addr::from(ip_bytes);
                            format!("http://{}:{}", ip, port)
                        })
                        .collect::<Vec<String>>();
                    info!("got peers: {:?}", results);
                    for peer in results {
                        let _ = peer_tx.send(peer).await;
                    }
                } else {
                    warn!(
                        "Failed to request tracker {} for {}",
                        tracker,
                        params.as_str()
                    );
                }
                sleep(interval).await;
            }
            let params = form_requst_params(&*metainfo, port, &client, Event::Completed).await;
            let mut url = url.clone();
            url.set_query(Some(params.as_str()));
            let _ = client.http_client.get(url).send().await;
        });
    }
    Ok(())
}

pub async fn shutdown_tracker() {
    todo!()
}
