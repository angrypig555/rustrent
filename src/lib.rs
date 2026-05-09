/*
 lib.rs - rustrent
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

use std::fs;
use std::sync::RwLock;
use std::sync::LazyLock;
use std::path::{Path, PathBuf};

use crate::bencode::parse_bencode;
use crate::connection_handler::start_con_handler;
/// needs to be initialized 
mod bencode;
mod connection_handler;

/// Struct for the configuration
/// Used by GLOBAL_CONFIG
pub struct config {
    pub version: String,
    pub name: String,
    pub port: u16,
}
/// Global configuration
/// 
/// LazyLock + RwLock global struct.
/// Filled with placeholder values at start.
/// It is reccomended to save / read the configuration fromn the user.
pub static GLOBAL_CONFIG: LazyLock<RwLock<config>> = LazyLock::new(|| {
    RwLock::new(config {
        version: String::from("v0.1"),
        name: String::from("rustrent"),
        port: 6881,
    })
});


/// Get the current configuration as a formatted string
/// 
/// Returns: name version listen_port
pub fn get_conf_info() -> String {
    let cfg = GLOBAL_CONFIG.read().unwrap();
    format!("{} {} {}", cfg.name, cfg.version, cfg.port)
}

/// Starts the session, finds / downloads / seeds a file
/// 
/// Takes the path to the torrent file as an argument
pub fn start_session(file_path: PathBuf) {
    let file = fs::read(file_path).expect("failed to open torrent file");
        match parse_bencode(file) {
            Ok(file_i_have) => {
                start_con_handler(file_i_have);
            }
            Err(e) => println!("{}", e),
        }
}
#[cfg(test)]
mod tests {
    use crate::bencode::parse_bencode;
    use std::io;
    use std::io::Read;
    use std::fs;
use super::*;

    #[test]
    fn infotest() {
        println!("{}", get_conf_info());
    }
    
}
