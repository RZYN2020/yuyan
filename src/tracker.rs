use crate::metainfo::MetaInfo;
use futures::future::join_all;
use reqwest::{Client, Url};

// #[derive(Debug)]
// pub struct Tracker {
//     url: String,
//     uploaded: usize,   // in bytes
//     downloaded: usize, // in bytes
// }

// impl Drop for Tracker {
//     fn drop(&mut self) {
//         // todo: graceful shutdown (send shutdown or completed to tracker)
//         unimplemented!()
//     }
// }

// impl Tracker {
//     pub fn new(url: &str) -> Tracker {
//         Tracker {
//             url: url.to_owned(),
//             uploaded: 0,
//             downloaded: 0,
//         }
//     }

//     pub async fn update(&mut self) {
//         // request

//     //     let params = [
//     //         ("info_hash", self.info.info_hash.clone()),
//     //         ("peer_id", self.peer_id.to_owned()),
//     //         ("port", self.port.to_string()),
//     //         ("uploaded", self.uploaded.to_string()),
//     //         ("downloaded", self.downloaded.to_string()),
//     //         ("left", (self.total - self.downloaded).to_string()),
//     //         ("event", "started".to_owned()),
//     //         ("compact", "1".to_owned()),
//     //     ];
//     //     let query = params
//     //         .iter()
//     //         .map(|(param, value)| format!("{}={}", param, value))
//     //         .collect::<Vec<String>>()
//     //         .join("&");

//     //     let client = Client::new();
//     //     let mut url = Url::parse(&self.url).unwrap();
//     //     // use set_query to avoid escape twice
//     //     url.set_query(Some(&query));
//     //     println!("req url: {:?}", url);
//     //     let res = client
//     //         .get(url)
//     //         .timeout(Duration::from_secs(100000000000000))
//     //         .send()
//     //         .await
//     //         .unwrap();
//     //     println!("res: {:?}", res);
//     // }
// }

// encoded in a bencoded dictionay with keys...
struct TrackerResponse {
    failure_reason: Option<String>,
    warning_message: Option<String>,
    interval: usize,
    min_interval: Option<usize>,
    tracker_id: String,
    complete: usize,
    incomplete: usize,
    // peers (dict of peers)
}

enum Event {
    Started,
    Stopped,
    Completed,
}

struct TrackerRequest {
    info_hash: String,
    peer_id: String,
    port: u128,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    event: Event,
    compact: bool,
}

impl TrackerRequest {
    fn new(metainfo: &MetaInfo, port: u128, event: Event) -> Self {
        unimplemented!()
    }

    fn form_url(&self, base_url: &str) -> Url {
        unimplemented!()
    }
}

async fn request_tracker(
    url: &str,
    metainfo: &MetaInfo,
    req: &TrackerRequest,
    client: &Client,
) -> anyhow::Result<Vec<String>> {
    let url = req.form_url(url);
    let response = client.get(url).send().await.unwrap();
    unimplemented!()
}

pub async fn get_peers(
    metainfo: &MetaInfo,
    port: u128,
    client: Client,
) -> anyhow::Result<Vec<String>> {
    let req = TrackerRequest::new(metainfo, port, Event::Started);

    let mut futures = Vec::new();

    for tracker in metainfo.trackers() {
        let future = request_tracker(tracker, metainfo, &req, &client);
        futures.push(future);
    }

    let results = join_all(futures).await;
    
    Ok(results
        .into_iter()
        .filter_map(Result::ok)
        .flatten()
        .collect())
}
