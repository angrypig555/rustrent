/*
 connection_handler.rs - rustrent
 Copyright [2026] [angrypig555]

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use std::time::Duration;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::GLOBAL_CONFIG;
use crate::bencode::Torrent;
use reqwest::Url;
use sha1::{Sha1, Digest};
use rand::Rng;
use urlencoding;
use rand::rng;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerResponse {
    pub interval: u64,
    pub peers: serde_bytes::ByteBuf,
    #[serde(default)]
    pub complete: Option<i64>,
    #[serde(default)]
    pub incomplete: Option<i64>,
}

async fn announce(file_i_have: &Torrent, port: u16) -> Result<(TrackerResponse), Box<dyn std::error::Error + Send + Sync>>{
    let mut hasher = Sha1::new();
    let info_bytes = serde_bencode::to_bytes(&file_i_have.info)?;
    hasher.update(&info_bytes);
    let info_hash = hasher.finalize();
    let peerid = {
        let mut rng = rand::rng(); // or rand::thread_rng()
        let rng_id: u64 = rng.random_range(0..999999999999);
        format!("-RR0001-{}", rng_id)
    };
    let tracker_url_str = file_i_have.announce.as_deref().ok_or("No tracker URL")?;
    let mut url = Url::parse(tracker_url_str)?;
    {
        //let mut query = url.query_pairs_mut();
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        let encoded_hash = urlencoding::encode_binary(&info_hash);
        query.append_pair("info_hash", &encoded_hash);
        query.append_pair("peer_id", &peerid);
        query.append_pair("port", &port.to_string());
        query.append_pair("uploaded", "0");
        query.append_pair("downloaded", "0");
        let length = file_i_have.info.length.unwrap_or(0).to_string();
        query.append_pair("left", &length);
        query.append_pair("compact", "1");
        query.append_pair("event", "started");
        let full_query = format!("info_hash={}&{}", encoded_hash, query.finish());
        url.set_query(Some(&full_query));
    }
    let response = reqwest::get(url.as_str()).await?;
    let body = response.bytes().await?;
    let tracker_res: TrackerResponse = serde_bencode::from_bytes(&body)?;
    println!("{:?}", body);
    Ok(tracker_res)
}

#[tokio::main]
pub async fn start_con_handler(file_i_have: Torrent) {
    let file_i_have = Arc::new(file_i_have);
    let cfg = GLOBAL_CONFIG.read().unwrap();
    let address = if cfg!(debug_assertions) {"127.0.0.1"} else {"0.0.0.0"};
    let port = cfg.port;
    let listener = TcpListener::bind(format!("{}:{}", address, port)).await.expect("error happened, failed to open tcplistener");
    let tracker_file_ref = Arc::clone(&file_i_have);
    tokio::spawn(async move {
        loop {
            match announce(&tracker_file_ref, port).await {
                Ok(response) => {
                    let sleep_dur = response.interval;
                    tokio::time::sleep(Duration::from_secs(sleep_dur)).await;
                }
                Err(e) => {
                    eprintln!("Tracker error: {}", e);
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            }
        }
    });

    loop {
        let (mut socket, _) = listener.accept().await.expect("failed to accept connection");

        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::{bencode::parse_bencode, connection_handler::start_con_handler};
    #[test]
    fn full_stack_test() {
        let test = fs::read("testdata/bencode.rs.torrent").expect("failed to open test file");
        match parse_bencode(test) {
            Ok(test_struct) => {
                println!("{} {:?}", test_struct.info.name, test_struct.created_by);
                start_con_handler(test_struct);
            }
            Err(e) => println!("{}", e),
        }
        
    }
}