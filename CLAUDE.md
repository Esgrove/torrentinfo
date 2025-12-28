# Agent instructions

## Project Overview

This is a Rust CLI tool for parsing and displaying information from `.torrent` files.
The library code lives in `src/lib.rs` with the CLI tool in `src/cli.rs`.

## Build and Test Commands

After making code changes, always run:

```shell
cargo clippy --fix --allow-dirty
cargo fmt
cargo test
```

### Other commands

```shell
# Build
cargo build

# Run with arguments
cargo run -- [args]

# Format code
cargo fmt
```

## Project Structure

- `src/lib.rs` - Core library: `Torrent`, `Info`, `File` structs and parsing logic
- `src/errors.rs` - Error types using `thiserror`
- `src/main.rs` - CLI entry point and argument parsing
- `src/cli.rs` - `TorrentInfo` struct handling torrent display logic
- `src/utils.rs` - Utility functions for file handling, formatting, and path resolution
- `tests/` - Integration tests with example `.torrent` files

## Code organization

- Put all struct definitions before their implementations
- Functions after implementations
- In implementations, order public methods before private methods
- In implementations, put associated functions last

## Code Style and Conventions

- Uses Rust 2024 edition
- Clippy is configured with pedantic and nursery lints enabled
- Do not use plain unwrap. Use proper error handling or `.expect()` in constants and test cases.
- Use `anyhow` for error handling in the binary, `thiserror` for library error types
- Use `clap` with derive macros for CLI argument parsing
- Use `colored` crate for terminal output coloring
- Use descriptive variable and function names. No single character variables.
- Prefer full names over abbreviations. For example: `filepath` instead of `fp`.
- Create docstrings for structs and functions.
- Avoid trailing comments.

## Testing

- Unit tests are in `src/lib.rs` under `#[cfg(test)]`
- Integration tests are in `tests/torrent_tests.rs`
- Test torrent files are in `tests/` directory
- Always add test cases for new features and functionality
