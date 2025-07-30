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

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

use chrono::prelude::*;
use clap::{App, AppSettings, Arg};
use colored::Colorize;
use number_prefix::{Prefixed, Standalone, binary_prefix};
use serde_bencode::value::Value;

use torrentinfo::Torrent;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
        colored::control::set_override(false);
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

    println!("{}", Path::new(filename).file_name().unwrap().to_str().unwrap().bold());

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
            print_line("total size", &size.cyan(), indent, &col_width);
            let info_hash_str = match torrent.info_hash() {
                Ok(info_hash) => torrentinfo::to_hex(&info_hash),
                Err(e) => format!("could not calculate info hash: {e}"),
            };

            print_line("info hash", &info_hash_str, indent, &col_width);
        }

        if show_files || show_details {
            println!("{}{}", indent, "files".bold());
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
                println!("{}{}", indent.repeat(2), index.to_string().bold());
                println!("{}{}", indent.repeat(3), file.path().join("/"));
                let size = match binary_prefix(*file.length() as f64) {
                    Standalone(bytes) => format!("{bytes} bytes"),
                    Prefixed(prefix, n) => format!("{n:.2} {prefix}B"),
                };
                println!("{}{}", indent.repeat(3), size.cyan());
            }
        }

        if show_details {
            println!("{}{}", indent, "piece length".bold());
            println!("{}{}", indent.repeat(2), &info.piece_length());
            println!("{}{}", indent, "pieces".bold());
            println!(
                "{}{}",
                indent.repeat(2),
                format!("[{} Bytes]", info.pieces().len()).red().bold()
            );
            println!("{}{}", indent, "private".bold());
            println!("{}{}", indent.repeat(2), &info.private().unwrap_or_default());
        }
    }
}

fn print_line<T: std::fmt::Display>(name: &str, value: &T, indent: &str, col_width: &u32) {
    let n = *col_width as usize - name.len();
    println!("{}{} {}{}", indent, name.bold(), " ".repeat(n), value);
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
    for (k, v) in dict {
        let key = String::from_utf8_lossy(k);
        println!(
            "{}{}",
            indent.repeat(depth),
            if depth % 2 == 0 { key.green() } else { key.bold() }
        );
        match v {
            Value::Dict(d) => print_dict(d, indent, depth + 1),
            Value::List(l) => print_list(l, indent, depth + 1),
            Value::Bytes(b) => {
                if b.len() > 80 {
                    println!(
                        "{}{}",
                        indent.repeat(depth + 1),
                        format!("[{} Bytes]", b.len()).red().bold()
                    );
                } else {
                    println!("{}{}", indent.repeat(depth + 1), String::from_utf8_lossy(b));
                }
            }
            Value::Int(i) => println!("{}{}", indent.repeat(depth + 1), i.to_string().cyan()),
        }
    }
}

fn print_list(list: &[Value], indent: &str, depth: usize) {
    for (key, value) in list.iter().enumerate() {
        println!(
            "{}{}",
            indent.repeat(depth),
            if depth % 2 == 0 {
                key.to_string().green()
            } else {
                key.to_string().bold()
            }
        );
        match value {
            Value::Dict(d) => print_dict(d, indent, depth + 1),
            Value::List(l) => print_list(l, indent, depth + 1),
            Value::Bytes(b) => {
                if b.len() > 80 {
                    println!(
                        "{}{}",
                        indent.repeat(depth + 1),
                        format!("[{} Bytes]", b.len()).red().bold()
                    );
                } else {
                    println!(
                        "{}{}",
                        indent.repeat(depth + 1),
                        std::str::from_utf8(b).unwrap_or("[invalid utf-8]")
                    );
                }
            }
            Value::Int(i) => println!("{}{}", indent.repeat(depth + 1), i.to_string().cyan()),
        }
    }
}
