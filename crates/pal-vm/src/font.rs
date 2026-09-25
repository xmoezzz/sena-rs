use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use pal_asset::Nls;
use std::collections::BTreeMap;

static DEFAULT_TTF_BYTES: &[u8] = include_bytes!("default.ttf");

#[derive(Clone, Debug)]
pub struct PalFontSystem {
    fallback: PalFontFallback,
    bitmap: Option<PalBitmapFont>,
    begun: bool,
    font_size: u16,
    font_type: u16,
    effect: u16,
    color: u32,
    effect_color: u32,
    ex_font_loaded: bool,
}

impl Default for PalFontSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PalFontSystem {
    pub fn new() -> Self {
        Self {
            fallback: PalFontFallback::default_ttf(),
            bitmap: None,
            begun: false,
            font_size: 28,
            font_type: 1,
            effect: 0,
            color: 0xFF00_0000,
            effect_color: 0xFFFF_FFFF,
            ex_font_loaded: false,
        }
    }

    pub fn begin(&mut self) -> bool {
        if self.font_type == 4 && self.bitmap.is_none() && !self.ex_font_loaded {
            self.font_type = 1;
        }
        self.begun = true;
        true
    }

    pub fn end(&mut self) -> bool {
        self.begun = false;
        true
    }

    pub fn is_begun(&self) -> bool {
        self.begun
    }

    pub fn set_color(&mut self, color: u32, effect_color: u32) {
        self.color = color;
        self.effect_color = effect_color;
    }

    pub fn color(&self) -> (u32, u32) {
        (self.color, self.effect_color)
    }

    pub fn set_effect(&mut self, effect: u16) {
        self.effect = effect;
    }

    pub fn effect(&self) -> u16 {
        self.effect
    }

    pub fn set_font_size(&mut self, font_size: u16) {
        self.font_size = font_size.max(1);
    }

    pub fn font_size(&self) -> u16 {
        self.font_size
    }

    pub fn set_type(&mut self, font_type: u16) -> bool {
        if font_type == 4 && self.bitmap.is_none() && !self.ex_font_loaded {
            return false;
        }
        self.font_type = font_type;
        true
    }

    pub fn font_type(&self) -> u16 {
        self.font_type
    }

    pub fn set_ex_font_loaded(&mut self, loaded: bool) {
        self.ex_font_loaded = loaded;
        if !loaded && self.bitmap.is_none() && self.font_type == 4 {
            self.font_type = 1;
        }
    }

    pub fn load_bitmap_font(&mut self, bytes: Vec<u8>, nls: Nls) -> Result<(), &'static str> {
        self.bitmap = Some(PalBitmapFont::parse(bytes, nls)?);
        Ok(())
    }

    pub fn measure(&self, text: &str) -> (u32, u32) {
        let font_size = f32::from(self.font_size.max(1));
        if self.font_type == 4 {
            if let Some(bitmap) = &self.bitmap {
                if bitmap.supports(text) {
                    return bitmap.measure_line(text, font_size);
                }
            }
        }
        self.fallback.measure_line(text, font_size)
    }

    pub fn rasterize(&self, text: &str) -> (u32, u32, Vec<u8>) {
        let color = argb_to_bgra(self.color);
        let font_size = f32::from(self.font_size.max(1));
        let (width, height, mut pixels) = if self.font_type == 4
            && self.bitmap.as_ref().is_some_and(|font| font.supports(text))
        {
            self.bitmap
                .as_ref()
                .unwrap()
                .rasterize_line(text, font_size, color)
        } else {
            self.fallback.rasterize_line(text, font_size, color)
        };
        for px in pixels.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
        if self.effect == 0 || text.is_empty() {
            return (width, height, pixels);
        }
        let edge = argb_to_rgba(self.effect_color);
        if edge[3] == 0 {
            return (width, height, pixels);
        }
        apply_text_edge(&pixels, width, height, edge)
    }
}

/// PAL's `DEFAULT_FONT.DAT` stores a direct lookup table followed by grayscale
/// glyph records. Two-byte character codes use the native PAL slot mapping
/// `(lead - 0x80) * 255 + trail`; the selected NLS converts rendered Unicode
/// text back to those original byte codes.
#[derive(Clone, Debug)]
struct PalBitmapFont {
    bytes: Vec<u8>,
    offsets: Vec<u32>,
    nls: Nls,
    em_size: u32,
    baseline: i32,
}

#[derive(Clone, Copy, Debug)]
struct PalBitmapGlyph<'a> {
    width: u32,
    height: u32,
    bearing_x: i32,
    bearing_y: i32,
    advance: u32,
    stride: u32,
    alpha: &'a [u8],
}

impl PalBitmapFont {
    fn parse(bytes: Vec<u8>, nls: Nls) -> Result<Self, &'static str> {
        if bytes.len() < 0x84 {
            return Err("font data is too short");
        }
        let first_offset = u32::from_le_bytes(bytes[0x80..0x84].try_into().unwrap()) as usize;
        if first_offset < 0x84 || first_offset > bytes.len() || first_offset % 4 != 0 {
            return Err("font lookup table has an invalid size");
        }
        let offsets = bytes[..first_offset]
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
            .collect::<Vec<_>>();
        if offsets
            .iter()
            .any(|&offset| offset != 0 && offset as usize >= bytes.len())
        {
            return Err("font lookup table points outside the resource");
        }
        let mut font = Self {
            bytes,
            offsets,
            nls,
            em_size: 28,
            baseline: 24,
        };
        if let Some((em_size, baseline)) = font.detect_cell_metrics() {
            font.em_size = em_size;
            font.baseline = baseline;
        }
        Ok(font)
    }

    /// Most PAL font records share a fullwidth cell advance. Infer the cell
    /// size and baseline from the modal glyph metrics instead of assuming one
    /// game's 28-pixel font; other resources use larger cells.
    fn detect_cell_metrics(&self) -> Option<(u32, i32)> {
        let mut counts = BTreeMap::<(u32, i32), usize>::new();
        for &offset in &self.offsets {
            let offset = offset as usize;
            if offset == 0 {
                continue;
            }
            let Some(end) = offset.checked_add(24) else {
                continue;
            };
            let Some(header) = self.bytes.get(offset..end) else {
                continue;
            };
            let word = |index: usize| {
                u32::from_le_bytes(header[index * 4..index * 4 + 4].try_into().unwrap())
            };
            let (width, height, bearing_y, advance, bitmap_len) =
                (word(0), word(1), word(3) as i32, word(4), word(5) as usize);
            if width == 0
                || height == 0
                || advance == 0
                || advance > 128
                || width > advance
                || height > advance
            {
                continue;
            }
            let stride = (width + 3) & !3;
            if bitmap_len != stride as usize * height as usize
                || end
                    .checked_add(bitmap_len)
                    .is_none_or(|end| end > self.bytes.len())
            {
                continue;
            }
            let baseline = bearing_y + ((advance - height) / 2) as i32;
            if !(0..=advance as i32).contains(&baseline) {
                continue;
            }
            *counts.entry((advance, baseline)).or_default() += 1;
        }
        counts
            .into_iter()
            .filter(|(_, count)| *count >= 8)
            .max_by_key(|(_, count)| *count)
            .map(|(metrics, _)| metrics)
    }

    fn slot_for_char(&self, ch: char) -> Option<usize> {
        let encoded = self.nls.encode(&ch.to_string()).ok()?;
        match encoded.as_slice() {
            [byte] => Some(usize::from(*byte)),
            [lead, trail] if *lead >= 0x80 => Some(
                usize::from(*lead - 0x80)
                    .saturating_mul(255)
                    .saturating_add(usize::from(*trail)),
            ),
            _ => None,
        }
    }

    fn glyph(&self, ch: char) -> Option<PalBitmapGlyph<'_>> {
        let slot = self.slot_for_char(ch)?;
        let offset = *self.offsets.get(slot)? as usize;
        if offset == 0 || offset.checked_add(24)? > self.bytes.len() {
            return None;
        }
        let field = |index: usize| {
            u32::from_le_bytes(
                self.bytes[offset + index * 4..offset + index * 4 + 4]
                    .try_into()
                    .unwrap(),
            )
        };
        let width = field(0);
        let height = field(1);
        let bearing_x = field(2) as i32;
        let bearing_y = field(3) as i32;
        let advance = field(4);
        let bitmap_len = field(5) as usize;
        let start = offset + 24;
        let end = start.checked_add(bitmap_len)?;
        let alpha = self.bytes.get(start..end)?;
        let stride = (width + 3) & !3;
        if bitmap_len != 0 && bitmap_len != stride as usize * height as usize {
            return None;
        }
        Some(PalBitmapGlyph {
            width,
            height,
            bearing_x,
            bearing_y,
            advance,
            stride,
            alpha,
        })
    }

    fn supports(&self, text: &str) -> bool {
        text.chars().all(|ch| self.glyph(ch).is_some())
    }

    fn measure_line(&self, text: &str, px_height: f32) -> (u32, u32) {
        let scale = px_height / self.em_size as f32;
        let width = text
            .chars()
            .map(|ch| self.glyph(ch).map_or(self.em_size, |glyph| glyph.advance) as f32 * scale)
            .sum::<f32>()
            .ceil() as u32;
        (width.max(1), px_height.ceil().max(1.0) as u32)
    }

    fn rasterize_line(
        &self,
        text: &str,
        px_height: f32,
        color_bgra: [u8; 4],
    ) -> (u32, u32, Vec<u8>) {
        let (width, height) = self.measure_line(text, px_height);
        let scale = px_height / self.em_size as f32;
        let mut pixels = vec![0_u8; width as usize * height as usize * 4];
        let mut cursor_x = 0.0_f32;
        for ch in text.chars() {
            let Some(glyph) = self.glyph(ch) else {
                cursor_x += self.em_size as f32 * scale;
                continue;
            };
            if glyph.alpha.is_empty() {
                cursor_x += glyph.advance as f32 * scale;
                continue;
            }
            let left = cursor_x + glyph.bearing_x as f32 * scale;
            let top = (self.baseline - glyph.bearing_y) as f32 * scale;
            let draw_width = (glyph.width as f32 * scale).ceil().max(1.0) as u32;
            let draw_height = (glyph.height as f32 * scale).ceil().max(1.0) as u32;
            for dy in 0..draw_height {
                let sy = ((dy as f32 / scale).floor() as u32).min(glyph.height.saturating_sub(1));
                let py = top.floor() as i32 + dy as i32;
                if py < 0 || py >= height as i32 {
                    continue;
                }
                for dx in 0..draw_width {
                    let sx =
                        ((dx as f32 / scale).floor() as u32).min(glyph.width.saturating_sub(1));
                    let px = left.floor() as i32 + dx as i32;
                    if px < 0 || px >= width as i32 {
                        continue;
                    }
                    let coverage = glyph.alpha[(sy * glyph.stride + sx) as usize].min(64);
                    if coverage == 0 {
                        continue;
                    }
                    let alpha = ((u16::from(coverage) * u16::from(color_bgra[3]) + 32) / 64) as u8;
                    let index = (py as usize * width as usize + px as usize) * 4;
                    pixels[index] = color_bgra[0];
                    pixels[index + 1] = color_bgra[1];
                    pixels[index + 2] = color_bgra[2];
                    pixels[index + 3] = pixels[index + 3].max(alpha);
                }
            }
            cursor_x += glyph.advance as f32 * scale;
        }
        (width, height, pixels)
    }
}

#[derive(Clone, Debug)]
pub struct PalFontFallback {
    font: FontRef<'static>,
}

impl PalFontFallback {
    /// Construct using the embedded default.ttf.  Panics only if the embedded file
    /// is corrupt (which would be a build-time error, not a runtime condition).
    pub fn default_ttf() -> Self {
        for path in SYSTEM_CJK_FONT_CANDIDATES {
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            let leaked: &'static [u8] = Box::leak(bytes.into_boxed_slice());
            if let Ok(font) = FontRef::try_from_slice(leaked) {
                return Self { font };
            }
        }
        let font = FontRef::try_from_slice(DEFAULT_TTF_BYTES)
            .expect("default.ttf embedded in pal-vm is not a valid TrueType font");
        Self { font }
    }

    /// Measure the pixel width of a single text line at the given pixel height.
    /// Returns `(width_px, height_px)`.
    pub fn measure_line(&self, text: &str, px_height: f32) -> (u32, u32) {
        let scale = PxScale::from(px_height);
        let scaled = self.font.as_scaled(scale);
        let width: f32 = text
            .chars()
            .map(|c| scaled.h_advance(self.font.glyph_id(c)))
            .sum();
        (width.ceil() as u32, px_height.ceil() as u32)
    }

    /// Rasterize a single line of text into BGRA8 pixels.
    ///
    /// Returns `(width, height, pixels_bgra)`.  If the text is empty or all
    /// glyphs have zero advance, returns a 1×height blank buffer.
    ///
    /// Color is `[B, G, R, A]` matching the PAL surface format.
    pub fn rasterize_line(
        &self,
        text: &str,
        px_height: f32,
        color_bgra: [u8; 4],
    ) -> (u32, u32, Vec<u8>) {
        let scale = PxScale::from(px_height);
        let scaled = self.font.as_scaled(scale);

        let (width, height) = self.measure_line(text, px_height);
        let w = width.max(1) as usize;
        let h = height.max(1) as usize;
        let mut pixels = vec![0u8; w * h * 4];

        let mut cursor_x = 0.0f32;
        let baseline_y = scaled.ascent();

        for ch in text.chars() {
            let glyph_id = self.font.glyph_id(ch);
            let glyph =
                glyph_id.with_scale_and_position(scale, ab_glyph::point(cursor_x, baseline_y));
            cursor_x += scaled.h_advance(glyph_id);

            if let Some(outlined) = self.font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                // Some fallback fonts place shorter glyphs a pixel or two
                // above the bottom of the ideographic cell. PAL text uses a
                // shared cell baseline, so align substantial short glyphs to
                // that baseline while leaving centered marks and low commas
                // at their font-defined positions.
                let y_shift = glyph_baseline_shift(
                    bounds.min.y as i32,
                    bounds.max.y as i32,
                    baseline_y,
                    px_height,
                );
                outlined.draw(|gx, gy, cov| {
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32 + y_shift;
                    if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                        return;
                    }
                    let idx = (py as usize * w + px as usize) * 4;
                    let alpha = (cov * color_bgra[3] as f32) as u8;
                    pixels[idx] = color_bgra[0];
                    pixels[idx + 1] = color_bgra[1];
                    pixels[idx + 2] = color_bgra[2];
                    pixels[idx + 3] = alpha;
                });
            }
        }

        (w as u32, h as u32, pixels)
    }
}

fn glyph_baseline_shift(min_y: i32, max_y: i32, ascent: f32, cell_height: f32) -> i32 {
    if (max_y - min_y) as f32 <= cell_height * 0.45 {
        return 0;
    }
    let baseline_bottom = ascent.ceil() as i32 - 1;
    (baseline_bottom - (max_y - 1)).clamp(0, (cell_height * 0.1).ceil() as i32)
}

const SYSTEM_CJK_FONT_CANDIDATES: &[&str] = &[
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
    "/Library/Fonts/Arial Unicode.ttf",
    "/System/Library/Fonts/ヒラギノ角ゴシック W4.ttc",
    "/System/Library/Fonts/Hiragino Sans GB.ttc",
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    "C:/Windows/Fonts/msyh.ttc",
    "C:/Windows/Fonts/msgothic.ttc",
];

fn argb_to_bgra(color: u32) -> [u8; 4] {
    [
        (color & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        ((color >> 16) & 0xFF) as u8,
        ((color >> 24) & 0xFF) as u8,
    ]
}

fn argb_to_rgba(color: u32) -> [u8; 4] {
    [
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
        ((color >> 24) & 0xFF) as u8,
    ]
}

fn apply_text_edge(src: &[u8], width: u32, height: u32, edge: [u8; 4]) -> (u32, u32, Vec<u8>) {
    let pad = 1u32;
    let out_w = width.saturating_add(pad * 2).max(1);
    let out_h = height.saturating_add(pad * 2).max(1);
    let mut dst = vec![0u8; out_w as usize * out_h as usize * 4];
    let stamp = |dst: &mut [u8], x: i32, y: i32, px: &[u8]| {
        if x < 0 || y < 0 || x >= out_w as i32 || y >= out_h as i32 || px.len() < 4 || px[3] == 0 {
            return;
        }
        let index = (y as usize * out_w as usize + x as usize) * 4;
        dst[index..index + 4].copy_from_slice(&px[..4]);
    };
    for y in 0..height {
        for x in 0..width {
            let src_index = (y as usize * width as usize + x as usize) * 4;
            let source_alpha = src.get(src_index + 3).copied().unwrap_or(0);
            if source_alpha == 0 {
                continue;
            }
            let ox = x as i32 + pad as i32;
            let oy = y as i32 + pad as i32;
            let mut edge_pixel = edge;
            edge_pixel[3] = ((u16::from(edge[3]) * u16::from(source_alpha) + 127) / 255) as u8;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let index = ((oy + dy) as usize * out_w as usize + (ox + dx) as usize) * 4;
                    if ox + dx >= 0
                        && oy + dy >= 0
                        && ox + dx < out_w as i32
                        && oy + dy < out_h as i32
                        && dst[index + 3] < edge_pixel[3]
                    {
                        dst[index..index + 4].copy_from_slice(&edge_pixel);
                    }
                }
            }
        }
    }
    for y in 0..height {
        for x in 0..width {
            let src_index = (y as usize * width as usize + x as usize) * 4;
            let Some(px) = src.get(src_index..src_index + 4) else {
                continue;
            };
            stamp(&mut dst, x as i32 + pad as i32, y as i32 + pad as i32, px);
        }
    }
    (out_w, out_h, dst)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitmap_font_fixture() -> Vec<u8> {
        let table_end = 0x108_u32;
        let mut bytes = vec![0_u8; table_end as usize];
        bytes[0x80..0x84].copy_from_slice(&table_end.to_le_bytes());
        bytes[0x104..0x108].copy_from_slice(&table_end.to_le_bytes());
        for value in [2_u32, 2, 1, 2, 4, 8] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&[64, 0, 0, 0, 0, 32, 0, 0]);
        bytes
    }

    fn fullwidth_bitmap_font_fixture(
        width: u32,
        height: u32,
        bearing_y: i32,
        advance: u32,
    ) -> Vec<u8> {
        // Shift-JIS "あ" occupies PAL slot (0x82 - 0x80) * 255 + 0xA0.
        let kana_slot = 2 * 255 + 0xA0;
        let table_end = ((kana_slot + 1) * 4) as u32;
        let mut bytes = vec![0_u8; table_end as usize];
        for slot in 0x20..0x30 {
            bytes[slot * 4..slot * 4 + 4].copy_from_slice(&table_end.to_le_bytes());
        }
        bytes[kana_slot * 4..kana_slot * 4 + 4].copy_from_slice(&table_end.to_le_bytes());
        let stride = (width + 3) & !3;
        for value in [width, height, 0, bearing_y as u32, advance, stride * height] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend(std::iter::repeat_n(64, (stride * height) as usize));
        bytes
    }

    #[test]
    fn short_glyphs_share_the_cell_baseline_without_moving_centered_marks() {
        assert_eq!(glyph_baseline_shift(7, 23, 24.0, 28.0), 1);
        assert_eq!(glyph_baseline_shift(7, 22, 24.0, 28.0), 2);
        assert_eq!(glyph_baseline_shift(12, 16, 24.0, 28.0), 0);
        assert_eq!(glyph_baseline_shift(18, 25, 24.0, 28.0), 0);
    }

    #[test]
    fn default_ttf_loads() {
        let font = PalFontFallback::default_ttf();
        let (w, h) = font.measure_line("A", 16.0);
        assert!(w > 0, "glyph width must be positive");
        assert!(h > 0, "glyph height must be positive");
    }

    #[test]
    fn rasterize_produces_correct_dimensions() {
        let font = PalFontFallback::default_ttf();
        let (w, h, pixels) = font.rasterize_line("Hi", 20.0, [255, 255, 255, 255]);
        assert_eq!(pixels.len(), w as usize * h as usize * 4);
        assert!(w > 0);
        assert!(h > 0);
    }

    #[test]
    fn effect_adds_edge_pixels_around_the_glyph() {
        let mut font = PalFontSystem::new();
        font.set_font_size(28);
        font.set_color(0xFFFF_0000, 0xFF00_0000);
        font.set_effect(0);
        let (_, _, plain) = font.rasterize("A");
        font.set_effect(1);
        let (_, _, edged) = font.rasterize("A");
        let ink = |pixels: &[u8]| pixels.chunks_exact(4).filter(|px| px[3] > 0).count();
        assert!(ink(&edged) > ink(&plain));
        assert!(edged.chunks_exact(4).any(|px| px[0] == 255 && px[3] > 0));
    }

    #[test]
    fn bitmap_font_uses_pal_metrics_and_grayscale_coverage() {
        let mut font = PalFontSystem::new();
        font.load_bitmap_font(bitmap_font_fixture(), Nls::ShiftJis)
            .unwrap();
        font.set_type(4);
        font.set_font_size(28);
        font.set_effect(0);
        let (width, height, pixels) = font.rasterize("A");
        assert_eq!((width, height), (4, 28));
        let alpha = |x: usize, y: usize| pixels[(y * width as usize + x) * 4 + 3];
        assert_eq!(alpha(1, 22), 255);
        assert_eq!(alpha(2, 23), 128);
    }

    #[test]
    fn bitmap_font_uses_its_own_cell_size_without_clipping_large_glyphs() {
        let font =
            PalBitmapFont::parse(fullwidth_bitmap_font_fixture(36, 36, 31, 36), Nls::ShiftJis)
                .unwrap();
        assert_eq!((font.em_size, font.baseline), (36, 31));
        let (width, height, pixels) = font.rasterize_line("あ", 26.0, [255; 4]);
        assert_eq!((width, height), (26, 26));
        assert_eq!(pixels[3], 255, "top row was clipped");
        assert_eq!(
            pixels[((height - 1) * width * 4 + 3) as usize],
            255,
            "bottom row was clipped"
        );

        let font =
            PalBitmapFont::parse(fullwidth_bitmap_font_fixture(26, 26, 23, 28), Nls::ShiftJis)
                .unwrap();
        assert_eq!((font.em_size, font.baseline), (28, 24));
    }

    #[test]
    fn rasterize_empty_string_returns_blank() {
        let font = PalFontFallback::default_ttf();
        let (w, h, pixels) = font.rasterize_line("", 16.0, [0, 0, 0, 255]);
        assert_eq!(pixels.len(), w as usize * h as usize * 4);
        assert!(h > 0);
    }
}
