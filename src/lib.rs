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

use std::sync::RwLock;
use std::sync::LazyLock;
/// needs to be initialized 
mod bencode;
pub struct config {
    version: String,
    name: String,
}

pub static GLOBAL_CONFIG: LazyLock<RwLock<config>> = LazyLock::new(|| {
    RwLock::new(config {
        version: String::from("v0.1"),
        name: String::from("rustrent"),
    })
});

pub fn get_conf_info() -> String {
    let cfg = GLOBAL_CONFIG.read().unwrap();
    format!("{} {}", cfg.name, cfg.version)
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
    #[test]
    fn bencode_parse_test() {
        let test = fs::read("testdata/bencode.rs.torrent").expect("failed to open test file");
        parse_bencode(test);
    }
    #[test]
    fn bencode_render_test() {
        let test = fs::read("testdata/bencode.rs.torrent").expect("failed to open test file");
        match parse_bencode(test) {
            Ok(test_struct) => {
                println!("{} {:?}", test_struct.info.name, test_struct.created_by);
            }
            Err(e) => println!("{}", e),
        }
    }
}
