# torrentinfo

A torrent file parser library and CLI utility.

## Usage

By default,
`torrentinfo` will print the information for all `.torrent` files directly in the working directory.
An optional path to a directory or torrent file can be specified,
with a recursive option to recursively find all torrent files.

```
Usage: torrentinfo [OPTIONS] [PATH]

Arguments:
  [PATH]  Optional input directory or file

Options:
  -d, --details     Show detailed information about the torrent
  -e, --everything  Print everything about the torrent
  -f, --files       Show files within the torrent
  -n, --nocolour    Disable colour output
  -r, --recursive   Recursive directory iteration
  -v, --verbose     Verbose output
  -h, --help        Print help
  -V, --version     Print version
```

## Installation

```bash
cargo install torrentinfo
```

Or from source

```bash
git clone https://github.com/fuchsi/torrentinfo.git
cd torrentinfo
cargo install
```