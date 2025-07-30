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

#[macro_use]
extern crate clap;
extern crate chrono;
extern crate number_prefix;
extern crate serde;
extern crate serde_bencode;
extern crate serde_bytes;
extern crate torrentinfo;
extern crate yansi;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::sync::LazyLock;

use chrono::prelude::*;
use clap::{App, AppSettings, Arg};
use number_prefix::{Prefixed, Standalone, binary_prefix};
use serde_bencode::value::Value;
use yansi::{Paint, Style};

use torrentinfo::Torrent;

const VERSION: &str = crate_version!();

static S_NUMBER: LazyLock<Style> = LazyLock::new(Style::cyan);
static S_BYTES: LazyLock<Style> = LazyLock::new(|| Style::red().bold());
static S_LABEL: LazyLock<Style> = LazyLock::new(|| Style::new().dimmed().bold());
static S_LABEL_ALT: LazyLock<Style> = LazyLock::new(Style::green);

fn main() {
    let app = App::new("torrentinfo")
        .version(VERSION)
        .about("A torrent file parser")
        .author("Daniel Müller <perlfuchsi@gmail.com>")
        .global_setting(AppSettings::ArgRequiredElseHelp)
        .global_setting(AppSettings::ColorAuto)
        .global_setting(AppSettings::DontCollapseArgsInUsage)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .arg(
            Arg::with_name("files")
                .short("f")
                .long("files")
                .help("Show files within the torrent")
                .required(false)
                .takes_value(false)
                .conflicts_with_all(&["details", "everything"]),
        )
        .arg(
            Arg::with_name("details")
                .short("d")
                .long("details")
                .help("Show detailed information about the torrent")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("everything")
                .short("e")
                .long("everything")
                .help("Print everything about the torrent")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("nocolour")
                .short("n")
                .long("nocolour")
                .help("No Colours")
                .required(false)
                .takes_value(false),
        )
        .arg(Arg::with_name("filename").required(true).takes_value(true));

    let matches = app.get_matches();

    let show_files = matches.is_present("files");
    let show_details = matches.is_present("details");
    let show_everything = matches.is_present("everything");
    let filename = matches.value_of("filename").unwrap();

    if matches.is_present("nocolour") {
        Paint::disable();
    }

    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Application Error: {e}");
            process::exit(1);
        }
    };

    let indent = "    ";
    let col_width: u32 = 19;
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).unwrap();

    println!(
        "{}",
        Paint::new(Path::new(filename).file_name().unwrap().to_str().unwrap()).bold()
    );

    if show_everything {
        print_everything(&buf, indent);
    } else {
        let torrent = Torrent::from_buf(&buf).unwrap();
        let info = torrent.info();

        if !show_details {
            if let Some(v) = info.name() {
                print_line("name", &v, indent, &col_width);
            }
            if let Some(v) = &torrent.comment() {
                print_line("comment", &v, indent, &col_width);
            }
            if let Some(v) = &torrent.announce() {
                print_line("announce url", &v, indent, &col_width);
            }
            if let Some(v) = &torrent.created_by() {
                print_line("created by", &v, indent, &col_width);
            }
            if let Some(v) = &torrent.creation_date() {
                let date = Utc.timestamp(*v, 0);
                print_line("created on", &date, indent, &col_width);
            }
            if let Some(v) = &torrent.encoding() {
                print_line("encoding", &v, indent, &col_width);
            }

            let files = torrent.num_files();
            print_line("num files", &files, indent, &col_width);
            let size = match binary_prefix(torrent.total_size() as f64) {
                Standalone(bytes) => format!("{bytes} bytes"),
                Prefixed(prefix, n) => format!("{n:.2} {prefix}B"),
            };
            print_line("total size", &S_NUMBER.paint(size), indent, &col_width);
            let info_hash_str = match torrent.info_hash() {
                Ok(info_hash) => torrentinfo::to_hex(&info_hash),
                Err(e) => format!("could not calculate info hash: {e}"),
            };

            print_line("info hash", &info_hash_str, indent, &col_width);
        }

        if show_files || show_details {
            println!("{}{}", indent, S_LABEL.paint("files"));
            let _files: Vec<torrentinfo::File>;
            let files = if let Some(f) = torrent.files() {
                f
            } else {
                let name = info.name().clone().unwrap();
                let f = torrentinfo::File::new(torrent.total_size(), vec![name]);
                _files = vec![f];
                &_files
            };

            for (index, file) in files.iter().enumerate() {
                println!("{}{}", indent.repeat(2), S_LABEL.paint(index));
                println!("{}{}", indent.repeat(3), file.path().join("/"));
                let size = match binary_prefix(*file.length() as f64) {
                    Standalone(bytes) => format!("{bytes} bytes"),
                    Prefixed(prefix, n) => format!("{n:.2} {prefix}B"),
                };
                println!("{}{}", indent.repeat(3), S_NUMBER.paint(size));
            }
        }

        if show_details {
            println!("{}{}", indent, S_LABEL.paint("piece length"));
            println!("{}{}", indent.repeat(2), &info.piece_length());
            println!("{}{}", indent, S_LABEL.paint("pieces"));
            println!(
                "{}{}",
                indent.repeat(2),
                S_BYTES.paint(format!("[{} Bytes]", info.pieces().len()))
            );
            println!("{}{}", indent, S_LABEL.paint("private"));
            println!("{}{}", indent.repeat(2), &info.private().unwrap_or_default());
        }
    }
}

fn print_line<T: std::fmt::Display>(name: &str, value: &T, indent: &str, col_width: &u32) {
    let n = *col_width as usize - name.len();
    println!("{}{} {}{}", indent, S_LABEL.paint(name), " ".repeat(n), value);
}

fn print_everything(buf: &[u8], indent: &str) {
    let bencoded = serde_bencode::from_bytes(buf).expect("could not decode .torrent file");
    if let Value::Dict(root) = bencoded {
        print_dict(&root, indent, 1);
    } else {
        println!("torrent file is not a dict");
    }
}

type Dict = HashMap<Vec<u8>, Value>;

fn print_dict(dict: &Dict, indent: &str, depth: usize) {
    let style = |key| {
        if depth % 2 == 0 {
            S_LABEL_ALT.paint(key)
        } else {
            S_LABEL.paint(key)
        }
    };
    for (k, v) in dict {
        let key = String::from_utf8_lossy(k);
        println!("{}{}", indent.repeat(depth), style(key));

        match v {
            Value::Dict(d) => print_dict(d, indent, depth + 1),
            Value::List(l) => print_list(l, indent, depth + 1),
            Value::Bytes(b) => {
                if b.len() > 80 {
                    println!(
                        "{}{}",
                        indent.repeat(depth + 1),
                        S_BYTES.paint(format!("[{} Bytes]", b.len()))
                    );
                } else {
                    println!("{}{}", indent.repeat(depth + 1), String::from_utf8_lossy(b));
                }
            }
            Value::Int(i) => println!("{}{}", indent.repeat(depth + 1), S_NUMBER.paint(i)),
        }
    }
}

fn print_list(list: &[Value], indent: &str, depth: usize) {
    let style = |key| {
        if depth % 2 == 0 {
            S_LABEL_ALT.paint(key)
        } else {
            S_LABEL.paint(key)
        }
    };
    for (k, v) in list.iter().enumerate() {
        println!("{}{}", indent.repeat(depth), style(k));
        match v {
            Value::Dict(d) => print_dict(d, indent, depth + 1),
            Value::List(l) => print_list(l, indent, depth + 1),
            Value::Bytes(b) => {
                if b.len() > 80 {
                    println!(
                        "{}{}",
                        indent.repeat(depth + 1),
                        S_BYTES.paint(format!("[{} Bytes]", b.len()))
                    );
                } else {
                    println!("{}{}", indent.repeat(depth + 1), String::from_utf8_lossy(b));
                }
            }
            Value::Int(i) => println!("{}{}", indent.repeat(depth + 1), S_NUMBER.paint(i)),
        }
    }
}
