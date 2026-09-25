//! Shared Koikake save header used by thumbnail, title, mosaic, and lock.
//!
//! In the observed 128x72 saves, the native image continues after the pixels
//! with fixed blocks through `0x13338` and a variable-length subsystem section.
//! This module only models the common header. `SENARSAV` immediately after the
//! pixels is a portable sena-rs extension, not a native image. `thumbnail_set` reads RGBA at
//! `0x224`; `save_lock` reads and writes the first dword.

use std::path::{Path, PathBuf};

pub const TITLE_LEN: usize = 0x200;
pub const HEADER_BEFORE_PIXELS: usize = 0x224;
pub const MOSAIC_OFFSET: usize = 0x208;
/// `c20+0x20`: script offset of the text command that was current when the
/// image was assembled. The native engine re-enters the script at this
/// position after `load` (koikake save010/save005 verified).
pub const RESUME_PC_OFFSET: usize = 0x20C;
/// `c20+0x38`: secondary script position (park/return address written by
/// `set_load_after_process`; constant per game).
pub const SECONDARY_PC_OFFSET: usize = 0x210;
/// Current text value (string id) inside the fixed `wrapper+0x22A44` block
/// (file offset 0x12A30 + 0x0C; koikake save010 = 6557, save005 = 6649).
pub const TEXT_VALUE_OFFSET: usize = 0x12A3C;
pub const THUMB_WIDTH_OFFSET: usize = 0x214;
pub const THUMB_HEIGHT_OFFSET: usize = 0x218;
pub const THUMB_BYTES_OFFSET: usize = 0x21C;
pub const LOCK_OFFSET: usize = 0;
/// Native `PalThumbnailCreateMosaic` call passes this factor.
pub const MOSAIC_FACTOR: u32 = 6;
pub const DEFAULT_THUMB_WIDTH: i32 = 0x80;
pub const DEFAULT_THUMB_HEIGHT: i32 = 0x48;
/// `load_thumbnail` uses this sentinel to enable screen capture.
pub const LOAD_THUMBNAIL_CAPTURE_SENTINEL: i32 = 0x0FFF_FFFF;
/// Thumbnail RGBA larger than this is rejected instead of being read into memory.
pub const MAX_THUMB_BYTES: usize = 8 * 1024 * 1024;
const SNAPSHOT_MAGIC: &[u8] = b"SENARSAV";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginalSavePrefix {
    pub lock: i32,
    pub title: Vec<u8>,
    pub mosaic: i32,
    pub resume_pc: i32,
    pub secondary_pc: i32,
    pub thumb_width: i32,
    pub thumb_height: i32,
    pub pixels: Vec<u8>,
}

impl OriginalSavePrefix {
    pub fn empty(width: i32, height: i32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self {
            lock: 0,
            title: Vec::new(),
            mosaic: 0,
            resume_pc: 0,
            secondary_pc: -1,
            thumb_width: width,
            thumb_height: height,
            pixels: vec![0; width as usize * height as usize * 4],
        }
    }
}

pub fn original_save_filename(slot: i32) -> String {
    if slot < 0 {
        "continue.dat".to_owned()
    } else {
        format!("save{slot:03}.dat")
    }
}

pub fn original_save_path(root: &Path, slot: i32) -> PathBuf {
    root.join("save").join(original_save_filename(slot))
}

pub fn encode_original_save(prefix: &OriginalSavePrefix, snapshot: &[u8]) -> Vec<u8> {
    let width = prefix.thumb_width.max(0) as usize;
    let height = prefix.thumb_height.max(0) as usize;
    let expected = width.saturating_mul(height).saturating_mul(4);
    let mut pixels = prefix.pixels.clone();
    if pixels.len() != expected {
        pixels.resize(expected, 0);
    }
    let mut out = vec![0u8; HEADER_BEFORE_PIXELS + pixels.len()];
    write_i32(&mut out, LOCK_OFFSET, prefix.lock);
    let title_len = prefix.title.len().min(TITLE_LEN.saturating_sub(1));
    out[8..8 + title_len].copy_from_slice(&prefix.title[..title_len]);
    write_i32(&mut out, MOSAIC_OFFSET, prefix.mosaic);
    write_i32(&mut out, RESUME_PC_OFFSET, prefix.resume_pc);
    write_i32(&mut out, SECONDARY_PC_OFFSET, prefix.secondary_pc);
    write_i32(&mut out, THUMB_WIDTH_OFFSET, prefix.thumb_width);
    write_i32(&mut out, THUMB_HEIGHT_OFFSET, prefix.thumb_height);
    write_i32(&mut out, THUMB_BYTES_OFFSET, pixels.len() as i32);
    write_i32(&mut out, 0x220, 1);
    out[HEADER_BEFORE_PIXELS..].copy_from_slice(&pixels);
    if snapshot.starts_with(SNAPSHOT_MAGIC) {
        out.extend_from_slice(snapshot);
    }
    out
}

/// Byte length of the header plus thumbnail, from the first `0x224` bytes.
///
/// Callers use this to read a prefix without mapping the VM trailer. A 1 GiB
/// trailer must not be pulled in just to draw a save-slot thumbnail.
pub fn original_prefix_len(header: &[u8]) -> Option<usize> {
    if header.len() < HEADER_BEFORE_PIXELS || header.starts_with(SNAPSHOT_MAGIC) {
        return None;
    }
    let width = read_i32(header, THUMB_WIDTH_OFFSET)?;
    let height = read_i32(header, THUMB_HEIGHT_OFFSET)?;
    let pixel_len = read_i32(header, THUMB_BYTES_OFFSET)?.max(0) as usize;
    if width < 0 || height < 0 || pixel_len > MAX_THUMB_BYTES {
        return None;
    }
    let expected = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if pixel_len != expected && pixel_len != 0 {
        return None;
    }
    HEADER_BEFORE_PIXELS.checked_add(pixel_len)
}

pub fn decode_original_save(bytes: &[u8]) -> Option<(OriginalSavePrefix, Option<&[u8]>)> {
    if bytes.starts_with(SNAPSHOT_MAGIC) || bytes.len() < HEADER_BEFORE_PIXELS {
        return None;
    }
    let width = read_i32(bytes, THUMB_WIDTH_OFFSET)?;
    let height = read_i32(bytes, THUMB_HEIGHT_OFFSET)?;
    let pixel_len = read_i32(bytes, THUMB_BYTES_OFFSET)?.max(0) as usize;
    let pixel_end = HEADER_BEFORE_PIXELS.checked_add(pixel_len)?;
    if pixel_end > bytes.len() || width < 0 || height < 0 {
        return None;
    }
    let expected = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if pixel_len != expected && pixel_len != 0 {
        return None;
    }
    let title_end = bytes[8..8 + TITLE_LEN]
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(TITLE_LEN);
    let prefix = OriginalSavePrefix {
        lock: read_i32(bytes, LOCK_OFFSET)?,
        title: bytes[8..8 + title_end].to_vec(),
        mosaic: read_i32(bytes, MOSAIC_OFFSET)?,
        resume_pc: read_i32(bytes, RESUME_PC_OFFSET)?,
        secondary_pc: read_i32(bytes, SECONDARY_PC_OFFSET)?,
        thumb_width: width,
        thumb_height: height,
        pixels: bytes[HEADER_BEFORE_PIXELS..pixel_end].to_vec(),
    };
    let rest = &bytes[pixel_end..];
    let snapshot = rest.starts_with(SNAPSHOT_MAGIC).then_some(rest);
    Some((prefix, snapshot))
}

pub fn read_lock_dword(bytes: &[u8]) -> i32 {
    if bytes.starts_with(SNAPSHOT_MAGIC) || bytes.len() < 4 {
        return 0;
    }
    read_i32(bytes, 0).unwrap_or(0)
}

/// Current text value (string id) stored in the fixed `wrapper+0x22A44`
/// block of an original save image.
pub fn read_original_text_value(bytes: &[u8]) -> Option<i32> {
    if bytes.starts_with(SNAPSHOT_MAGIC) {
        return None;
    }
    read_i32(bytes, TEXT_VALUE_OFFSET)
}

/// Point-sample downsample by `factor`, then replicate each sample back to the
/// original size. Matches the visible result of `PalThumbnailCreateMosaic`.
pub fn mosaic_rgba(pixels: &[u8], width: u32, height: u32, factor: u32) -> Vec<u8> {
    let factor = factor.max(1);
    let expected = width as usize * height as usize * 4;
    if factor == 1 || width == 0 || height == 0 || pixels.len() < expected {
        return pixels.get(..expected).unwrap_or(pixels).to_vec();
    }
    let mut out = vec![0u8; expected];
    for y in 0..height {
        let src_y = (y / factor) * factor;
        for x in 0..width {
            let src_x = (x / factor) * factor;
            let src = ((src_y as usize * width as usize) + src_x as usize) * 4;
            let dst = ((y as usize * width as usize) + x as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
        }
    }
    out
}

#[derive(Clone, Debug)]
pub struct ThumbnailSprite {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Composite positioned sprites into a thumbnail, then optionally mosaic it.
pub fn composite_thumbnail(
    sprites: &[ThumbnailSprite],
    logical_width: u32,
    logical_height: u32,
    thumb_width: u32,
    thumb_height: u32,
    mosaic_factor: Option<u32>,
) -> Vec<u8> {
    let thumb_width = thumb_width.max(1);
    let thumb_height = thumb_height.max(1);
    let logical_width = logical_width.max(1);
    let logical_height = logical_height.max(1);
    let mut canvas = vec![0u8; thumb_width as usize * thumb_height as usize * 4];
    for sprite in sprites {
        if sprite.width == 0
            || sprite.height == 0
            || sprite.rgba.len() < sprite.width as usize * sprite.height as usize * 4
        {
            continue;
        }
        let dest_x = sprite.x as i64 * thumb_width as i64 / logical_width as i64;
        let dest_y = sprite.y as i64 * thumb_height as i64 / logical_height as i64;
        let dest_w = (sprite.width as i64 * thumb_width as i64 / logical_width as i64).max(1);
        let dest_h = (sprite.height as i64 * thumb_height as i64 / logical_height as i64).max(1);
        for dy in 0..dest_h {
            let y = dest_y + dy;
            if y < 0 || y >= thumb_height as i64 {
                continue;
            }
            let src_y =
                (dy * sprite.height as i64 / dest_h).clamp(0, sprite.height as i64 - 1) as u32;
            for dx in 0..dest_w {
                let x = dest_x + dx;
                if x < 0 || x >= thumb_width as i64 {
                    continue;
                }
                let src_x =
                    (dx * sprite.width as i64 / dest_w).clamp(0, sprite.width as i64 - 1) as u32;
                let src = ((src_y as usize * sprite.width as usize) + src_x as usize) * 4;
                let dst = ((y as usize * thumb_width as usize) + x as usize) * 4;
                let src_px = &sprite.rgba[src..src + 4];
                if src_px[3] == 0 {
                    continue;
                }
                canvas[dst..dst + 4].copy_from_slice(src_px);
            }
        }
    }
    if let Some(factor) = mosaic_factor {
        mosaic_rgba(&canvas, thumb_width, thumb_height, factor)
    } else {
        canvas
    }
}

fn write_i32(out: &mut [u8], offset: usize, value: i32) {
    out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    let end = offset.checked_add(4)?;
    if end > bytes.len() {
        return None;
    }
    Some(i32::from_le_bytes(bytes[offset..end].try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_prefix_round_trips_thumbnail_lock_and_snapshot() {
        let mut pixels = vec![0u8; 2 * 2 * 4];
        pixels[0] = 9;
        pixels[3] = 255;
        let prefix = OriginalSavePrefix {
            lock: 7,
            title: b"scene".to_vec(),
            mosaic: 1,
            resume_pc: 0x1234,
            secondary_pc: -1,
            thumb_width: 2,
            thumb_height: 2,
            pixels: pixels.clone(),
        };
        let snapshot = b"SENARSAVpayload";
        let bytes = encode_original_save(&prefix, snapshot);
        assert_eq!(read_lock_dword(&bytes), 7);
        assert_eq!(&bytes[..4], &7i32.to_le_bytes()[..]);
        let (decoded, trailer) = decode_original_save(&bytes).expect("original save");
        assert_eq!(decoded.lock, 7);
        assert_eq!(decoded.title, b"scene");
        assert_eq!(decoded.mosaic, 1);
        assert_eq!(decoded.resume_pc, 0x1234);
        assert_eq!(decoded.secondary_pc, -1);
        assert_eq!(decoded.pixels, pixels);
        assert_eq!(trailer, Some(snapshot.as_slice()));
        assert_eq!(
            original_prefix_len(&bytes[..HEADER_BEFORE_PIXELS]),
            Some(HEADER_BEFORE_PIXELS + pixels.len())
        );
    }

    #[test]
    fn mosaic_replicates_factor_blocks() {
        let mut pixels = vec![0u8; 4 * 2 * 4];
        pixels[0..4].copy_from_slice(&[1, 2, 3, 255]);
        let mosaiced = mosaic_rgba(&pixels, 4, 2, 2);
        assert_eq!(&mosaiced[0..4], &[1, 2, 3, 255]);
        assert_eq!(&mosaiced[4..8], &[1, 2, 3, 255]);
        assert_eq!(&mosaiced[8..12], &[0, 0, 0, 0]);
    }

    #[test]
    fn composite_thumbnail_scales_a_sprite_into_the_slot() {
        let sprite = ThumbnailSprite {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
            rgba: vec![8, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        };
        let thumb = composite_thumbnail(&[sprite], 4, 4, 2, 2, None);
        assert_eq!(&thumb[0..4], &[8, 0, 0, 255]);
    }
}
