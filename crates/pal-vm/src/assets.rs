use std::collections::BTreeMap;

use pal_asset::{LoadedAsset, ResourceManager};
use pal_script::{PointTable, ScriptImage};

use crate::config::SystemDat;

#[derive(Debug)]
pub struct CoreAssets {
    pub script: LoadedAsset,
    pub file_dat: LoadedAsset,
    pub text_dat: LoadedAsset,
    pub mem_dat: LoadedAsset,
    pub point_dat: LoadedAsset,
    pub graphic_dat: Option<LoadedAsset>,
    pub script_check_value: u32,
    pub script_entry_pc: u32,
    /// A later SoftPAL extension set exposes the millisecond clock at 18:121
    /// and uses a different SE load/play argument contract.
    pub extended_softpal: bool,
    pub point_table: PointTable,
    pub graphic_index: Option<GraphicIndex>,
}

impl CoreAssets {
    pub fn load(
        resource_manager: &mut ResourceManager,
        system_dat: Option<&SystemDat>,
    ) -> anyhow::Result<Self> {
        let script = resource_manager.open("Script.src")?;
        let script_image = ScriptImage::parse(&script.bytes)?;

        if let Some(system_dat) = system_dat {
            if !system_dat.matches_script_check(script_image.check_value()) {
                return Err(anyhow::anyhow!(
                    "system.dat check value 0x{:08X} does not match Script.src check value 0x{:08X}",
                    system_dat.script_check_value,
                    script_image.check_value()
                ));
            }
        }

        let file_dat = resource_manager.open_decrypting("File.dat")?;
        let text_dat = resource_manager.open_decrypting("Text.dat")?;
        let mem_dat = resource_manager.open_decrypting("Mem.dat")?;
        let point_dat = resource_manager.open_decrypting("Point.dat")?;
        let point_table = PointTable::parse(&point_dat.bytes)?;

        let graphic_dat = match resource_manager.open_decrypting("graphic.dat") {
            Ok(asset) => Some(asset),
            Err(pal_asset::AssetError::AssetNotFound { .. }) => None,
            Err(err) => return Err(err.into()),
        };
        let graphic_index = match graphic_dat.as_ref() {
            Some(asset) => match GraphicIndex::parse(&asset.bytes) {
                Ok(index) => Some(index),
                Err(err) => {
                    log::warn!("graphic.dat parse failed (non-fatal): {err}");
                    None
                }
            },
            None => None,
        };

        // Extract values before moving `script`; script_image borrows script.bytes.
        let script_check_value = script_image.check_value();
        let script_entry_pc = script_image.entry_pc();
        // Probe an aligned extcall opcode/category pair rather than a game
        // title. The later script uses 18:121 as its PAL millisecond clock.
        let extended_softpal = script
            .bytes[12..]
            .windows(8)
            .step_by(4)
            .any(|words| words == [0x17, 0x00, 0x01, 0x00, 0x79, 0x00, 0x12, 0x00]);
        let _ = script_image; // release borrow of script.bytes before moving script

        Ok(Self {
            script,
            file_dat,
            text_dat,
            mem_dat,
            point_dat,
            graphic_dat,
            script_check_value,
            script_entry_pc,
            extended_softpal,
            point_table,
            graphic_index,
        })
    }

    pub fn script_image(&self) -> anyhow::Result<ScriptImage<'_>> {
        Ok(ScriptImage::parse(&self.script.bytes)?)
    }
}

#[derive(Debug)]
pub struct GraphicIndex {
    pub buckets: Vec<GraphicBucket>,
    pub data_offset: usize,
    pub entries_by_key: BTreeMap<[u8; 32], GraphicEntry>,
    pub records: Vec<GraphicRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphicBucket {
    pub record_count: u16,
    pub record_offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicEntry {
    pub key: [u8; 32],
    pub data_size: u32,
    pub data_offset: u32,
    pub bucket: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphicRecord {
    pub key: [u8; 32],
    pub replacement_name: [u8; 64],
    pub animation_name: [u8; 64],
    pub flags: u32,
    pub priority_lane: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub scale_percent: i32,
    pub alpha: i32,
    pub bucket: u8,
}

impl GraphicRecord {
    pub fn replacement_resource(&self) -> Option<String> {
        graphic_c_string(&self.replacement_name)
    }

    pub fn animation_resource(&self) -> Option<String> {
        graphic_c_string(&self.animation_name)
    }
}

impl GraphicIndex {
    pub const BUCKET_COUNT: usize = 255;
    pub const BUCKET_TABLE_SIZE: usize = 0x7F8;
    pub const RECORD_BASE_AFTER_DOLLAR_HEADER: usize = 0x10 + Self::BUCKET_TABLE_SIZE;
    pub const RECORD_SIZE: usize = 0x84;

    pub fn parse(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() < 0x10 + Self::BUCKET_TABLE_SIZE {
            return Err(anyhow::anyhow!(
                "graphic.dat is too small: got 0x{:X}, expected at least 0x{:X}",
                bytes.len(),
                0x10 + Self::BUCKET_TABLE_SIZE
            ));
        }

        let bucket_base = 0x10;
        let mut buckets = Vec::with_capacity(Self::BUCKET_COUNT);
        for bucket in 0..Self::BUCKET_COUNT {
            let offset = bucket_base + bucket * 8;
            buckets.push(GraphicBucket {
                record_count: read_u16(bytes, offset)?,
                record_offset: read_u32(bytes, offset + 4)?,
            });
        }

        let mut entries_by_key = BTreeMap::new();
        let mut records = Vec::new();
        for (bucket, info) in buckets.iter().enumerate() {
            let count = info.record_count as usize;
            let bucket_data_offset = Self::RECORD_BASE_AFTER_DOLLAR_HEADER
                .checked_add(info.record_offset as usize)
                .ok_or_else(|| anyhow::anyhow!("graphic.dat bucket data offset overflows"))?;
            for record_index in 0..count {
                let offset =
                    bucket_data_offset
                        .checked_add(record_index.checked_mul(Self::RECORD_SIZE).ok_or_else(
                            || anyhow::anyhow!("graphic.dat record offset overflows"),
                        )?)
                        .ok_or_else(|| anyhow::anyhow!("graphic.dat record offset overflows"))?;
                if offset + Self::RECORD_SIZE > bytes.len() {
                    return Err(anyhow::anyhow!(
                        "graphic.dat record {} is out of range at 0x{:X}",
                        record_index,
                        offset
                    ));
                }
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[offset..offset + 32]);
                normalize_graphic_key(&mut key);
                let mut replacement_name = [0u8; 64];
                replacement_name[..0x1C].copy_from_slice(&bytes[offset + 0x24..offset + 0x40]);
                normalize_graphic_name(&mut replacement_name);
                let mut animation_name = [0u8; 64];
                animation_name.copy_from_slice(&bytes[offset + 0x44..offset + 0x84]);
                normalize_graphic_name(&mut animation_name);
                let flags = read_u32(bytes, offset + 0x20)?;
                // Koikake `sub_435230` applies lanes from this compact 0x84-byte
                // record when the corresponding flag bits are set.  The caller
                // that installs a sprite passes mask -1, so every set bit applies.
                // +0xC4/+0xD4 belong to a different in-memory wrapper and must
                // not be read from this file.
                let priority_lane = if flags & 0x2000 != 0 {
                    read_i32(bytes, offset + 0x68)?.saturating_add(1)
                } else {
                    0
                };
                let (offset_x, offset_y) = if flags & 0x60000 != 0 {
                    (
                        read_i32(bytes, offset + 0x6C)?,
                        read_i32(bytes, offset + 0x70)?,
                    )
                } else {
                    (0, 0)
                };
                let scale_percent = if flags & 0x100000 != 0 {
                    read_i32(bytes, offset + 0x74)?
                } else {
                    100
                };
                let alpha = if flags & 0x80000 != 0 {
                    read_i32(bytes, offset + 0x78)?
                } else {
                    0
                };
                records.push(GraphicRecord {
                    key,
                    replacement_name,
                    animation_name,
                    flags,
                    priority_lane,
                    offset_x,
                    offset_y,
                    scale_percent,
                    alpha,
                    bucket: bucket as u8,
                });
                entries_by_key.insert(
                    key,
                    GraphicEntry {
                        key,
                        data_size: Self::RECORD_SIZE as u32,
                        data_offset: info.record_offset + (record_index * Self::RECORD_SIZE) as u32,
                        bucket: bucket as u8,
                    },
                );
            }
        }

        Ok(Self {
            buckets,
            data_offset: Self::RECORD_BASE_AFTER_DOLLAR_HEADER,
            entries_by_key,
            records,
        })
    }

    pub fn len(&self) -> usize {
        self.entries_by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries_by_key.is_empty()
    }

    pub fn lookup(&self, name: &str) -> Option<&GraphicRecord> {
        let key = make_graphic_key(name);
        self.records.iter().find(|record| record.key == key)
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> anyhow::Result<u16> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| anyhow::anyhow!("read offset overflows"))?;
    if end > bytes.len() {
        return Err(anyhow::anyhow!("read out of range at 0x{:X}", offset));
    }
    Ok(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
}

fn read_i32(bytes: &[u8], offset: usize) -> anyhow::Result<i32> {
    Ok(read_u32(bytes, offset)? as i32)
}

fn read_u32(bytes: &[u8], offset: usize) -> anyhow::Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| anyhow::anyhow!("read offset overflows"))?;
    if end > bytes.len() {
        return Err(anyhow::anyhow!("read out of range at 0x{:X}", offset));
    }
    Ok(u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

fn make_graphic_key(name: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    let leaf = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem = leaf.split('.').next().unwrap_or(leaf);
    for (dst, src) in key.iter_mut().zip(
        stem.as_bytes()
            .iter()
            .copied()
            .map(|b| b.to_ascii_uppercase()),
    ) {
        *dst = src;
    }
    key
}

fn normalize_graphic_key(key: &mut [u8; 32]) {
    if key[1] == 0 && key[3] == 0 {
        let mut compact = [0u8; 32];
        for i in 0..20.min(key.len() / 2) {
            compact[i] = key[i * 2];
        }
        *key = compact;
    }
    for byte in key {
        if *byte == 0xCC {
            *byte = 0;
        } else {
            *byte = byte.to_ascii_uppercase();
        }
    }
}

fn normalize_graphic_name(name: &mut [u8; 64]) {
    if name[1] == 0 && name[3] == 0 {
        let mut compact = [0u8; 64];
        for i in 0..32.min(name.len() / 2) {
            compact[i] = name[i * 2];
        }
        *name = compact;
    }
    for byte in name {
        if *byte == 0xCC {
            *byte = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphic_record_lanes_follow_flag_bits() {
        let mut bytes =
            vec![0u8; GraphicIndex::RECORD_BASE_AFTER_DOLLAR_HEADER + GraphicIndex::RECORD_SIZE];
        bytes[0x10] = 1;
        let record = GraphicIndex::RECORD_BASE_AFTER_DOLLAR_HEADER;
        bytes[record..record + 8].copy_from_slice(b"HANABI_B");
        bytes[record + 0x20..record + 0x24].copy_from_slice(&0x0008_7000u32.to_le_bytes());
        bytes[record + 0x68..record + 0x6C].copy_from_slice(&1i32.to_le_bytes());
        bytes[record + 0x6C..record + 0x70].copy_from_slice(&3i32.to_le_bytes());
        bytes[record + 0x70..record + 0x74].copy_from_slice(&(-4i32).to_le_bytes());
        bytes[record + 0x78..record + 0x7C].copy_from_slice(&0x00FF_0000u32.to_le_bytes());
        // 0x60000 is not set, so offsets stay neutral even though the dwords are filled.
        bytes[record + 0x20] = 0x00;
        bytes[record + 0x21] = 0x70;
        bytes[record + 0x22] = 0x08;
        let index = GraphicIndex::parse(&bytes).expect("graphic.dat");
        let parsed = index.lookup("hanabi_b").expect("record");
        assert_eq!(parsed.priority_lane, 2);
        assert_eq!(parsed.offset_x, 0);
        assert_eq!(parsed.offset_y, 0);
        assert_eq!(parsed.scale_percent, 100);
        assert_eq!(parsed.alpha, 0x00FF_0000);

        bytes[record + 0x20..record + 0x24].copy_from_slice(&0x0016_7000u32.to_le_bytes());
        bytes[record + 0x74..record + 0x78].copy_from_slice(&80i32.to_le_bytes());
        let index = GraphicIndex::parse(&bytes).expect("graphic.dat with placement bits");
        let parsed = index.lookup("HANABI_B").expect("record");
        assert_eq!(parsed.offset_x, 3);
        assert_eq!(parsed.offset_y, -4);
        assert_eq!(parsed.scale_percent, 80);
    }
}

fn graphic_c_string(bytes: &[u8]) -> Option<String> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    if end == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes[..end]).to_string())
}
