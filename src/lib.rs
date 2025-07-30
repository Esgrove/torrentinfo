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

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_bencode;
extern crate serde_bytes;
extern crate sha1;
#[macro_use]
extern crate error_chain;

use serde_bencode::{de, ser};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

pub use crate::error::{Error, Result};

pub mod error;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Torrent {
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<String>>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(default)]
    encoding: Option<String>,
    info: Info,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
}

impl Torrent {
    pub fn from_buf(buf: &[u8]) -> Result<Self> {
        de::from_bytes(buf).map_err(std::convert::Into::into)
    }

    #[must_use]
    pub const fn files(&self) -> &Option<Vec<File>> {
        &self.info.files
    }

    #[must_use]
    pub const fn num_files(&self) -> usize {
        match self.files() {
            Some(f) => f.len(),
            None => 1,
        }
    }

    #[must_use]
    pub fn total_size(&self) -> i64 {
        if self.files().is_none() {
            return self.info.length.unwrap_or_default();
        }
        let mut total_size = 0;

        if let Some(files) = self.files() {
            for file in files {
                total_size += file.length;
            }
        }

        total_size
    }

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
    pub const fn comment(&self) -> &Option<String> {
        &self.comment
    }

    #[must_use]
    pub const fn announce(&self) -> &Option<String> {
        &self.announce
    }

    #[must_use]
    pub const fn announce_list(&self) -> &Option<Vec<String>> {
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
}

#[derive(Debug, Deserialize, Serialize)]
struct Node(String, i64);

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Info {
    #[serde(default)]
    files: Option<Vec<File>>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    md5sum: Option<String>,
    name: Option<String>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(rename = "piece length")]
    piece_length: i64,
    pieces: ByteBuf,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    root_hash: Option<String>,
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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct File {
    length: i64,
    path: Vec<String>,
    #[serde(default)]
    md5sum: Option<String>,
}

impl File {
    #[must_use]
    pub fn new(length: i64, path: Vec<String>) -> Self {
        Self {
            length,
            path,
            ..Default::default()
        }
    }

    #[must_use]
    pub const fn length(&self) -> &i64 {
        &self.length
    }

    #[must_use]
    pub fn path(&self) -> &[String] {
        &self.path
    }
}

const CHARS: &[u8] = b"0123456789abcdef";

#[must_use]
pub fn to_hex(bytes: &[u8]) -> String {
    let mut v = Vec::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        v.push(CHARS[(byte >> 4) as usize]);
        v.push(CHARS[(byte & 0xf) as usize]);
    }

    String::from_utf8(v).expect("Invalid UTF-8 sequence")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_to_hex() {
        assert_eq!(to_hex(b"foobar"), "666f6f626172");
    }
}
