use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use colored::Colorize;
use itertools::Itertools;
use number_prefix::NumberPrefix;
use serde_bencode::value::Value;

use torrentinfo::Torrent;

use crate::{Args, utils};

const BYTE_THRESHOLD: usize = 80;
const COLUMN_WIDTH: usize = 19;
const INDENT: &str = "    ";

type Dict = HashMap<Vec<u8>, Value>;

/// Process all torrent files and print their information
pub fn print_torrent_files(files: &[PathBuf], root: &Path, args: &Args) -> anyhow::Result<()> {
    if args.sort {
        print_torrents_sorted(files)
    } else {
        print_torrents(files, root, args);
        Ok(())
    }
}

fn print_torrents(files: &[PathBuf], root: &Path, args: &Args) {
    let num_files = files.len();
    let digits = utils::digit_count(num_files);

    for (number, file) in files.iter().enumerate() {
        print_file_header(number + 1, num_files, file, root, digits);
        if let Err(e) = print_single_torrent(file, args) {
            eprintln!("{}", format!("Error: {e}").red());
        }
    }
}

fn print_torrents_sorted(files: &[PathBuf]) -> anyhow::Result<()> {
    let mut total_size: u64 = 0;
    files
        .iter()
        .map(|file| {
            Torrent::from_file(file)
                .map(|torrent| (file, torrent))
                .map_err(anyhow::Error::from)
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .sorted_by(|(_, a), (_, b)| a.total_size().cmp(&b.total_size()))
        .for_each(|(file, torrent)| {
            total_size += torrent.total_size() as u64;
            let size = utils::format_file_size(torrent.total_size() as f64);
            let name: Cow<str> = torrent.name().as_deref().map_or_else(
                || {
                    file.file_stem()
                        .and_then(|s| s.to_str())
                        .map_or(Cow::Borrowed("unknown"), Cow::Borrowed)
                },
                Cow::Borrowed,
            );
            println!("{:>10}   {name}", size.cyan());
        });

    let total_str = utils::format_file_size(total_size as f64);
    println!("\n{}: {}", "Total size:".bold(), total_str.cyan());

    Ok(())
}

/// Print information for a single torrent file
fn print_single_torrent(filepath: &Path, args: &Args) -> anyhow::Result<()> {
    if args.everything {
        print_raw_data(filepath, INDENT)
    } else {
        print_torrent_info(filepath, args)
    }
}

/// Print information for a single torrent file
fn print_torrent_info(filepath: &Path, args: &Args) -> anyhow::Result<()> {
    let torrent = Torrent::from_file(filepath)?;

    print_info(&torrent);
    if args.details {
        print_extra_info(&torrent);
    }
    if args.files {
        print_files(&torrent);
    }

    Ok(())
}

/// Print basic torrent information
fn print_info(torrent: &Torrent) {
    if let Some(name) = torrent.name() {
        print_line("name", &name);
    }
    if let Some(comment) = &torrent.comment() {
        print_line("comment", &comment);
    }
    if let Some(announce_url) = &torrent.announce() {
        print_line("announce url", &announce_url);
    }
    if let Some(created_by) = &torrent.created_by() {
        print_line("created by", &created_by);
    }
    if let Some(creation_date) = torrent.creation_date() {
        let date_str = utils::format_creation_date(*creation_date);
        print_line("created on", &date_str);
    }
    if let Some(encoding) = &torrent.encoding() {
        print_line("encoding", &encoding);
    }

    let files = torrent.num_files();
    print_line("num files", &files);

    let size_str = utils::format_file_size(torrent.total_size() as f64);
    print_line("total size", &size_str.cyan());

    let info_hash_str = match torrent.info_hash() {
        Ok(info_hash) => torrentinfo::to_hex(&info_hash),
        Err(e) => format!("Could not calculate info hash: {e}"),
    };
    print_line("info hash", &info_hash_str);
}

/// Print detailed torrent information
fn print_extra_info(torrent: &Torrent) {
    let piece_length_str = format!("[{} Bytes]", torrent.info.pieces().len()).cyan().bold();
    print_line("piece length", &piece_length_str);

    if let Some(path) = &torrent.info.path {
        print_line("path", &format!("{path:#?}").cyan());
    }

    if let Some(private) = torrent.info.private() {
        print_line("private", &utils::colorize_bool(private > &0));
    }
}

/// Print a list of all the files in the torrent.
fn print_files(torrent: &Torrent) {
    let mut files_list: Vec<torrentinfo::File> = Vec::new();
    let files = torrent.files().as_ref().map_or_else(
        || {
            let name = torrent.name().to_owned().unwrap_or_default();
            let f = torrentinfo::File::new(torrent.total_size(), vec![name]);
            files_list = vec![f];
            &files_list
        },
        |f| f,
    );

    if files.len() == 1 {
        print_line("files", &files[0].path().join("/"));
    } else {
        println!("{INDENT}{}", "files".bold());

        let digits = utils::digit_count(files.len());

        for (index, file) in files.iter().enumerate() {
            let size = match NumberPrefix::decimal(file.length() as f64) {
                NumberPrefix::Standalone(bytes) => format!("{bytes} bytes"),
                NumberPrefix::Prefixed(prefix, n) => format!("{n:.2} {prefix}B"),
            };
            println!(
                "{}{:>0width$}{INDENT}{:>9}{INDENT}{}",
                INDENT.repeat(2),
                (index + 1).to_string().bold(),
                size.cyan(),
                file.path().join("/"),
                width = digits
            );
        }
    }
}

/// Print the file header with numbering
fn print_file_header(current: usize, total: usize, file: &Path, root: &Path, width: usize) {
    println!(
        "{}",
        format!(
            "{:>0width$}/{total}: {}",
            current,
            utils::get_relative_path_or_filename(file, root),
            width = width
        )
        .bold()
    );
}

/// Print a formatted line of data with indentation
fn print_line<T: std::fmt::Display>(name: &str, value: &T) {
    let num_whitespace = COLUMN_WIDTH.saturating_sub(name.len());
    println!("{INDENT}{} {}{value}", name.bold(), " ".repeat(num_whitespace));
}

/// Print all data in the torrent file without trying to parse it into a `Torrent`
fn print_raw_data(filepath: &Path, indent: &str) -> anyhow::Result<()> {
    let bytes = Torrent::read_bytes(filepath)?;
    let bencoded = serde_bencode::from_bytes(&bytes).context("could not decode .torrent file")?;
    if let Value::Dict(root) = bencoded {
        print_dict(&root, indent, 1);
    } else {
        println!("torrent file is not a dict");
    }
    Ok(())
}

/// Print a single bencode value
fn print_value(value: &Value, indent: &str, depth: usize) {
    match value {
        Value::Dict(d) => print_dict(d, indent, depth),
        Value::List(l) => print_list(l, indent, depth),
        Value::Bytes(b) => print_bytes(b, indent, depth),
        Value::Int(i) => println!("{}{}", indent.repeat(depth), i.to_string().cyan()),
    }
}

/// Print dictionary values recursively
fn print_dict(dict: &Dict, indent: &str, depth: usize) {
    for (key, value) in dict {
        let key = String::from_utf8_lossy(key);
        println!(
            "{}{}",
            indent.repeat(depth),
            if depth.is_multiple_of(2) {
                key.green()
            } else {
                key.bold()
            }
        );
        print_value(value, indent, depth + 1);
    }
}

/// Print list values recursively
fn print_list(list: &[Value], indent: &str, depth: usize) {
    for (key, value) in list.iter().enumerate() {
        println!(
            "{}{}",
            indent.repeat(depth),
            if depth.is_multiple_of(2) {
                key.to_string().green()
            } else {
                key.to_string().bold()
            }
        );
        print_value(value, indent, depth + 1);
    }
}

/// Print byte values with appropriate formatting
fn print_bytes(bytes: &[u8], indent: &str, depth: usize) {
    if bytes.len() > BYTE_THRESHOLD {
        println!(
            "{}{}",
            indent.repeat(depth),
            format!("[{} Bytes]", bytes.len()).cyan().bold()
        );
    } else {
        let content = std::str::from_utf8(bytes).unwrap_or("[invalid utf-8]");
        println!("{}{content}", indent.repeat(depth));
    }
}
