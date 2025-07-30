/*
 * torrentinfo, A torrent file parser
 * Copyright (C) 2018  Daniel Müller
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>
 */

pub mod errors;

use std::fs::File as StdFile;
use std::io::Read;
use std::path::Path;

use serde_bencode::ser;
use serde_bencode::value::Value;
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::errors::Result;

const HEX_CHARS: &[u8] = b"0123456789abcdef";
const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Torrent {
    #[serde(default)]
    pub announce: Option<String>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(rename = "comment")]
    pub comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
    #[serde(default)]
    pub encoding: Option<String>,
    pub info: Info,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    pub httpseeds: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Info {
    #[serde(default)]
    pub files: Option<Vec<File>>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub md5sum: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub pieces: ByteBuf,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct File {
    pub length: i64,
    pub path: Vec<String>,
    #[serde(default)]
    pub md5sum: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Node(String, i64);

impl Torrent {
    /// Create `Torrent` from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let buf = Self::read_bytes(path.as_ref())?;
        Self::from_buf(&buf)
    }

    /// Create `Torrent` from bytes
    pub fn from_buf(buf: &[u8]) -> Result<Self> {
        serde_bencode::from_bytes(buf).map_err(|e| {
            if let Ok(Value::Dict(dict)) = serde_bencode::from_bytes::<Value>(buf) {
                eprintln!("Bencode decode error. Torrent structure:");
                Self::debug_torrent_structure(&dict);
            }
            e.into()
        })
    }

    /// Read torrent file bytes without converting to a `Torrent`
    pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
        let file = StdFile::open(path)?;
        let file_size = file.metadata().map(|m| m.len() as usize).unwrap_or(DEFAULT_BUFFER_SIZE);

        let mut buf = Vec::with_capacity(file_size);
        let mut reader = std::io::BufReader::new(file);
        reader.read_to_end(&mut buf)?;
        Ok(buf)
    }

    #[must_use]
    pub const fn files(&self) -> &Option<Vec<File>> {
        &self.info.files
    }

    #[must_use]
    pub fn num_files(&self) -> usize {
        self.info.files.as_ref().map_or(1, Vec::len)
    }

    /// Get total size of all files in the torrent
    #[must_use]
    pub fn total_size(&self) -> i64 {
        self.info.files.as_ref().map_or_else(
            || self.info.length.unwrap_or(0),
            |files| files.iter().map(|file| file.length).sum(),
        )
    }

    /// Calculate SHA-1 info hash
    pub fn info_hash(&self) -> Result<Vec<u8>> {
        let info = ser::to_bytes(&self.info)?;
        let info_hash: Vec<u8> = Sha1::digest(&info).to_vec();
        Ok(info_hash)
    }

    #[must_use]
    pub const fn info(&self) -> &Info {
        &self.info
    }

    #[must_use]
    pub const fn name(&self) -> &Option<String> {
        &self.info.name
    }

    #[must_use]
    pub const fn comment(&self) -> &Option<String> {
        &self.comment
    }

    #[must_use]
    pub const fn announce(&self) -> &Option<String> {
        &self.announce
    }

    #[must_use]
    pub const fn announce_list(&self) -> &Option<Vec<Vec<String>>> {
        &self.announce_list
    }

    #[must_use]
    pub const fn created_by(&self) -> &Option<String> {
        &self.created_by
    }

    #[must_use]
    pub const fn creation_date(&self) -> &Option<i64> {
        &self.creation_date
    }

    #[must_use]
    pub const fn encoding(&self) -> &Option<String> {
        &self.encoding
    }

    /// Debug helper to print torrent structure
    fn debug_torrent_structure(dict: &std::collections::HashMap<Vec<u8>, Value>) {
        for (key, value) in dict {
            let key_str = String::from_utf8_lossy(key);
            match value {
                Value::List(list) => {
                    eprintln!("  {}: List with {} elements", key_str, list.len());
                    if key_str == "announce-list" {
                        eprintln!("    announce-list structure issue detected");
                    }
                }
                Value::Bytes(bytes) => {
                    eprintln!("  {key_str}: Bytes ({} bytes)", bytes.len());
                }
                Value::Int(i) => {
                    eprintln!("  {key_str}: Integer ({i})");
                }
                Value::Dict(_) => {
                    eprintln!("  {key_str}: Dictionary");
                }
            }
        }
    }
}

impl Info {
    #[must_use]
    pub const fn name(&self) -> &Option<String> {
        &self.name
    }

    #[must_use]
    pub const fn piece_length(&self) -> &i64 {
        &self.piece_length
    }

    #[must_use]
    pub const fn pieces(&self) -> &ByteBuf {
        &self.pieces
    }

    #[must_use]
    pub const fn private(&self) -> &Option<u8> {
        &self.private
    }
}

impl File {
    #[must_use]
    pub const fn new(length: i64, path: Vec<String>) -> Self {
        Self {
            length,
            path,
            md5sum: None,
        }
    }

    #[must_use]
    pub const fn length(&self) -> i64 {
        self.length
    }

    #[must_use]
    pub fn path(&self) -> &[String] {
        &self.path
    }
}

/// Convert bytes to hexadecimal string representation
#[must_use]
pub fn to_hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(HEX_CHARS[(byte >> 4) as usize] as char);
        result.push(HEX_CHARS[(byte & 0xf) as usize] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_to_hex() {
        assert_eq!(to_hex(b"foobar"), "666f6f626172");
        assert_eq!(to_hex(&[0xff, 0x00, 0xaa]), "ff00aa");
        assert_eq!(to_hex(&[]), "");
    }
}
