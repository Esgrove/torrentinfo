/*
 * torrentinfo, A torrent file parser
 * Copyright (C) 2018  Daniel Müller
 * Copyright (C) 2025  Akseli Lukkarila (modifications and new features)
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
        self.info
            .files
            .as_ref()
            .map_or_else(|| usize::from(self.info.length.is_some()), Vec::len)
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

    /// Path to the Ubuntu test torrent file
    const UBUNTU_TORRENT: &str = "tests/ubuntu-24.04.3-desktop-amd64.iso.torrent";
    /// Expected file size in bytes for the Ubuntu ISO
    const UBUNTU_SIZE: i64 = 6_345_887_744;
    /// Expected info hash for the Ubuntu torrent
    const UBUNTU_INFO_HASH: &str = "d160b8d8ea35a5b4e52837468fc8f03d55cef1f7";

    /// Path to the Pop!_OS test torrent file
    const POPOS_TORRENT: &str = "tests/pop-os_24.04_amd64_nvidia_22.iso.torrent";
    /// Expected file size in bytes for the Pop!_OS ISO
    const POPOS_SIZE: i64 = 3_600_056_320;
    /// Expected info hash for the Pop!_OS torrent
    const POPOS_INFO_HASH: &str = "d4d16dbb800d9560f92b3821c84800f7047c186b";

    #[test]
    fn test_to_hex_basic() {
        assert_eq!(to_hex(b"foobar"), "666f6f626172");
    }

    #[test]
    fn test_to_hex_binary() {
        assert_eq!(to_hex(&[0xff, 0x00, 0xaa]), "ff00aa");
    }

    #[test]
    fn test_to_hex_empty() {
        assert_eq!(to_hex(&[]), "");
    }

    #[test]
    fn test_to_hex_single_byte() {
        assert_eq!(to_hex(&[0x0f]), "0f");
        assert_eq!(to_hex(&[0xf0]), "f0");
        assert_eq!(to_hex(&[0x00]), "00");
        assert_eq!(to_hex(&[0xff]), "ff");
    }

    #[test]
    fn test_file_new() {
        let file = File::new(1024, vec!["path".to_string(), "to".to_string(), "file.txt".to_string()]);
        assert_eq!(file.length(), 1024);
        assert_eq!(file.path(), &["path", "to", "file.txt"]);
    }

    #[test]
    fn test_file_accessors() {
        let file = File {
            length: 2048,
            path: vec!["test.txt".to_string()],
            md5sum: Some("abc123".to_string()),
        };
        assert_eq!(file.length(), 2048);
        assert_eq!(file.path(), &["test.txt"]);
    }

    #[test]
    fn test_info_default() {
        let info = Info::default();
        assert!(info.name().is_none());
        assert!(info.files.is_none());
        assert!(info.length.is_none());
        assert!(info.private().is_none());
        assert_eq!(*info.piece_length(), 0);
        assert!(info.pieces().is_empty());
    }

    #[test]
    fn test_torrent_default() {
        let torrent = Torrent::default();
        assert!(torrent.name().is_none());
        assert!(torrent.comment().is_none());
        assert!(torrent.announce().is_none());
        assert!(torrent.announce_list().is_none());
        assert!(torrent.created_by().is_none());
        assert!(torrent.creation_date().is_none());
        assert!(torrent.encoding().is_none());
        assert!(torrent.files().is_none());
        assert_eq!(torrent.num_files(), 0);
        assert_eq!(torrent.total_size(), 0);
    }

    #[test]
    fn test_ubuntu_torrent_from_file() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert!(torrent.name().is_some());
    }

    #[test]
    fn test_ubuntu_torrent_name() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert_eq!(torrent.name().as_deref(), Some("ubuntu-24.04.3-desktop-amd64.iso"));
    }

    #[test]
    fn test_ubuntu_torrent_comment() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert_eq!(torrent.comment().as_deref(), Some("Ubuntu CD releases.ubuntu.com"));
    }

    #[test]
    fn test_ubuntu_torrent_announce() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert_eq!(
            torrent.announce().as_deref(),
            Some("https://torrent.ubuntu.com/announce")
        );
    }

    #[test]
    fn test_ubuntu_torrent_created_by() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert_eq!(torrent.created_by().as_deref(), Some("mktorrent 1.1"));
    }

    #[test]
    fn test_ubuntu_torrent_creation_date() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert!(torrent.creation_date().is_some());
        let creation_date = torrent.creation_date().unwrap();
        assert!(creation_date > 0);
    }

    #[test]
    fn test_ubuntu_torrent_num_files() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert_eq!(torrent.num_files(), 1);
    }

    #[test]
    fn test_ubuntu_torrent_total_size() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert_eq!(torrent.total_size(), UBUNTU_SIZE);
    }

    #[test]
    fn test_ubuntu_torrent_info_hash() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        let info_hash = torrent.info_hash().expect("Failed to calculate info hash");
        assert_eq!(info_hash.len(), 20, "SHA-1 hash should be 20 bytes");

        let hex_hash = to_hex(&info_hash);
        assert_eq!(hex_hash.len(), 40, "Hex hash should be 40 characters");
        assert_eq!(hex_hash, UBUNTU_INFO_HASH);
    }

    #[test]
    fn test_ubuntu_torrent_piece_length() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        let piece_length = *torrent.info().piece_length();
        assert_eq!(piece_length, 262_144, "Ubuntu torrent piece length should be 256 KB");
    }

    #[test]
    fn test_ubuntu_torrent_pieces_not_empty() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        assert!(!torrent.info().pieces().is_empty(), "Pieces should not be empty");
    }

    #[test]
    fn test_ubuntu_torrent_read_bytes() {
        let bytes = Torrent::read_bytes(Path::new(UBUNTU_TORRENT)).expect("Failed to read torrent bytes");
        assert!(!bytes.is_empty(), "Torrent file should not be empty");
    }

    #[test]
    fn test_ubuntu_torrent_from_buf() {
        let bytes = Torrent::read_bytes(Path::new(UBUNTU_TORRENT)).expect("Failed to read torrent bytes");
        let torrent = Torrent::from_buf(&bytes).expect("Failed to parse torrent from buffer");
        assert_eq!(torrent.name().as_deref(), Some("ubuntu-24.04.3-desktop-amd64.iso"));
    }

    // Pop!_OS torrent tests

    #[test]
    fn test_popos_torrent_from_file() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert!(torrent.name().is_some());
    }

    #[test]
    fn test_popos_torrent_name() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert_eq!(torrent.name().as_deref(), Some("pop-os_24.04_amd64_nvidia_22.iso"));
    }

    #[test]
    fn test_popos_torrent_comment() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert_eq!(
            torrent.comment().as_deref(),
            Some(
                "Unofficial Pop!_OS (Nvidia - 24.04 - revision 22) torrent created by FOSS Torrents. Published on https://fosstorrents.com/"
            )
        );
    }

    #[test]
    fn test_popos_torrent_announce() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert_eq!(
            torrent.announce().as_deref(),
            Some("udp://fosstorrents.com:6969/announce")
        );
    }

    #[test]
    fn test_popos_torrent_created_by() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert_eq!(
            torrent.created_by().as_deref(),
            Some("FOSS Torrents (https://fosstorrents.com/)")
        );
    }

    #[test]
    fn test_popos_torrent_creation_date() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert!(torrent.creation_date().is_some());
        let creation_date = torrent.creation_date().unwrap();
        assert!(creation_date > 0);
    }

    #[test]
    fn test_popos_torrent_num_files() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert_eq!(torrent.num_files(), 1);
    }

    #[test]
    fn test_popos_torrent_total_size() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert_eq!(torrent.total_size(), POPOS_SIZE);
    }

    #[test]
    fn test_popos_torrent_info_hash() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        let info_hash = torrent.info_hash().expect("Failed to calculate info hash");
        assert_eq!(info_hash.len(), 20, "SHA-1 hash should be 20 bytes");

        let hex_hash = to_hex(&info_hash);
        assert_eq!(hex_hash.len(), 40, "Hex hash should be 40 characters");
        assert_eq!(hex_hash, POPOS_INFO_HASH);
    }

    #[test]
    fn test_popos_torrent_piece_length() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        let piece_length = *torrent.info().piece_length();
        assert_eq!(piece_length, 1_048_576, "Pop!_OS torrent piece length should be 1 MB");
    }

    #[test]
    fn test_popos_torrent_pieces_not_empty() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        assert!(!torrent.info().pieces().is_empty(), "Pieces should not be empty");
    }

    #[test]
    fn test_popos_torrent_from_buf() {
        let bytes = Torrent::read_bytes(Path::new(POPOS_TORRENT)).expect("Failed to read torrent bytes");
        let torrent = Torrent::from_buf(&bytes).expect("Failed to parse torrent from buffer");
        assert_eq!(torrent.name().as_deref(), Some("pop-os_24.04_amd64_nvidia_22.iso"));
    }

    #[test]
    fn test_torrent_from_nonexistent_file() {
        let result = Torrent::from_file("nonexistent.torrent");
        assert!(result.is_err());
    }

    #[test]
    fn test_torrent_from_invalid_buf() {
        let invalid_data = b"not a valid torrent file";
        let result = Torrent::from_buf(invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_torrent_total_size_with_files() {
        let mut torrent = Torrent::default();
        torrent.info.files = Some(vec![
            File::new(1000, vec!["file1.txt".to_string()]),
            File::new(2000, vec!["file2.txt".to_string()]),
            File::new(3000, vec!["file3.txt".to_string()]),
        ]);
        assert_eq!(torrent.total_size(), 6000);
    }

    #[test]
    fn test_torrent_total_size_single_file() {
        let mut torrent = Torrent::default();
        torrent.info.length = Some(5000);
        assert_eq!(torrent.total_size(), 5000);
    }

    #[test]
    fn test_torrent_num_files_multiple() {
        let mut torrent = Torrent::default();
        torrent.info.files = Some(vec![
            File::new(100, vec!["a.txt".to_string()]),
            File::new(200, vec!["b.txt".to_string()]),
        ]);
        assert_eq!(torrent.num_files(), 2);
    }

    #[test]
    fn test_ubuntu_torrent_announce_list() {
        let torrent = Torrent::from_file(UBUNTU_TORRENT).expect("Failed to load Ubuntu torrent");
        // Ubuntu torrent has announce-list with backup trackers
        if let Some(announce_list) = torrent.announce_list() {
            assert!(!announce_list.is_empty(), "Announce list should not be empty");
        }
    }

    #[test]
    fn test_popos_torrent_announce_list() {
        let torrent = Torrent::from_file(POPOS_TORRENT).expect("Failed to load Pop!_OS torrent");
        // Pop!_OS torrent has announce-list with backup trackers
        if let Some(announce_list) = torrent.announce_list() {
            assert!(!announce_list.is_empty(), "Announce list should not be empty");
        }
    }
}
