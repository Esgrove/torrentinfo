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

mod cli;
mod utils;

use anyhow::Result;
use clap::Parser;

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

    /// Sort files by size
    #[arg(short, long)]
    sort: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    cli::TorrentInfo::new(args)?.run()
}
