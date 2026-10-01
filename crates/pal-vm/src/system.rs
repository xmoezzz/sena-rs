use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PalRandomState {
    seeds: Vec<i32>,
    cursor: usize,
    lcg_state: u32,
}

impl Default for PalRandomState {
    fn default() -> Self {
        Self::new(256)
    }
}

impl PalRandomState {
    pub fn new(seed_count: usize) -> Self {
        let seed_count = seed_count.max(1);
        let mut state = Self {
            seeds: vec![0; seed_count],
            cursor: 0,
            lcg_state: 0x1234_ABCD,
        };
        state.reset();
        state
    }

    /// PalRandomEx always reads the current seed entry and advances modulo seed_count.
    pub fn random_ex(&mut self) -> i32 {
        let value = self.seeds[self.cursor];
        self.cursor += 1;
        if self.cursor == self.seeds.len() {
            self.cursor = 0;
        }
        value
    }

    /// PalRandom uses the same seed ring when debug mode is active; otherwise PAL calls rand().
    pub fn random(&mut self, debug_mode: bool) -> i32 {
        if debug_mode {
            self.random_ex()
        } else {
            self.next_lcg() as i32
        }
    }

    /// Fill the PAL seed ring. Mirrors PalRandomSetSeed_0's truncated copy and reset cursor.
    /// The original returns copied_count - 1.
    pub fn set_seed(&mut self, values: &[i32]) -> i32 {
        self.cursor = 0;
        let count = values.len().min(self.seeds.len());
        self.seeds[..count].copy_from_slice(&values[..count]);
        count as i32 - 1
    }

    /// Copy the PAL seed ring out. Mirrors PalRandomGetSeed_0's truncated copy count.
    pub fn get_seed(&self, out: &mut [i32]) -> usize {
        let count = out.len().min(self.seeds.len());
        out[..count].copy_from_slice(&self.seeds[..count]);
        count
    }

    /// PalRandomReset_0 resets cursor and refills the seed ring with rand() output.
    /// This uses a deterministic LCG in place of CRT rand() so tests and PAL_DEBUG are stable.
    pub fn reset(&mut self) {
        self.cursor = 0;
        for index in 0..self.seeds.len() {
            self.seeds[index] = self.next_lcg() as i32;
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn seed_count(&self) -> usize {
        self.seeds.len()
    }

    fn next_lcg(&mut self) -> u32 {
        self.lcg_state = self
            .lcg_state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        self.lcg_state
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PalSystemState {
    language: i32,
    /// Game.exe task data +8, written by window_change_mode after posting
    /// PalWindowChangeMode.
    window_mode: i32,
    /// Game.exe task data +12, written by window_set_mode_cache and returned
    /// by window_get_mode_cache.  This is intentionally separate from the
    /// live PAL window mode; system scripts use it as pending/config state.
    window_mode_cache: i32,
    auto_speed_percent: i32,
    /// koikake.exe task data +0x20, written by the category-2 index-9
    /// extcall (message speed set) and returned by index 10.  Units match
    /// auto_speed_percent (0..100, higher = slower per-glyph delay).
    text_speed_percent: i32,
    /// koikake.exe task data +0xC, written by the category-9 index-7
    /// extcall: 0 = skip read text only, nonzero = skip everything.  The two
    /// skip gates (skip_set refusal and the display-time cancel message) are
    /// its only native consumers.
    skip_gate: i32,
    effect_enabled: i32,
    window_change_enabled: i32,
    aspect_mode: i32,
    aspect_position_enabled: i32,
    hide_cursor_time: i32,
    cursor_null: bool,
    touch_enabled: bool,
    touch_mode: i32,
    gesture_mode: i32,
    gestures: [PalGesture; 5],
    window_pos: (i32, i32),
    window_size: (i32, i32),
    windowed_content_rect: PalRectI,
    fullscreen_content_rect: PalRectI,
    logical_size: (i32, i32),
    system_paths: PalSystemPaths,
    window_requests: Vec<PalWindowRequest>,
}

impl Default for PalSystemState {
    fn default() -> Self {
        Self {
            language: 1,
            window_mode: 0,
            window_mode_cache: 0,
            auto_speed_percent: 50,
            text_speed_percent: 50,
            skip_gate: 0,
            effect_enabled: 1,
            window_change_enabled: 1,
            aspect_mode: 0,
            aspect_position_enabled: 0,
            hide_cursor_time: 0,
            cursor_null: false,
            touch_enabled: false,
            touch_mode: 0,
            gesture_mode: 0,
            gestures: [PalGesture::default(); 5],
            window_pos: (0, 0),
            window_size: (1280, 720),
            windowed_content_rect: PalRectI::new(0, 0, 1280, 720),
            fullscreen_content_rect: PalRectI::new(0, 0, 1280, 720),
            logical_size: (1280, 720),
            system_paths: PalSystemPaths::default(),
            window_requests: Vec::new(),
        }
    }
}

impl PalSystemState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn language(&self) -> i32 {
        self.language
    }

    pub fn set_language(&mut self, language: i32) -> i32 {
        self.language = language;
        1
    }

    pub fn window_mode(&self) -> i32 {
        self.window_mode
    }

    pub fn window_mode_cache(&self) -> i32 {
        self.window_mode_cache
    }

    pub fn change_window_mode(&mut self, mode: i32) -> i32 {
        self.window_mode = mode;
        self.window_requests
            .push(PalWindowRequest::ChangeMode { mode });
        1
    }

    pub fn set_window_mode_cache(&mut self, mode: i32) -> i32 {
        self.window_mode_cache = mode;
        1
    }

    pub fn auto_speed_percent(&self) -> i32 {
        self.auto_speed_percent
    }

    pub fn set_auto_speed_percent(&mut self, speed: i32) -> i32 {
        // Native clamp is an unsigned `cmp v,100; jbe`: anything above 100
        // (including negative values) becomes 100.
        self.auto_speed_percent = if (0..=100).contains(&speed) { speed } else { 100 };
        1
    }

    pub fn text_speed_percent(&self) -> i32 {
        self.text_speed_percent
    }

    pub fn set_text_speed_percent(&mut self, speed: i32) -> i32 {
        self.text_speed_percent = if (0..=100).contains(&speed) { speed } else { 100 };
        1
    }

    pub fn skip_gate(&self) -> i32 {
        self.skip_gate
    }

    pub fn set_skip_gate(&mut self, gate: i32) -> i32 {
        self.skip_gate = gate;
        1
    }

    pub fn effect_enabled(&self) -> i32 {
        self.effect_enabled
    }

    pub fn set_effect_enabled(&mut self, enabled: i32) -> i32 {
        self.effect_enabled = if enabled == 0 { 0 } else { 1 };
        1
    }

    pub fn window_change_enabled(&self) -> i32 {
        self.window_change_enabled
    }

    pub fn set_window_change_enabled(&mut self, enabled: i32) -> i32 {
        self.window_change_enabled = enabled;
        1
    }

    pub fn aspect_mode(&self) -> i32 {
        self.aspect_mode
    }

    pub fn change_aspect_mode(&mut self, mode: i32) -> i32 {
        self.aspect_mode = mode;
        self.window_requests
            .push(PalWindowRequest::ChangeAspect { mode });
        1
    }

    pub fn set_aspect_position_enabled(&mut self, enabled: i32) -> i32 {
        self.aspect_position_enabled = enabled;
        1
    }

    pub fn aspect_position_enabled(&self) -> i32 {
        self.aspect_position_enabled
    }

    pub fn set_hide_cursor_time(&mut self, time: i32) -> i32 {
        self.hide_cursor_time = time.max(0);
        1
    }

    pub fn hide_cursor_time(&self) -> i32 {
        self.hide_cursor_time
    }

    pub fn set_cursor_null(&mut self, enabled: bool) -> i32 {
        self.cursor_null = enabled;
        1
    }

    pub fn cursor_null(&self) -> bool {
        self.cursor_null
    }

    pub fn touch_enabled(&self) -> bool {
        self.touch_enabled
    }

    pub fn set_touch_enabled(&mut self, enabled: bool) {
        self.touch_enabled = enabled;
    }

    pub fn touch_mode(&self) -> i32 {
        self.touch_mode
    }

    pub fn set_touch_mode(&mut self, mode: i32) -> i32 {
        self.touch_mode = mode;
        mode
    }

    pub fn is_touch(&self) -> bool {
        self.touch_enabled || self.touch_mode != 0
    }

    pub fn gesture_mode(&self) -> i32 {
        self.gesture_mode
    }

    pub fn set_gesture_mode(&mut self, mode: i32) {
        self.gesture_mode = mode;
    }

    pub fn set_gesture(&mut self, index: usize, gesture: PalGesture) -> bool {
        let Some(slot) = self.gestures.get_mut(index) else {
            return false;
        };
        *slot = gesture;
        true
    }

    pub fn gestures(&self) -> [PalGesture; 5] {
        self.gestures
    }

    pub fn gesture_check(&self) -> bool {
        self.gestures
            .iter()
            .any(|gesture| matches!(gesture.kind, 3..=7))
    }

    pub fn window_pos(&self) -> (i32, i32) {
        if self.window_mode != 0 {
            (
                self.fullscreen_content_rect.left,
                self.fullscreen_content_rect.top,
            )
        } else {
            self.window_pos
        }
    }

    pub fn set_window_pos(&mut self, x: i32, y: i32) -> i32 {
        self.window_pos = (x, y);
        self.window_requests.push(PalWindowRequest::SetPos { x, y });
        1
    }

    pub fn window_size(&self) -> (i32, i32) {
        if self.window_mode != 0 {
            (
                self.fullscreen_content_rect.width(),
                self.fullscreen_content_rect.height(),
            )
        } else {
            self.window_size
        }
    }

    pub fn set_window_size(&mut self, width: i32, height: i32) {
        self.window_size = (width.max(1), height.max(1));
        self.windowed_content_rect = PalRectI::new(0, 0, width.max(1), height.max(1));
        self.window_requests.push(PalWindowRequest::ChangeSize {
            width: width.max(1),
            height: height.max(1),
        });
    }

    pub fn set_logical_size(&mut self, width: i32, height: i32) {
        self.logical_size = (width.max(1), height.max(1));
    }

    pub fn logical_size(&self) -> (i32, i32) {
        self.logical_size
    }

    pub fn set_fullscreen_content_rect(&mut self, rect: PalRectI) {
        self.fullscreen_content_rect = rect.normalized_nonempty();
    }

    pub fn cursor_device_pos(&self, x: i32, y: i32) -> (i32, i32) {
        let (logical_w, logical_h) = self.logical_size;
        if self.window_mode != 0 {
            let rect = self.fullscreen_content_rect.normalized_nonempty();
            (
                rect.left + (rect.width() as f32 * (x as f32 / logical_w.max(1) as f32)) as i32,
                rect.top + (rect.height() as f32 * (y as f32 / logical_h.max(1) as f32)) as i32,
            )
        } else {
            (
                self.window_pos.0
                    + (self.window_size.0 as f32 * (x as f32 / logical_w.max(1) as f32)) as i32,
                self.window_pos.1
                    + (self.window_size.1 as f32 * (y as f32 / logical_h.max(1) as f32)) as i32,
            )
        }
    }

    pub fn active_content_rect(&self) -> PalRectI {
        if self.window_mode != 0 {
            self.fullscreen_content_rect.normalized_nonempty()
        } else {
            self.windowed_content_rect.normalized_nonempty()
        }
    }

    pub fn push_system_path(
        &mut self,
        kind: PalSystemPathKind,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> bool {
        self.system_paths.push(kind, base, path)
    }

    pub fn clear_system_paths(&mut self) -> bool {
        self.system_paths.clear()
    }

    pub fn system_paths(&self, kind: PalSystemPathKind) -> &[PathBuf] {
        self.system_paths.get(kind)
    }

    pub fn take_window_requests(&mut self) -> Vec<PalWindowRequest> {
        std::mem::take(&mut self.window_requests)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PalSystemPaths {
    normal: Vec<PathBuf>,
    cn: Vec<PathBuf>,
    tc: Vec<PathBuf>,
}

impl PalSystemPaths {
    pub fn push(
        &mut self,
        kind: PalSystemPathKind,
        base: impl AsRef<Path>,
        path: impl AsRef<Path>,
    ) -> bool {
        let path = normalize_pal_path(base.as_ref(), path.as_ref());
        self.list_mut(kind).push(path);
        true
    }

    pub fn clear(&mut self) -> bool {
        self.normal.clear();
        self.cn.clear();
        self.tc.clear();
        true
    }

    pub fn get(&self, kind: PalSystemPathKind) -> &[PathBuf] {
        match kind {
            PalSystemPathKind::Normal => &self.normal,
            PalSystemPathKind::ChineseSimplified => &self.cn,
            PalSystemPathKind::ChineseTraditional => &self.tc,
        }
    }

    fn list_mut(&mut self, kind: PalSystemPathKind) -> &mut Vec<PathBuf> {
        match kind {
            PalSystemPathKind::Normal => &mut self.normal,
            PalSystemPathKind::ChineseSimplified => &mut self.cn,
            PalSystemPathKind::ChineseTraditional => &mut self.tc,
        }
    }
}

fn normalize_pal_path(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PalSystemPathKind {
    Normal,
    ChineseSimplified,
    ChineseTraditional,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PalWindowRequest {
    ChangeMode { mode: i32 },
    ChangeAspect { mode: i32 },
    ChangeSize { width: i32, height: i32 },
    SetPos { x: i32, y: i32 },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PalGesture {
    pub kind: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalRectI {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl PalRectI {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn width(self) -> i32 {
        self.right.saturating_sub(self.left).max(1)
    }

    pub fn height(self) -> i32 {
        self.bottom.saturating_sub(self.top).max(1)
    }

    fn normalized_nonempty(self) -> Self {
        let left = self.left.min(self.right);
        let right = self.left.max(self.right).max(left + 1);
        let top = self.top.min(self.bottom);
        let bottom = self.top.max(self.bottom).max(top + 1);
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}
