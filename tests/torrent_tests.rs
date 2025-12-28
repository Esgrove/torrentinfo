//! Integration tests for torrentinfo library

use std::path::Path;

use torrentinfo::{File, Info, Torrent, to_hex};

/// Path to the Ubuntu test torrent file
const UBUNTU_TORRENT_PATH: &str = "tests/ubuntu-24.04.3-desktop-amd64.iso.torrent";

/// Path to the Pop!_OS test torrent file
const POPOS_TORRENT_PATH: &str = "tests/pop-os_24.04_amd64_nvidia_22.iso.torrent";

/// Expected values for the Ubuntu torrent
mod ubuntu {
    pub const NAME: &str = "ubuntu-24.04.3-desktop-amd64.iso";
    pub const COMMENT: &str = "Ubuntu CD releases.ubuntu.com";
    pub const ANNOUNCE_URL: &str = "https://torrent.ubuntu.com/announce";
    pub const CREATED_BY: &str = "mktorrent 1.1";
    pub const INFO_HASH: &str = "d160b8d8ea35a5b4e52837468fc8f03d55cef1f7";
    pub const NUM_FILES: usize = 1;
    pub const TOTAL_SIZE: i64 = 6_345_887_744;
    pub const PIECE_LENGTH: i64 = 262_144;
}

/// Expected values for the Pop!_OS torrent
mod popos {
    pub const NAME: &str = "pop-os_24.04_amd64_nvidia_22.iso";
    pub const COMMENT: &str = "Unofficial Pop!_OS (Nvidia - 24.04 - revision 22) torrent created by FOSS Torrents. Published on https://fosstorrents.com/";
    pub const ANNOUNCE_URL: &str = "udp://fosstorrents.com:6969/announce";
    pub const CREATED_BY: &str = "FOSS Torrents (https://fosstorrents.com/)";
    pub const INFO_HASH: &str = "d4d16dbb800d9560f92b3821c84800f7047c186b";
    pub const NUM_FILES: usize = 1;
    pub const TOTAL_SIZE: i64 = 3_600_056_320;
    pub const PIECE_LENGTH: i64 = 1_048_576;
}

// Ubuntu torrent tests

#[test]
fn test_ubuntu_load_torrent_from_file() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH);
    assert!(torrent.is_ok(), "Should successfully load Ubuntu torrent file");
}

#[test]
fn test_ubuntu_load_torrent_from_path() {
    let path = Path::new(UBUNTU_TORRENT_PATH);
    let torrent = Torrent::from_file(path);
    assert!(torrent.is_ok(), "Should successfully load Ubuntu torrent from Path");
}

#[test]
fn test_ubuntu_torrent_name() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    assert_eq!(torrent.name().as_deref(), Some(ubuntu::NAME));
}

#[test]
fn test_ubuntu_torrent_comment() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    assert_eq!(torrent.comment().as_deref(), Some(ubuntu::COMMENT));
}

#[test]
fn test_ubuntu_torrent_announce_url() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    assert_eq!(torrent.announce().as_deref(), Some(ubuntu::ANNOUNCE_URL));
}

#[test]
fn test_ubuntu_torrent_created_by() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    assert_eq!(torrent.created_by().as_deref(), Some(ubuntu::CREATED_BY));
}

#[test]
fn test_ubuntu_torrent_creation_date() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    assert!(torrent.creation_date().is_some());
    let creation_date = torrent.creation_date().unwrap();
    assert!(creation_date > 0);
    // Should be after 2024 (timestamp > 1704067200)
    assert!(creation_date > 1_704_067_200);
}

#[test]
fn test_ubuntu_torrent_num_files() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    assert_eq!(torrent.num_files(), ubuntu::NUM_FILES);
}

#[test]
fn test_ubuntu_torrent_total_size() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    assert_eq!(torrent.total_size(), ubuntu::TOTAL_SIZE);
}

#[test]
fn test_ubuntu_torrent_info_hash() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    let info_hash = torrent.info_hash().expect("Should calculate info hash");
    assert_eq!(info_hash.len(), 20, "SHA-1 hash should be 20 bytes");
    let hex_hash = to_hex(&info_hash);
    assert_eq!(hex_hash, ubuntu::INFO_HASH);
}

#[test]
fn test_ubuntu_torrent_piece_length() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    let piece_length = *torrent.info().piece_length();
    assert_eq!(piece_length, ubuntu::PIECE_LENGTH);
}

#[test]
fn test_ubuntu_torrent_pieces_exist() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    let pieces = torrent.info().pieces();
    assert!(!pieces.is_empty(), "Pieces should not be empty");
    // SHA-1 hashes are 20 bytes each, so pieces length should be divisible by 20
    assert_eq!(pieces.len() % 20, 0);
}

#[test]
fn test_ubuntu_torrent_announce_list() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    if let Some(announce_list) = torrent.announce_list() {
        assert!(
            !announce_list.is_empty(),
            "Announce list should not be empty if present"
        );
    }
}

#[test]
fn test_ubuntu_torrent_read_bytes() {
    let bytes = Torrent::read_bytes(Path::new(UBUNTU_TORRENT_PATH));
    assert!(bytes.is_ok(), "Should successfully read torrent bytes");
    assert!(!bytes.unwrap().is_empty(), "Torrent file should not be empty");
}

#[test]
fn test_ubuntu_torrent_from_buf() {
    let bytes = Torrent::read_bytes(Path::new(UBUNTU_TORRENT_PATH)).unwrap();
    let torrent = Torrent::from_buf(&bytes);
    assert!(torrent.is_ok(), "Should parse torrent from buffer");
    assert_eq!(torrent.unwrap().name().as_deref(), Some(ubuntu::NAME));
}

#[test]
fn test_ubuntu_torrent_roundtrip() {
    let torrent1 = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    let bytes = Torrent::read_bytes(Path::new(UBUNTU_TORRENT_PATH)).unwrap();
    let torrent2 = Torrent::from_buf(&bytes).unwrap();

    let hash1 = torrent1.info_hash().unwrap();
    let hash2 = torrent2.info_hash().unwrap();
    assert_eq!(hash1, hash2, "Info hashes should match");
}

#[test]
fn test_ubuntu_torrent_files_accessor() {
    let torrent = Torrent::from_file(UBUNTU_TORRENT_PATH).unwrap();
    // Ubuntu ISO is a single-file torrent, so files() returns None
    let files = torrent.files();
    if files.is_none() {
        assert!(torrent.info.length.is_some(), "Single-file torrent should have length");
    } else {
        assert!(
            !files.as_ref().unwrap().is_empty(),
            "Multi-file torrent should have files"
        );
    }
}

// Pop!_OS torrent tests

#[test]
fn test_popos_load_torrent_from_file() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH);
    assert!(torrent.is_ok(), "Should successfully load Pop!_OS torrent file");
}

#[test]
fn test_popos_load_torrent_from_path() {
    let path = Path::new(POPOS_TORRENT_PATH);
    let torrent = Torrent::from_file(path);
    assert!(torrent.is_ok(), "Should successfully load Pop!_OS torrent from Path");
}

#[test]
fn test_popos_torrent_name() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    assert_eq!(torrent.name().as_deref(), Some(popos::NAME));
}

#[test]
fn test_popos_torrent_comment() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    assert_eq!(torrent.comment().as_deref(), Some(popos::COMMENT));
}

#[test]
fn test_popos_torrent_announce_url() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    assert_eq!(torrent.announce().as_deref(), Some(popos::ANNOUNCE_URL));
}

#[test]
fn test_popos_torrent_created_by() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    assert_eq!(torrent.created_by().as_deref(), Some(popos::CREATED_BY));
}

#[test]
fn test_popos_torrent_creation_date() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    assert!(torrent.creation_date().is_some());
    let creation_date = torrent.creation_date().unwrap();
    assert!(creation_date > 0);
    // Should be after 2024 (timestamp > 1704067200)
    assert!(creation_date > 1_704_067_200);
}

#[test]
fn test_popos_torrent_num_files() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    assert_eq!(torrent.num_files(), popos::NUM_FILES);
}

#[test]
fn test_popos_torrent_total_size() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    assert_eq!(torrent.total_size(), popos::TOTAL_SIZE);
}

#[test]
fn test_popos_torrent_info_hash() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    let info_hash = torrent.info_hash().expect("Should calculate info hash");
    assert_eq!(info_hash.len(), 20, "SHA-1 hash should be 20 bytes");
    let hex_hash = to_hex(&info_hash);
    assert_eq!(hex_hash, popos::INFO_HASH);
}

#[test]
fn test_popos_torrent_piece_length() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    let piece_length = *torrent.info().piece_length();
    assert_eq!(piece_length, popos::PIECE_LENGTH);
}

#[test]
fn test_popos_torrent_pieces_exist() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    let pieces = torrent.info().pieces();
    assert!(!pieces.is_empty(), "Pieces should not be empty");
    // SHA-1 hashes are 20 bytes each, so pieces length should be divisible by 20
    assert_eq!(pieces.len() % 20, 0);
}

#[test]
fn test_popos_torrent_announce_list() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    if let Some(announce_list) = torrent.announce_list() {
        assert!(
            !announce_list.is_empty(),
            "Announce list should not be empty if present"
        );
    }
}

#[test]
fn test_popos_torrent_from_buf() {
    let bytes = Torrent::read_bytes(Path::new(POPOS_TORRENT_PATH)).unwrap();
    let torrent = Torrent::from_buf(&bytes);
    assert!(torrent.is_ok(), "Should parse torrent from buffer");
    assert_eq!(torrent.unwrap().name().as_deref(), Some(popos::NAME));
}

#[test]
fn test_popos_torrent_roundtrip() {
    let torrent1 = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    let bytes = Torrent::read_bytes(Path::new(POPOS_TORRENT_PATH)).unwrap();
    let torrent2 = Torrent::from_buf(&bytes).unwrap();

    let hash1 = torrent1.info_hash().unwrap();
    let hash2 = torrent2.info_hash().unwrap();
    assert_eq!(hash1, hash2, "Info hashes should match");
}

#[test]
fn test_popos_torrent_files_accessor() {
    let torrent = Torrent::from_file(POPOS_TORRENT_PATH).unwrap();
    let files = torrent.files();
    if files.is_none() {
        assert!(torrent.info.length.is_some(), "Single-file torrent should have length");
    } else {
        assert!(
            !files.as_ref().unwrap().is_empty(),
            "Multi-file torrent should have files"
        );
    }
}

// Error handling tests

#[test]
fn test_nonexistent_file_error() {
    let result = Torrent::from_file("nonexistent_file.torrent");
    assert!(result.is_err(), "Should return error for nonexistent file");
}

#[test]
fn test_invalid_torrent_data_error() {
    let invalid_data = b"this is not a valid torrent file";
    let result = Torrent::from_buf(invalid_data);
    assert!(result.is_err(), "Should return error for invalid data");
}

#[test]
fn test_empty_data_error() {
    let empty_data: &[u8] = &[];
    let result = Torrent::from_buf(empty_data);
    assert!(result.is_err(), "Should return error for empty data");
}

// Utility function tests

#[test]
fn test_to_hex_function() {
    assert_eq!(to_hex(b"hello"), "68656c6c6f");
    assert_eq!(to_hex(&[0x00, 0xff, 0x0f, 0xf0]), "00ff0ff0");
    assert_eq!(to_hex(&[]), "");
    assert_eq!(to_hex(&[0x00]), "00");
    assert_eq!(to_hex(&[0xff]), "ff");
}

// Struct default and construction tests

#[test]
fn test_file_struct() {
    let file = File::new(1024, vec!["path".to_string(), "to".to_string(), "file.txt".to_string()]);
    assert_eq!(file.length(), 1024);
    assert_eq!(file.path(), &["path", "to", "file.txt"]);
}

#[test]
fn test_info_default() {
    let info = Info::default();
    assert!(info.name().is_none());
    assert!(info.files.is_none());
    assert!(info.length.is_none());
    assert!(info.private().is_none());
    assert_eq!(*info.piece_length(), 0);
    assert!(info.pieces().is_empty());
}

#[test]
fn test_torrent_default() {
    let torrent = Torrent::default();
    assert!(torrent.name().is_none());
    assert!(torrent.comment().is_none());
    assert!(torrent.announce().is_none());
    assert!(torrent.announce_list().is_none());
    assert!(torrent.created_by().is_none());
    assert!(torrent.creation_date().is_none());
    assert!(torrent.encoding().is_none());
    assert!(torrent.files().is_none());
    assert_eq!(torrent.num_files(), 0);
    assert_eq!(torrent.total_size(), 0);
}
