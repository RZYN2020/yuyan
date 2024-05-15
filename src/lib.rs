//! A simple implementation of Bittorrent client.
//! 
//! 
//! According to the BitTorrent [Specification]((https://wiki.theory.org/BitTorrentSpecification)), the BitTorrent protocol facilitates peer-to-peer file downloading.
//! 
//! In the protocol, each peer acts as both a sender and receiver, engaging in communication with other peers to 
//! download desired pieces while also providing the pieces other peers seek. Peers discover each other through a 
//! tracker server and communicate using bencode-encoded messages.
//! 
//! # Design
//! 
//! The client is designed as a **multi-threaded** program to handle multiple incoming peers and concurrently download multiple torrents. 
//! Each torrent file is associated with several threads:
//! 
//! - A **manager thread (M)** to initialize and manage the other threads.
//! - A **client thread (C)** responsible for sending pieces to peers.
//! - A **tracker thread (T)** periodically obtaining peers from the tracker server and informing peers about our presence.
//! - A **writer thread (W)** for writing data to files.
//! - For each peer, a **peer thread (P)** to request pieces from peers.
//! 
//! 
//! ## Lifetime of the threads
//! 
//! - Stage 1: **Initialization**: M creates C, W, and T threads, while M holds the essential data structures of the torrent. T requests peers from the tracker.
//! - Stage 2: **Downloading**: Pn threads are created to request and receive pieces, which are then sent to W for writing to the file. While the number of peers is insufficient, T requests more peers.
//! - Stage 3: **Serving**: Once all the pieces are downloaded, W and Pn threads are shut down, leaving T to inform other peers, C to upload pieces, and M to hold data.
//! - Stage 4: **Shutdown**: M shuts down T and C, clears up data, and exits.
//! 
//! 
//! ## Information passing between threads
//! 
//! - **Locked global data structures (GDS)** hold global structures like the torrent bitmap and peers' IP list.
//! - Channels:
//!   - **Shutdown channel (Shut)**
//!   - **Manager-Client channel (MC)**: M can shut down C through this channel.
//!   - **Manager-Tracker channel (MT)**: M can shut down T or request more peers.
//!   - **Tracker-peers-Manager channel (TpM)**: T sends peer IPs to M through this channel.
//!   - **Peers-Writer channel (PdW)**: Peers send downloaded pieces to W. (d represent data)
//!   - **Peers channel (PiP)**: Peers request the index of the pieces to download.
//!
//!
//!
//! Thread roles:
//! - Main thread: Sender of Shut.
//! - M: Holds GDS, receiver of Shut and TpM, sender of MC, PiP.
//! - C: Accesses GDS, receiver of MC, PC.
//! - T: Accesses GDS, receiver of MT, sender of TpM.
//! - W: Accesses GDS, receiver of PdW.
//! - Pn: Accesses GDS, sender of PdW PiP. receiver of PiP.
//! 
//! 
//! ## Questions
//! 
//! 1. Why use multiple threads instead of a single thread?
//! > I believe the tasks naturally divide into these threads, which makes the code cleaner. Additionally, downloading multiple pieces concurrently allows for faster BitTorrent performance.
//! 
//! 2. Why not write files concurrently? Is file writing, rather than BitTorrent downloading, the bottleneck?
//! > Perhaps. We can consider adding concurrent file writing later if it becomes a bottleneck.
//! 
//! 3. Why do we have PW, PCh, SC, and other channels alongside GDS? Why not synchronize all data with channels? Why not synchronize all data with mutex?
//! > According to the post provided here, channels are somewhat more specialized than mutex and can be faster. However, not all synchronization can be done with channels, so different synchronization mechanisms may be used for different purposes.
//! 
//! # Example
//! 
//! ```
//! // file: torrent path
//! // dst: result path
//! let meta_info = MetaInfo::from(&file)?;
//! yuyan_torrent::download_once(meta_info, 10006, dst).await?
//! ```

pub mod bencode;
pub mod metainfo;
pub mod peer;
pub mod client;
pub mod bitmap;
pub mod tracker;
pub mod writer;

