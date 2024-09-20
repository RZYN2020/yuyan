// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lazy_static::lazy_static;
use libyuyan::{client::TClient, metainfo};
use serde::Serialize;
use std::sync::Mutex;

lazy_static! {
    static ref CLIENTS: Mutex<Vec<TClient>> = Mutex::new(Vec::new());
}

#[derive(Serialize)]
struct TClientState {
    name: String,
    dst: String,
    piece_sz: u64,
    length: u64,
    left: u64,
}


#[tauri::command]
fn get_client_states() -> Vec<TClientState> {
    let clients = CLIENTS.lock().unwrap();
    clients
     .iter()
     .map(|client| TClientState {
            name: client.metainfo.name.clone(),
            dst: client.dest.clone(),
            piece_sz: client.metainfo.piece_length as u64,
            length: client.metainfo.pieces.len() as u64,
            left: client.bit_fields.lock().unwrap().left() as u64,
        }).collect()
}


#[tauri::command]
fn add_torrent(torrent_path: &str, dst: &str) -> Result<(), String> {
    use libyuyan::metainfo::MetaInfo;
    let metainfo = MetaInfo::from(torrent_path).map_err(|e| e.to_string())?;
    let client = TClient::new(metainfo, 10006, dst);
    let mut clients = CLIENTS.lock().unwrap();
    clients.push(client);
    Ok(())
}


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_client_states, add_torrent])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
