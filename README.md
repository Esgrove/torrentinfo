# torrentinfo

A torrent file parser library and CLI utility.

## Usage

By default,
`torrentinfo` will print the information for all `.torrent` files directly in the working directory.
An optional path to a directory or torrent file can be specified,
with a recursive option to recursively find all torrent files.

```shell
A torrent file parser

Usage: torrentinfo [OPTIONS] [PATH]

Arguments:
  [PATH]  Optional input directory or file

Options:
  -d, --details             Show detailed information about the torrent
  -e, --everything          Print everything about the torrent
  -f, --files               Show files within the torrent
  -n, --nocolour            Disable colour output
  -r, --recursive           Recursive directory iteration
  -s, --sort                Sort files by size
  -l, --completion <SHELL>  Generate shell completion [possible values: bash, elvish, fish, powershell, zsh]
  -v, --verbose             Verbose output
  -h, --help                Print help
  -V, --version             Print version
```

### Examples

Display information for a single torrent file:

```shell
$ torrentinfo ubuntu-24.04.3-desktop-amd64.iso.torrent
ubuntu-24.04.3-desktop-amd64.iso.torrent
    name                ubuntu-24.04.3-desktop-amd64.iso
    comment             Ubuntu CD releases.ubuntu.com
    announce url        https://torrent.ubuntu.com/announce
    created by          mktorrent 1.1
    created on          2025-08-07 10:28:19 UTC
    num files           1
    total size          6.35 GB
    info hash           d160b8d8ea35a5b4e52837468fc8f03d55cef1f7
```

Sort torrents in a directory by size in increasing order:

```shell
$ torrentinfo -s ~/Downloads/
   3.60 GB   pop-os_24.04_amd64_nvidia_22.iso
   6.35 GB   ubuntu-24.04.3-desktop-amd64.iso

Total size: 9.95 GB
```

## Installation

With script:

```shell
./install.sh
```

## Library Usage

The library can be used to parse torrent files programmatically:

```rust
use torrentinfo::Torrent;

fn main() -> anyhow::Result<()> {
    let torrent = Torrent::from_file("example.torrent")?;

    println!("Name: {:?}", torrent.name());
    println!("Size: {} bytes", torrent.total_size());
    println!("Files: {}", torrent.num_files());

    Ok(())
}
```

## License

GPL-3.0
