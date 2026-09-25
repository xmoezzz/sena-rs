use std::collections::BTreeMap;
use std::io::Cursor;

use plmpeg::MpegDecoder;
use wmv_decoder::{AsfWmv2Decoder, YuvFrame};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MSpriteHandle(pub u32);

#[derive(Debug)]
pub struct MSpriteSystem {
    next_handle: u32,
    entries: BTreeMap<MSpriteHandle, MSpriteEntry>,
    movie: Option<MoviePlayback>,
}

impl Default for MSpriteSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MSpriteSystem {
    pub fn new() -> Self {
        Self {
            next_handle: 1,
            entries: BTreeMap::new(),
            movie: None,
        }
    }

    pub fn load_wmv(
        &mut self,
        name: impl Into<String>,
        bytes: Vec<u8>,
    ) -> anyhow::Result<LoadedMSprite> {
        let decoder = AsfWmv2Decoder::open(Cursor::new(bytes.clone()))?;
        let info = decoder.video_stream_info().clone();
        let mut source = FrameSource::Wmv(decoder);
        let first = next_presented(&mut source)?;
        let (width, height, rgba, pts_ms) = match first {
            Some(frame) => (frame.width, frame.height, frame.rgba, frame.pts_ms),
            None => {
                let rgba = vec![0; info.width.max(1) as usize * info.height.max(1) as usize * 4];
                (info.width.max(1), info.height.max(1), rgba, 0)
            }
        };
        Ok(self.insert_loaded(name.into(), bytes, source, width, height, rgba, pts_ms))
    }

    pub fn load_movie(
        &mut self,
        name: impl Into<String>,
        bytes: Vec<u8>,
    ) -> anyhow::Result<LoadedMSprite> {
        match movie_container(&bytes) {
            MovieContainer::Mpeg => self.load_mpeg(name, bytes),
            MovieContainer::Mp4 => anyhow::bail!("mp4 movie is not decoded"),
            MovieContainer::Wmv | MovieContainer::Unknown => self.load_wmv(name, bytes),
        }
    }

    fn load_mpeg(
        &mut self,
        name: impl Into<String>,
        bytes: Vec<u8>,
    ) -> anyhow::Result<LoadedMSprite> {
        let mut decoder = MpegDecoder::open(bytes).map_err(|err| anyhow::anyhow!(err))?;
        let Some(frame) = decoder.next_frame().map_err(|err| anyhow::anyhow!(err))? else {
            anyhow::bail!("mpeg stream produced no video frame");
        };
        Ok(self.insert_loaded(
            name.into(),
            Vec::new(),
            FrameSource::Mpeg(decoder),
            frame.width,
            frame.height,
            frame.rgba,
            frame.pts_ms,
        ))
    }

    fn insert_loaded(
        &mut self,
        name: String,
        bytes: Vec<u8>,
        decoder: FrameSource,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        pts_ms: u32,
    ) -> LoadedMSprite {
        let handle = self.allocate_handle();
        self.entries.insert(
            handle,
            MSpriteEntry {
                name: name.clone(),
                bytes,
                decoder,
                width,
                height,
                current_rgba: rgba.clone(),
                current_pts_ms: pts_ms,
                playing: false,
                locked: false,
                loop_mode: 0,
                loop_start: 0,
                loop_end: 0,
                finished: false,
                state_bits: 0,
            },
        );
        LoadedMSprite {
            handle,
            width,
            height,
            rgba,
            name,
        }
    }

    pub fn release(&mut self, handle: MSpriteHandle) -> bool {
        self.entries.remove(&handle).is_some()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn check(&self, handle: MSpriteHandle) -> bool {
        self.entries.contains_key(&handle)
    }

    pub fn play(&mut self, handle: MSpriteHandle, loop_mode: i32) -> bool {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return false;
        };
        entry.playing = true;
        entry.finished = false;
        entry.loop_mode = loop_mode;
        entry.state_bits &= !MSPRITE_STATE_FINISHED;
        true
    }

    pub fn stop(&mut self, handle: MSpriteHandle) -> bool {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return false;
        };
        entry.playing = false;
        entry.finished = true;
        entry.state_bits |= MSPRITE_STATE_FINISHED;
        true
    }

    pub fn pause(&mut self, handle: MSpriteHandle) -> bool {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return false;
        };
        entry.playing = false;
        true
    }

    pub fn lock(&mut self, handle: MSpriteHandle) -> bool {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return false;
        };
        entry.locked = true;
        true
    }

    pub fn unlock(&mut self, handle: MSpriteHandle) -> bool {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return false;
        };
        entry.locked = false;
        true
    }

    pub fn set_loop(&mut self, handle: MSpriteHandle, loop_mode: i32) -> bool {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return false;
        };
        entry.loop_mode = loop_mode;
        true
    }

    pub fn set_loop_point(&mut self, handle: MSpriteHandle, start: i32, end: i32) -> bool {
        let Some(entry) = self.entries.get_mut(&handle) else {
            return false;
        };
        entry.loop_start = start.max(0);
        entry.loop_end = end.max(0);
        true
    }

    pub fn is_loop(&self, handle: MSpriteHandle) -> bool {
        self.entries
            .get(&handle)
            .is_some_and(|entry| entry.loop_mode != 0 || entry.loop_end > entry.loop_start)
    }

    pub fn state(&self, handle: MSpriteHandle) -> u32 {
        let Some(entry) = self.entries.get(&handle) else {
            return 0;
        };
        let mut state = entry.state_bits;
        if entry.playing {
            state |= MSPRITE_STATE_PLAYING;
        }
        if entry.finished {
            state |= MSPRITE_STATE_FINISHED;
        }
        if entry.locked {
            state |= MSPRITE_STATE_LOCKED;
        }
        if self.is_loop(handle) {
            state |= MSPRITE_STATE_LOOP;
        }
        state
    }

    pub fn advance(&mut self, delta_ms: u32) -> Vec<MSpriteFrameUpdate> {
        let mut updates = Vec::new();
        let handles = self.entries.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            let Some(entry) = self.entries.get_mut(&handle) else {
                continue;
            };
            if !entry.playing || entry.locked || entry.finished {
                continue;
            }
            let target_pts = entry.current_pts_ms.saturating_add(delta_ms);
            let mut latest = None;
            let mut restarts = 0u32;
            let mut skipped = 0u32;
            loop {
                let pulled = match next_presented(&mut entry.decoder) {
                    Ok(frame) => frame,
                    Err(err) => {
                        log::warn!("[trace-msprite] decode {:?} failed: {err}", entry.name);
                        finish_entry(entry);
                        break;
                    }
                };
                let Some(frame) = pulled else {
                    if !restart_playback(entry, &mut restarts, &mut latest, handle, target_pts) {
                        break;
                    }
                    continue;
                };
                if frame_inside_preroll(frame.pts_ms, entry.loop_start) {
                    skipped += 1;
                    if skipped > 4_000 {
                        finish_entry(entry);
                        break;
                    }
                    continue;
                }
                match movie_loop_at_frame(
                    frame.pts_ms,
                    entry.loop_mode,
                    entry.loop_start,
                    entry.loop_end,
                ) {
                    MovieLoopAction::Present => {
                        let reached = frame.pts_ms >= target_pts;
                        apply_presented(entry, handle, frame, &mut latest);
                        if reached {
                            break;
                        }
                    }
                    MovieLoopAction::Finish => {
                        finish_entry(entry);
                        break;
                    }
                    MovieLoopAction::Restart => {
                        if !restart_playback(entry, &mut restarts, &mut latest, handle, target_pts)
                        {
                            break;
                        }
                    }
                }
            }
            if let Some(update) = latest {
                updates.push(update);
            }
        }
        if let Some(movie) = self.movie.as_mut() {
            if let Some(handle) = movie.handle {
                if let Some(entry) = self.entries.get(&handle) {
                    movie.playing = entry.playing && !entry.finished;
                    movie.elapsed_ms = entry.current_pts_ms;
                } else {
                    movie.playing = false;
                }
            }
        }
        updates
    }

    pub fn start_movie(&mut self, name: impl Into<String>, layer: i32, handle: MSpriteHandle) {
        self.movie = Some(MoviePlayback {
            name: name.into(),
            layer,
            playing: true,
            elapsed_ms: 0,
            handle: Some(handle),
        });
    }

    pub fn stop_movie(&mut self) {
        self.movie = None;
    }

    pub fn is_movie(&self) -> bool {
        self.movie.as_ref().is_some_and(|movie| movie.playing)
    }

    pub fn movie(&self) -> Option<&MoviePlayback> {
        self.movie.as_ref()
    }

    fn allocate_handle(&mut self) -> MSpriteHandle {
        let handle = MSpriteHandle(self.next_handle);
        self.next_handle = self
            .next_handle
            .checked_add(1)
            .expect("MSprite handle space exhausted");
        handle
    }
}

#[derive(Clone, Debug)]
pub struct LoadedMSprite {
    pub handle: MSpriteHandle,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct MSpriteFrameUpdate {
    pub handle: MSpriteHandle,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub source_name: String,
}

#[derive(Clone, Debug)]
pub struct MoviePlayback {
    pub name: String,
    pub layer: i32,
    pub playing: bool,
    pub elapsed_ms: u32,
    pub handle: Option<MSpriteHandle>,
}

struct MSpriteEntry {
    name: String,
    bytes: Vec<u8>,
    decoder: FrameSource,
    width: u32,
    height: u32,
    current_rgba: Vec<u8>,
    current_pts_ms: u32,
    playing: bool,
    locked: bool,
    loop_mode: i32,
    loop_start: i32,
    loop_end: i32,
    finished: bool,
    state_bits: u32,
}

impl std::fmt::Debug for MSpriteEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MSpriteEntry")
            .field("name", &self.name)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("current_pts_ms", &self.current_pts_ms)
            .field("playing", &self.playing)
            .field("locked", &self.locked)
            .field("loop_mode", &self.loop_mode)
            .field("loop_start", &self.loop_start)
            .field("loop_end", &self.loop_end)
            .field("finished", &self.finished)
            .field("state_bits", &self.state_bits)
            .finish_non_exhaustive()
    }
}

enum FrameSource {
    Wmv(AsfWmv2Decoder<Cursor<Vec<u8>>>),
    Mpeg(MpegDecoder),
}

struct PresentedFrame {
    pts_ms: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MovieLoopAction {
    Present,
    Restart,
    Finish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MovieContainer {
    Mpeg,
    Wmv,
    Mp4,
    Unknown,
}

fn movie_container(bytes: &[u8]) -> MovieContainer {
    if plmpeg::is_mpeg_packet(bytes) {
        MovieContainer::Mpeg
    } else if bytes.len() >= 4 && bytes[..4] == [0x30, 0x26, 0xB2, 0x75] {
        MovieContainer::Wmv
    } else if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
        MovieContainer::Mp4
    } else {
        MovieContainer::Unknown
    }
}

fn movie_loop_at_frame(
    pts_ms: u32,
    loop_mode: i32,
    loop_start: i32,
    loop_end: i32,
) -> MovieLoopAction {
    let start = u32::try_from(loop_start).unwrap_or(0);
    let end = u32::try_from(loop_end).unwrap_or(0);
    if end > start && pts_ms >= end {
        if loop_mode != 0 {
            MovieLoopAction::Restart
        } else {
            MovieLoopAction::Finish
        }
    } else {
        MovieLoopAction::Present
    }
}

fn frame_inside_preroll(pts_ms: u32, loop_start: i32) -> bool {
    let start = u32::try_from(loop_start).unwrap_or(0);
    start > 0 && pts_ms < start
}

fn finish_entry(entry: &mut MSpriteEntry) {
    entry.playing = false;
    entry.finished = true;
    entry.state_bits |= MSPRITE_STATE_FINISHED;
}

fn apply_presented(
    entry: &mut MSpriteEntry,
    handle: MSpriteHandle,
    frame: PresentedFrame,
    latest: &mut Option<MSpriteFrameUpdate>,
) {
    entry.width = frame.width;
    entry.height = frame.height;
    entry.current_pts_ms = frame.pts_ms;
    entry.current_rgba = frame.rgba.clone();
    *latest = Some(MSpriteFrameUpdate {
        handle,
        width: frame.width,
        height: frame.height,
        rgba: frame.rgba,
        source_name: entry.name.clone(),
    });
}

fn restart_playback(
    entry: &mut MSpriteEntry,
    restarts: &mut u32,
    latest: &mut Option<MSpriteFrameUpdate>,
    handle: MSpriteHandle,
    target_pts: u32,
) -> bool {
    if entry.loop_mode == 0 {
        finish_entry(entry);
        return false;
    }
    *restarts += 1;
    if *restarts > 32 {
        finish_entry(entry);
        return false;
    }
    let restarted = restart_source(&mut entry.decoder, &entry.bytes, entry.loop_start);
    match restarted {
        Ok(Some(frame)) => {
            if movie_loop_at_frame(
                frame.pts_ms,
                entry.loop_mode,
                entry.loop_start,
                entry.loop_end,
            ) == MovieLoopAction::Finish
            {
                finish_entry(entry);
                return false;
            }
            let reached = frame.pts_ms >= target_pts;
            apply_presented(entry, handle, frame, latest);
            !reached
        }
        Ok(None) => true,
        Err(err) => {
            log::warn!("[trace-msprite] restart {:?} failed: {err}", entry.name);
            finish_entry(entry);
            false
        }
    }
}

fn next_presented(source: &mut FrameSource) -> anyhow::Result<Option<PresentedFrame>> {
    match source {
        FrameSource::Wmv(decoder) => match decoder.next_frame()? {
            Some(frame) => Ok(Some(PresentedFrame {
                pts_ms: frame.pts_ms,
                width: frame.frame.width,
                height: frame.frame.height,
                rgba: yuv420_to_rgba(&frame.frame),
            })),
            None => Ok(None),
        },
        FrameSource::Mpeg(decoder) => {
            match decoder.next_frame().map_err(|err| anyhow::anyhow!(err))? {
                Some(frame) => Ok(Some(PresentedFrame {
                    pts_ms: frame.pts_ms,
                    width: frame.width,
                    height: frame.height,
                    rgba: frame.rgba,
                })),
                None => Ok(None),
            }
        }
    }
}

fn restart_source(
    source: &mut FrameSource,
    bytes: &[u8],
    loop_start: i32,
) -> anyhow::Result<Option<PresentedFrame>> {
    let start = u32::try_from(loop_start).unwrap_or(0);
    match source {
        FrameSource::Wmv(decoder) => {
            *decoder = AsfWmv2Decoder::open(Cursor::new(bytes.to_vec()))?;
            if start == 0 {
                return Ok(None);
            }
            let mut skipped = 0u32;
            loop {
                match decoder.next_frame()? {
                    Some(frame) if frame.pts_ms >= start => {
                        return Ok(Some(PresentedFrame {
                            pts_ms: frame.pts_ms,
                            width: frame.frame.width,
                            height: frame.frame.height,
                            rgba: yuv420_to_rgba(&frame.frame),
                        }));
                    }
                    Some(_) => {
                        skipped += 1;
                        if skipped > 4_000 {
                            return Ok(None);
                        }
                    }
                    None => return Ok(None),
                }
            }
        }
        FrameSource::Mpeg(decoder) => {
            if start == 0 {
                decoder.rewind();
                Ok(None)
            } else {
                match decoder.seek_ms(start) {
                    Ok(Some(frame)) => Ok(Some(PresentedFrame {
                        pts_ms: frame.pts_ms,
                        width: frame.width,
                        height: frame.height,
                        rgba: frame.rgba,
                    })),
                    Ok(None) => Ok(None),
                    Err(err) => Err(anyhow::anyhow!(err)),
                }
            }
        }
    }
}

pub const MSPRITE_STATE_PLAYING: u32 = 0x0000_0001;
pub const MSPRITE_STATE_LOCKED: u32 = 0x0000_0002;
pub const MSPRITE_STATE_FINISHED: u32 = 0x0000_0004;
pub const MSPRITE_STATE_LOOP: u32 = 0x8000_0000;

fn yuv420_to_rgba(frame: &YuvFrame) -> Vec<u8> {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let chroma_width = (width / 2).max(1);
    let mut rgba = vec![0u8; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let yy = frame.y[y * width + x] as i32;
            let uv_idx = (y / 2) * chroma_width + (x / 2);
            let cb = frame.cb.get(uv_idx).copied().unwrap_or(128) as i32;
            let cr = frame.cr.get(uv_idx).copied().unwrap_or(128) as i32;
            let c = yy - 16;
            let d = cb - 128;
            let e = cr - 128;
            let r = ((298 * c + 409 * e + 128) >> 8).clamp(0, 255) as u8;
            let g = ((298 * c - 100 * d - 208 * e + 128) >> 8).clamp(0, 255) as u8;
            let b = ((298 * c + 516 * d + 128) >> 8).clamp(0, 255) as u8;
            let out = (y * width + x) * 4;
            rgba[out] = r;
            rgba[out + 1] = g;
            rgba[out + 2] = b;
            rgba[out + 3] = 255;
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuv420_black_frame_converts_to_opaque_rgba() {
        let frame = YuvFrame::new(2, 2);
        let rgba = yuv420_to_rgba(&frame);
        assert_eq!(rgba.len(), 16);
        assert!(rgba.chunks_exact(4).all(|px| px[3] == 255));
        assert!(rgba
            .chunks_exact(4)
            .all(|px| px[0] <= 1 && px[1] <= 1 && px[2] <= 1));
    }

    #[test]
    fn loop_end_restarts_when_looping_and_finishes_when_not() {
        assert_eq!(
            movie_loop_at_frame(1_500, 1, 0, 1_000),
            MovieLoopAction::Restart
        );
        assert_eq!(
            movie_loop_at_frame(1_500, 0, 0, 1_000),
            MovieLoopAction::Finish
        );
        assert_eq!(
            movie_loop_at_frame(400, 1, 0, 1_000),
            MovieLoopAction::Present
        );
        assert!(frame_inside_preroll(200, 500));
        assert!(!frame_inside_preroll(500, 500));
        assert!(!frame_inside_preroll(10, 0));
    }

    #[test]
    fn movie_container_detects_mpeg_wmv_and_mp4() {
        assert_eq!(
            movie_container(&[0x00, 0x00, 0x01, 0xBA, 0x21]),
            MovieContainer::Mpeg
        );
        assert_eq!(
            movie_container(&[0x30, 0x26, 0xB2, 0x75]),
            MovieContainer::Wmv
        );
        let mut mp4 = vec![0, 0, 0, 0x18];
        mp4.extend_from_slice(b"ftyp");
        assert_eq!(movie_container(&mp4), MovieContainer::Mp4);
    }
}
