use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use pal_asset::{make_pac_key, parse_archive_dat_paths, Nls, PacArchive, ResourceManager};

#[test]
fn archive_dat_parser_matches_pal_delimiters() {
    let bytes = b" data | bg\r\n| fgimage\t |";
    let paths = parse_archive_dat_paths(bytes, Nls::ShiftJis).unwrap();
    assert_eq!(paths, vec!["data", "bg", "fgimage"]);
}

#[test]
fn pac_archive_reads_bucketed_entry() {
    let dir = temp_dir("pal_asset_pac");
    let pac_path = dir.join("data.pac");
    let key = make_pac_key("script.src", Nls::ShiftJis).unwrap();
    fs::write(&pac_path, make_one_entry_pac(key, b"Sv20fixture")).unwrap();

    let pac = PacArchive::from_file(&pac_path).unwrap();
    let data = pac.read_key(&key).unwrap().unwrap();
    assert_eq!(data, b"Sv20fixture");

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn resource_manager_bootstraps_archive_dat_and_opens_from_pac() {
    let dir = temp_dir("pal_asset_resource");
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("data").join("archive.dat"), b"data|bg|").unwrap();

    let key = make_pac_key("script.src", Nls::ShiftJis).unwrap();
    fs::write(dir.join("data.pac"), make_one_entry_pac(key, b"Sv20script")).unwrap();

    let mut manager = ResourceManager::bootstrap(&dir, Nls::ShiftJis).unwrap();
    let asset = manager.open("script.src").unwrap();
    assert_eq!(asset.bytes, b"Sv20script");
    assert_eq!(manager.paths(), &["data".to_string(), "bg".to_string()]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn resource_manager_bootstraps_archive_dat_stored_in_data_pac() {
    let dir = temp_dir("pal_asset_archive_in_pac");
    let archive_key = make_pac_key("ARCHIVE.DAT", Nls::ShiftJis).unwrap();
    let script_key = make_pac_key("script.src", Nls::ShiftJis).unwrap();
    fs::write(
        dir.join("data.pac"),
        make_two_entry_pac(
            archive_key,
            b"movie|bgm|mask|em|ev|ev2|etc|bk|se|system|face|voice|st|st2",
            script_key,
            b"Sv20script",
        ),
    )
    .unwrap();

    let mut manager = ResourceManager::bootstrap(&dir, Nls::ShiftJis).unwrap();
    assert_eq!(
        manager.paths(),
        &[
            "data", "movie", "bgm", "mask", "em", "ev", "ev2", "etc", "bk", "se", "system",
            "face", "voice", "st", "st2",
        ]
        .map(str::to_string)
    );
    let asset = manager.open("script.src").unwrap();
    assert_eq!(asset.bytes, b"Sv20script");
    assert!(matches!(asset.source, pal_asset::AssetSource::Pac { .. }));

    fs::remove_dir_all(dir).unwrap();
}

fn make_one_entry_pac(key: [u8; 32], payload: &[u8]) -> Vec<u8> {
    make_entries_pac(&[(key, payload)])
}

fn make_two_entry_pac(
    key_a: [u8; 32],
    payload_a: &[u8],
    key_b: [u8; 32],
    payload_b: &[u8],
) -> Vec<u8> {
    make_entries_pac(&[(key_a, payload_a), (key_b, payload_b)])
}

fn make_entries_pac(entries: &[([u8; 32], &[u8])]) -> Vec<u8> {
    const TABLE_OFF: usize = 0x0C;
    const TABLE_SIZE: usize = 255 * 8;
    const RECORD_BASE: usize = TABLE_OFF + TABLE_SIZE;
    const RECORD_SIZE: usize = 40;

    let mut by_bucket: Vec<Vec<([u8; 32], &[u8])>> = vec![Vec::new(); 255];
    for entry in entries {
        by_bucket[entry.0[0] as usize].push(*entry);
    }

    let record_count: usize = entries.len();
    let data_base = RECORD_BASE + record_count * RECORD_SIZE;
    let payload_len: usize = entries.iter().map(|(_, payload)| payload.len()).sum();
    let mut bytes = vec![0u8; data_base + payload_len];

    let mut record_index = 0u32;
    let mut data_cursor = data_base;
    for (bucket, bucket_entries) in by_bucket.iter().enumerate() {
        if bucket_entries.is_empty() {
            continue;
        }
        let bucket_off = TABLE_OFF + bucket * 8;
        bytes[bucket_off..bucket_off + 4].copy_from_slice(&record_index.to_le_bytes());
        bytes[bucket_off + 4..bucket_off + 8]
            .copy_from_slice(&(bucket_entries.len() as u32).to_le_bytes());
        for (key, payload) in bucket_entries {
            let record_off = RECORD_BASE + record_index as usize * RECORD_SIZE;
            bytes[record_off..record_off + 32].copy_from_slice(key);
            bytes[record_off + 32..record_off + 36]
                .copy_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes[record_off + 36..record_off + 40]
                .copy_from_slice(&(data_cursor as u32).to_le_bytes());
            bytes[data_cursor..data_cursor + payload.len()].copy_from_slice(payload);
            data_cursor += payload.len();
            record_index += 1;
        }
    }
    bytes
}

fn temp_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{}_{}", prefix, nonce));
    fs::create_dir_all(&path).unwrap();
    path
}
