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
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use anyhow::{Context, Error, Result, anyhow};
use chrono::prelude::*;
use clap::{Parser, arg};
use colored::Colorize;
use number_prefix::NumberPrefix;
use serde_bencode::value::Value;
use walkdir::WalkDir;

use torrentinfo::Torrent;

const COLUMN_WIDTH: u32 = 19;
const INDENT: &str = "    ";
const TORRENT_EXTENSION: &str = "torrent";

type Dict = HashMap<Vec<u8>, Value>;

#[derive(Parser)]
#[command(author, about, version)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// Optional input directory or file
    path: Option<String>,

    /// Show detailed information about the torrent
    #[arg(short, long)]
    details: bool,

    /// Print everything about the torrent
    #[arg(short, long)]
    everything: bool,

    /// Show files within the torrent
    #[arg(
        short,
        long,
        conflicts_with_all = ["everything"]
    )]
    files: bool,

    /// Disable colour output
    #[arg(short, long = "nocolour")]
    no_colour: bool,

    /// Recursive directory iteration
    #[arg(short, long)]
    recursive: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.no_colour {
        colored::control::set_override(false);
    }

    let input_path = resolve_input_path(args.path.as_deref())?;
    let (root, files) = resolve_input_files(&input_path, args.recursive, args.verbose)?;

    if files.is_empty() {
        anyhow::bail!("No torrent files found");
    }

    let num_files = files.len();
    let digits = digit_count(num_files);

    for (number, file) in files.into_iter().enumerate() {
        println!(
            "{}",
            format!(
                "{:>0width$}/{num_files}: {}",
                number + 1,
                get_relative_path_or_filename(&file, &root),
                width = digits
            )
            .bold()
        );
        if let Err(e) = torrent_info(file, &args) {
            eprintln!("{}", format!("Error: {e}").red());
        }
    }
    Ok(())
}

fn torrent_info(filepath: PathBuf, args: &Args) -> Result<(), Error> {
    let mut buf: Vec<u8> = vec![];
    File::open(filepath)?.read_to_end(&mut buf)?;

    if args.everything {
        print_everything(&buf, INDENT);
    } else {
        let torrent = Torrent::from_buf(&buf)?;
        let info = torrent.info();

        if let Some(v) = info.name() {
            print_line("name", &v);
        }
        if let Some(v) = &torrent.comment() {
            print_line("comment", &v);
        }
        if let Some(v) = &torrent.announce() {
            print_line("announce url", &v);
        }
        if let Some(v) = &torrent.created_by() {
            print_line("created by", &v);
        }
        if let Some(v) = &torrent.creation_date() {
            let date_str = Utc
                .timestamp_opt(*v, 0)
                .single()
                .map(|d| d.to_string())
                .unwrap_or_default();
            print_line("created on", &date_str);
        }
        if let Some(v) = &torrent.encoding() {
            print_line("encoding", &v);
        }

        let files = torrent.num_files();
        print_line("num files", &files);
        let size = match NumberPrefix::decimal(torrent.total_size() as f64) {
            NumberPrefix::Standalone(bytes) => format!("{bytes} bytes"),
            NumberPrefix::Prefixed(prefix, n) => format!("{n:.2} {prefix}B"),
        };
        print_line("total size", &size.cyan());
        let info_hash_str = match torrent.info_hash() {
            Ok(info_hash) => torrentinfo::to_hex(&info_hash),
            Err(e) => format!("could not calculate info hash: {e}"),
        };

        print_line("info hash", &info_hash_str);

        if args.details {
            let piece_length_str = format!("[{} Bytes]", info.pieces().len()).red().bold();
            print_line("piece length", &piece_length_str);

            let private_str = &info.private().unwrap_or_default().to_string();
            print_line("private", private_str);
        }

        if args.files {
            let mut files_list: Vec<torrentinfo::File> = Vec::new();
            let files = torrent.files().as_ref().map_or_else(
                || {
                    let name = info.name().to_owned().unwrap_or_else(String::new);
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

                let digits = digit_count(files.len());

                for (index, file) in files.iter().enumerate() {
                    let size = match NumberPrefix::decimal(*file.length() as f64) {
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
    }
    Ok(())
}

fn print_line<T: std::fmt::Display>(name: &str, value: &T) {
    let num_whitespace = COLUMN_WIDTH as usize - name.len();
    println!("{INDENT}{} {}{value}", name.bold(), " ".repeat(num_whitespace));
}

fn print_everything(buf: &[u8], indent: &str) {
    let bencoded = serde_bencode::from_bytes(buf).expect("could not decode .torrent file");
    if let Value::Dict(root) = bencoded {
        print_dict(&root, indent, 1);
    } else {
        println!("torrent file is not a dict");
    }
}

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

/// Return file root and list of files from the input path that can be either a directory or single file.
fn resolve_input_files(input: &PathBuf, recursive: bool, verbose: bool) -> Result<(PathBuf, Vec<PathBuf>)> {
    if input.is_file() {
        if verbose {
            println!("{}", format!("Reading file: {}", input.display()).bold().magenta());
        }
        if input.extension() == Some(TORRENT_EXTENSION.as_ref()) {
            let parent = input.parent().context("Failed to get parent directory")?.to_path_buf();
            Ok((parent, vec![input.clone()]))
        } else {
            Err(anyhow!("Input path is not an XML file: {}", input.display()))
        }
    } else {
        if verbose {
            println!(
                "{}",
                format!("Reading files from: {}", input.display()).bold().magenta()
            );
        }
        Ok((input.clone(), get_all_torrent_files(input, recursive)))
    }
}

/// Collect all torrent files from the given root path and sort by name.
fn get_all_torrent_files<P: AsRef<Path>>(root: P, recursive: bool) -> Vec<PathBuf> {
    let extension = OsStr::new(TORRENT_EXTENSION);
    let max_depth = if recursive { 999 } else { 1 };
    let mut files: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(std::result::Result::ok)
        .map(|e| e.path().to_owned())
        .filter(|path| path.is_file() && path.extension() == Some(extension))
        .collect();

    files.sort_unstable_by(|a, b| {
        let a_str = a.to_string_lossy().to_lowercase();
        let b_str = b.to_string_lossy().to_lowercase();
        a_str.cmp(&b_str)
    });
    files
}

/// Resolves the provided input path to a directory or file to an absolute path.
///
/// If `path` is `None` or an empty string, the current working directory is used.
/// The function verifies that the provided path exists and is accessible,
/// returning an error if it does not.
///
/// ```rust
/// use std::path::PathBuf;
/// use cli_tools::resolve_input_path;
///
/// let path = Some("src");
/// let absolute_path = resolve_input_path(path).unwrap();
/// ```
pub fn resolve_input_path(path: Option<&str>) -> Result<PathBuf> {
    let input_path = path.unwrap_or_default().trim().to_string();
    let filepath = if input_path.is_empty() {
        std::env::current_dir().context("Failed to get current working directory")?
    } else {
        PathBuf::from(input_path)
    };
    if !filepath.exists() {
        anyhow::bail!(
            "Input path does not exist or is not accessible: '{}'",
            filepath.display()
        );
    }

    // Dunce crate is used for nicer paths on Windows
    let absolute_input_path = dunce::canonicalize(&filepath)?;

    // Canonicalize fails for network drives on Windows :(
    if path_to_string(&absolute_input_path).starts_with(r"\\?") && !path_to_string(&filepath).starts_with(r"\\?") {
        Ok(filepath)
    } else {
        Ok(absolute_input_path)
    }
}

/// Gets the relative path or filename from a full path based on a root directory.
///
/// If the full path is within the root directory, the function returns the relative path.
/// Otherwise, it returns just the filename. If the filename cannot be determined, the
/// full path is returned.
///
/// ```rust
/// use std::path::Path;
/// use cli_tools::get_relative_path_or_filename;
///
/// let root = Path::new("/root/dir");
/// let full_path = root.join("subdir/file.txt");
/// let relative_path = get_relative_path_or_filename(&full_path, root);
/// assert_eq!(relative_path, "subdir/file.txt");
///
/// let outside_path = Path::new("/root/dir/another.txt");
/// let relative_or_filename = get_relative_path_or_filename(&outside_path, root);
/// assert_eq!(relative_or_filename, "another.txt");
/// ```
#[must_use]
pub fn get_relative_path_or_filename(full_path: &Path, root: &Path) -> String {
    if full_path == root {
        return full_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            .replace('\u{FFFD}', "");
    }
    full_path.strip_prefix(root).map_or_else(
        |_| {
            full_path.file_name().map_or_else(
                || full_path.display().to_string(),
                |name| name.to_string_lossy().to_string().replace('\u{FFFD}', ""),
            )
        },
        |relative_path| relative_path.display().to_string(),
    )
}

/// Convert a path to string with invalid Unicode handling
pub fn path_to_string(path: &Path) -> String {
    path.to_str().map_or_else(
        || path.to_string_lossy().to_string().replace('\u{FFFD}', ""),
        std::string::ToString::to_string,
    )
}

/// Check if entry is a hidden file or directory (starts with '.')
#[must_use]
pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().is_some_and(|s| s.starts_with('.'))
}

/// Count the number of digits in a number.
///
/// Used for getting the width required to print numbers.
///
/// Example input -> output return values:
/// ```not_rust
/// 0-9:     1
/// 10-99:   2
/// 100-999: 3
/// ```
#[must_use]
fn digit_count(number: usize) -> usize {
    if number < 10 {
        1
    } else {
        ((number as f64).log10() as usize) + 1
    }
}
