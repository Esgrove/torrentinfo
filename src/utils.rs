const TORRENT_EXTENSION: &str = "torrent";
const MAX_WALK_DEPTH: usize = 999;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use chrono::{TimeZone, Utc};
use clap_complete::Shell;
use colored::{ColoredString, Colorize};
use number_prefix::NumberPrefix;
use walkdir::WalkDir;

/// Return file root and list of files from the input path that can be either a directory or single file.
pub fn get_torrent_files(input: &PathBuf, recursive: bool, verbose: bool) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
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
pub fn resolve_input_path(path: Option<&Path>) -> anyhow::Result<PathBuf> {
    let input_path = path.and_then(|p| p.to_str()).unwrap_or("").trim();
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
pub fn digit_count(number: usize) -> usize {
    if number < 10 {
        1
    } else {
        ((number as f64).log10() as usize) + 1
    }
}

/// Format creation date as a human-readable string
pub fn format_creation_date(timestamp: i64) -> String {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map_or_else(String::new, |d| d.to_string())
}

/// Format file size with appropriate units
pub fn format_file_size(size: f64) -> String {
    match NumberPrefix::decimal(size) {
        NumberPrefix::Standalone(bytes) => format!("{bytes} bytes"),
        NumberPrefix::Prefixed(prefix, n) => format!("{n:.2} {prefix}B"),
    }
}

/// Collect all torrent files from the given root path and sort by name.
fn get_all_torrent_files<P: AsRef<Path>>(root: P, recursive: bool) -> Vec<PathBuf> {
    let extension = OsStr::new(TORRENT_EXTENSION);
    let max_depth = if recursive { MAX_WALK_DEPTH } else { 1 };
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

/// Check if entry is a hidden file or directory (starts with '.')
#[must_use]
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().is_some_and(|s| s.starts_with('.'))
}

/// Format bool value as a coloured string.
#[must_use]
pub fn colorize_bool(value: bool) -> ColoredString {
    if value { "true".green() } else { "false".red() }
}

/// Generate a shell completion script for the given shell.
///
/// # Errors
/// Returns an error if:
/// - The shell completion directory cannot be determined or created
/// - The completion file cannot be generated or written
pub fn generate_shell_completion(
    shell: Shell,
    mut command: clap::Command,
    install: bool,
    command_name: &str,
) -> anyhow::Result<()> {
    if install {
        let out_dir = get_shell_completion_dir(shell, command_name)?;
        let path = clap_complete::generate_to(shell, &mut command, command_name, out_dir)?;
        println!("Completion file generated to: {}", path.display());
    } else {
        clap_complete::generate(shell, &mut command, command_name, &mut std::io::stdout());
    }
    Ok(())
}

/// Determine the appropriate directory for storing shell completions.
///
/// First checks if the user-specific directory exists,
/// then checks for the global directory.
/// If neither exist, creates and uses the user-specific dir.
fn get_shell_completion_dir(shell: Shell, name: &str) -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().expect("Failed to get home directory");

    // Special handling for oh-my-zsh.
    // Create custom "plugin", which will then have to be loaded in .zshrc
    if shell == Shell::Zsh {
        let omz_plugins = home.join(".oh-my-zsh/custom/plugins");
        if omz_plugins.exists() {
            let plugin_dir = omz_plugins.join(name);
            std::fs::create_dir_all(&plugin_dir)?;
            return Ok(plugin_dir);
        }
    }

    let user_dir = match shell {
        Shell::PowerShell => {
            if cfg!(windows) {
                home.join(r"Documents\PowerShell\completions")
            } else {
                home.join(".config/powershell/completions")
            }
        }
        Shell::Bash => home.join(".bash_completion.d"),
        Shell::Elvish => home.join(".elvish"),
        Shell::Fish => home.join(".config/fish/completions"),
        Shell::Zsh => home.join(".zsh/completions"),
        _ => anyhow::bail!("Unsupported shell"),
    };

    if user_dir.exists() {
        return Ok(user_dir);
    }

    let global_dir = match shell {
        Shell::PowerShell => {
            if cfg!(windows) {
                home.join(r"Documents\PowerShell\completions")
            } else {
                home.join(".config/powershell/completions")
            }
        }
        Shell::Bash => PathBuf::from("/etc/bash_completion.d"),
        Shell::Fish => PathBuf::from("/usr/share/fish/completions"),
        Shell::Zsh => PathBuf::from("/usr/share/zsh/site-functions"),
        _ => anyhow::bail!("Unsupported shell"),
    };

    if global_dir.exists() {
        return Ok(global_dir);
    }

    std::fs::create_dir_all(&user_dir)?;
    Ok(user_dir)
}
