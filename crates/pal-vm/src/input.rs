use crate::event::{InputEvent, MouseButton};

/// PAL logical key identifiers. These are high-level keys used by game scripts.
/// The exact PAL virtual-key-to-bit mapping is not yet fully recovered from the binary;
/// this set covers the keys needed for basic ADV navigation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PalKey {
    Return,
    Escape,
    Space,
    Ctrl,
    Up,
    Down,
    Left,
    Right,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    MouseLeft,
    MouseRight,
    MouseMiddle,
}

/// PAL mouse button identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PalMouseButton {
    Left,
    Right,
    Middle,
}

const KEY_RETURN: u32 = 1 << 0;
const KEY_ESCAPE: u32 = 1 << 1;
const KEY_SPACE: u32 = 1 << 2;
const KEY_CTRL: u32 = 1 << 3;
const KEY_UP: u32 = 1 << 4;
const KEY_DOWN: u32 = 1 << 5;
const KEY_LEFT: u32 = 1 << 6;
const KEY_RIGHT: u32 = 1 << 7;
const KEY_F1: u32 = 1 << 8;
const KEY_F2: u32 = 1 << 9;
const KEY_F3: u32 = 1 << 10;
const KEY_F4: u32 = 1 << 11;
const KEY_F5: u32 = 1 << 12;
const KEY_F6: u32 = 1 << 13;
const KEY_F7: u32 = 1 << 14;
const KEY_F8: u32 = 1 << 15;
const KEY_F9: u32 = 1 << 16;
const KEY_F10: u32 = 1 << 17;
const KEY_F11: u32 = 1 << 18;
const KEY_F12: u32 = 1 << 19;

const MOUSE_LEFT: u8 = 1 << 0;
const MOUSE_RIGHT: u8 = 1 << 1;
const MOUSE_MIDDLE: u8 = 1 << 2;

fn pal_key_bit(key: PalKey) -> u32 {
    match key {
        PalKey::Return => KEY_RETURN,
        PalKey::Escape => KEY_ESCAPE,
        PalKey::Space => KEY_SPACE,
        PalKey::Ctrl => KEY_CTRL,
        PalKey::Up => KEY_UP,
        PalKey::Down => KEY_DOWN,
        PalKey::Left => KEY_LEFT,
        PalKey::Right => KEY_RIGHT,
        PalKey::F1 => KEY_F1,
        PalKey::F2 => KEY_F2,
        PalKey::F3 => KEY_F3,
        PalKey::F4 => KEY_F4,
        PalKey::F5 => KEY_F5,
        PalKey::F6 => KEY_F6,
        PalKey::F7 => KEY_F7,
        PalKey::F8 => KEY_F8,
        PalKey::F9 => KEY_F9,
        PalKey::F10 => KEY_F10,
        PalKey::F11 => KEY_F11,
        PalKey::F12 => KEY_F12,
        // Mouse keys are tracked separately; these overlap with mouse_on mask
        PalKey::MouseLeft | PalKey::MouseRight | PalKey::MouseMiddle => 0,
    }
}

fn pal_mouse_bit(button: PalMouseButton) -> u8 {
    match button {
        PalMouseButton::Left => MOUSE_LEFT,
        PalMouseButton::Right => MOUSE_RIGHT,
        PalMouseButton::Middle => MOUSE_MIDDLE,
    }
}

// Map winit logical key name strings to PAL key bits.
fn key_name_to_bit(key_name: &str) -> u32 {
    match key_name {
        "Return" | "Enter" | "NumpadEnter" => KEY_RETURN,
        "Escape" => KEY_ESCAPE,
        "Space" | " " => KEY_SPACE,
        "Ctrl" | "Control" | "ControlLeft" | "ControlRight" | "LeftCtrl" | "RightCtrl" => KEY_CTRL,
        "ArrowUp" => KEY_UP,
        "ArrowDown" => KEY_DOWN,
        "ArrowLeft" => KEY_LEFT,
        "ArrowRight" => KEY_RIGHT,
        "F1" => KEY_F1,
        "F2" => KEY_F2,
        "F3" => KEY_F3,
        "F4" => KEY_F4,
        "F5" => KEY_F5,
        "F6" => KEY_F6,
        "F7" => KEY_F7,
        "F8" => KEY_F8,
        "F9" => KEY_F9,
        "F10" => KEY_F10,
        "F11" => KEY_F11,
        "F12" => KEY_F12,
        _ => 0,
    }
}

fn winit_mouse_to_pal(button: MouseButton) -> u8 {
    match button {
        MouseButton::Left => MOUSE_LEFT,
        MouseButton::Right => MOUSE_RIGHT,
        MouseButton::Middle => MOUSE_MIDDLE,
        _ => 0,
    }
}

/// PAL-style stateful input. Three-state (on/push/pull) for keyboard and mouse.
///
/// PAL's input system keeps per-frame push/on/pull masks computed in sub_10246500.
/// Winit events feed into this state; the frame boundary is marked by begin_frame().
///
/// Mouse coordinates are PAL logical coordinates. Winit feeds physical window
/// coordinates, then the engine maps them through the current render scale.
#[derive(Clone, Debug, Default)]
pub struct PalInputState {
    // Keyboard three-state (bitmask over PalKey variants)
    key_on: u32,
    key_push: u32,
    key_pull: u32,

    // Mouse button three-state
    mouse_on: u8,
    mouse_push: u8,
    mouse_pull: u8,

    // Mouse position in PAL logical coordinates.
    mouse_x: i32,
    mouse_y: i32,

    // Per-frame movement delta; cleared by begin_frame.
    mouse_move_x: i32,
    mouse_move_y: i32,

    // Running cursor position in PAL logical coordinates for delta computation.
    cursor_prev_x: i32,
    cursor_prev_y: i32,

    // Last raw physical cursor position from winit.
    cursor_raw_x: f64,
    cursor_raw_y: f64,

    // Current physical-to-PAL coordinate transform.
    window_width: u32,
    window_height: u32,
    logical_width: u32,
    logical_height: u32,

    // Vertical wheel accumulator; cleared by begin_frame.
    wheel_delta: f32,

    // Tracks whether cursor position has ever been set.
    cursor_initialized: bool,
}

impl PalInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_coordinate_space(
        &mut self,
        window_width: u32,
        window_height: u32,
        logical_width: u32,
        logical_height: u32,
    ) {
        self.window_width = window_width.max(1);
        self.window_height = window_height.max(1);
        self.logical_width = logical_width.max(1);
        self.logical_height = logical_height.max(1);
        if self.cursor_initialized {
            let (x, y) = self.map_physical_to_logical(self.cursor_raw_x, self.cursor_raw_y);
            self.mouse_x = x;
            self.mouse_y = y;
            self.cursor_prev_x = x;
            self.cursor_prev_y = y;
        }
    }

    /// Clear per-frame transient state: push, pull, move delta, wheel.
    /// Must be called once per frame before new input events are processed.
    pub fn begin_frame(&mut self) {
        self.key_push = 0;
        self.key_pull = 0;
        self.mouse_push = 0;
        self.mouse_pull = 0;
        self.mouse_move_x = 0;
        self.mouse_move_y = 0;
        self.wheel_delta = 0.0;
    }

    /// Clear PAL's accumulated input masks. Mirrors PalInputClear_0's block reset.
    pub fn clear(&mut self) {
        self.key_on = 0;
        self.key_push = 0;
        self.key_pull = 0;
        self.mouse_on = 0;
        self.mouse_push = 0;
        self.mouse_pull = 0;
        self.mouse_move_x = 0;
        self.mouse_move_y = 0;
        self.wheel_delta = 0.0;
    }

    /// OR raw PAL key bits into the held-key mask.
    pub fn set_key_on_bits(&mut self, bits: u32) {
        self.key_on |= bits;
    }

    /// OR raw PAL key bits into the per-frame push mask.
    pub fn set_key_push_bits(&mut self, bits: u32) {
        self.key_push |= bits;
    }

    /// OR raw PAL key bits into the per-frame pull mask.
    pub fn set_key_pull_bits(&mut self, bits: u32) {
        self.key_pull |= bits;
    }

    /// Process a keyboard event.
    pub fn handle_keyboard_event(&mut self, key_name: &str, pressed: bool) {
        let bit = key_name_to_bit(key_name);
        if bit == 0 {
            return;
        }
        if pressed {
            if self.key_on & bit == 0 {
                self.key_push |= bit;
            }
            self.key_on |= bit;
        } else {
            if self.key_on & bit != 0 {
                self.key_pull |= bit;
            }
            self.key_on &= !bit;
        }
    }

    /// Process a mouse button event.
    pub fn handle_mouse_button_event(&mut self, button: MouseButton, pressed: bool) {
        let bit = winit_mouse_to_pal(button);
        if bit == 0 {
            return;
        }
        if pressed {
            if self.mouse_on & bit == 0 {
                self.mouse_push |= bit;
            }
            self.mouse_on |= bit;
        } else {
            if self.mouse_on & bit != 0 {
                self.mouse_pull |= bit;
            }
            self.mouse_on &= !bit;
        }
    }

    /// Process a cursor-moved event.
    pub fn handle_cursor_moved(&mut self, x: f64, y: f64) {
        self.cursor_raw_x = x;
        self.cursor_raw_y = y;
        let (new_x, new_y) = self.map_physical_to_logical(x, y);
        if self.cursor_initialized {
            self.mouse_move_x += new_x - self.cursor_prev_x;
            self.mouse_move_y += new_y - self.cursor_prev_y;
        }
        self.cursor_prev_x = new_x;
        self.cursor_prev_y = new_y;
        self.mouse_x = new_x;
        self.mouse_y = new_y;
        self.cursor_initialized = true;
    }

    fn map_physical_to_logical(&self, x: f64, y: f64) -> (i32, i32) {
        let window_width = self.window_width.max(1) as f64;
        let window_height = self.window_height.max(1) as f64;
        let logical_width = self.logical_width.max(1) as f64;
        let logical_height = self.logical_height.max(1) as f64;
        (
            ((x * logical_width) / window_width).round() as i32,
            ((y * logical_height) / window_height).round() as i32,
        )
    }

    /// Process a mouse wheel event.
    pub fn handle_mouse_wheel(&mut self, _delta_x: f32, delta_y: f32) {
        self.wheel_delta += delta_y;
    }

    /// True if the key was pressed this frame (was not held last frame).
    pub fn key_push(&self, key: PalKey) -> bool {
        self.key_push & pal_key_bit(key) != 0
    }

    /// True if the key is currently held.
    pub fn key_on(&self, key: PalKey) -> bool {
        self.key_on & pal_key_bit(key) != 0
    }

    /// True if the key was released this frame.
    pub fn key_pull(&self, key: PalKey) -> bool {
        self.key_pull & pal_key_bit(key) != 0
    }

    /// True if the mouse button was pressed this frame.
    pub fn mouse_push(&self, button: PalMouseButton) -> bool {
        self.mouse_push & pal_mouse_bit(button) != 0
    }

    /// True if the mouse button is currently held.
    pub fn mouse_on(&self, button: PalMouseButton) -> bool {
        self.mouse_on & pal_mouse_bit(button) != 0
    }

    /// True if the mouse button was released this frame.
    pub fn mouse_pull(&self, button: PalMouseButton) -> bool {
        self.mouse_pull & pal_mouse_bit(button) != 0
    }

    /// Current mouse position in window coordinates.
    pub fn mouse_position(&self) -> (i32, i32) {
        (self.mouse_x, self.mouse_y)
    }

    /// Per-frame mouse movement delta.
    pub fn mouse_delta(&self) -> (i32, i32) {
        (self.mouse_move_x, self.mouse_move_y)
    }

    /// Vertical wheel delta accumulated this frame.
    pub fn wheel_delta(&self) -> f32 {
        self.wheel_delta
    }

    /// True if any key, mouse button, or positive mouse wheel input was pushed this frame.
    pub fn any_push(&self) -> bool {
        self.key_push != 0 || self.mouse_push != 0 || self.wheel_delta > 0.0
    }

    /// True if a push this frame counts as a "click" that completes PAL click
    /// waits and the ADV typewriter reveal.
    ///
    /// Mirrors the push half of koikake.exe GetSkipState (0x437800): Space or
    /// Return push (PalInputGetKeyPush & 0x6), left mouse push (& 0x800000),
    /// or positive wheel delta (PalInputGetWheel > 0) set bit 3. Other keys
    /// (Escape, arrows, F-keys, right/middle mouse) never complete click
    /// waits; they reach scripts only through system-button slots.
    pub fn click_push(&self) -> bool {
        self.key_push & (KEY_SPACE | KEY_RETURN) != 0
            || self.mouse_push & MOUSE_LEFT != 0
            || self.wheel_delta > 0.0
    }

    /// True while the PAL fast-forward modifier is held.
    ///
    /// Native ADV skip is driven by PAL's keyboard state, not by a synthetic
    /// VM-side flag.  The headless Ctrl diagnostics feed the same keyboard
    /// path as winit, and wait/text code consumes this held state where the
    /// original engine treats Ctrl as fast-forward.
    pub fn fast_forward_held(&self) -> bool {
        self.key_on(PalKey::Ctrl)
    }

    /// Return a frame-local copy with mouse push edges removed.
    ///
    /// Game.exe button reactions consume the mouse click before PAL wait-click
    /// tasks see it. The held mouse state is preserved so hover/pressed visuals
    /// continue to render correctly during the same frame.
    pub fn without_mouse_push(&self) -> Self {
        let mut copy = self.clone();
        copy.mouse_push = 0;
        copy
    }

    /// Return a frame-local copy with all push edges removed.
    ///
    /// Used when an input edge has been consumed by the ADV typewriter reveal:
    /// the held state remains available for visuals, but the same edge must not
    /// also complete the following PAL wait-click task.
    pub fn without_push_edges(&self) -> Self {
        let mut copy = self.clone();
        copy.key_push = 0;
        copy.mouse_push = 0;
        copy.wheel_delta = 0.0;
        copy
    }

    /// Raw keyboard push bitmask (for direct PAL API access).
    pub fn raw_key_push(&self) -> u32 {
        self.key_push
    }

    /// Raw keyboard on bitmask.
    pub fn raw_key_on(&self) -> u32 {
        self.key_on
    }

    /// Raw keyboard pull bitmask.
    pub fn raw_key_pull(&self) -> u32 {
        self.key_pull
    }

    pub fn raw_mouse_on(&self) -> u8 {
        self.mouse_on
    }

    pub fn raw_mouse_push(&self) -> u8 {
        self.mouse_push
    }

    pub fn raw_mouse_pull(&self) -> u8 {
        self.mouse_pull
    }

    /// Feed a PalEvent::Input directly. Convenience wrapper for engine.handle_event.
    pub fn handle_input_event(&mut self, event: &crate::event::InputEvent) {
        match event {
            InputEvent::Keyboard { key_name, pressed } => {
                self.handle_keyboard_event(key_name, *pressed);
            }
            InputEvent::MouseInput { button, pressed } => {
                self.handle_mouse_button_event(*button, *pressed);
            }
            InputEvent::CursorMoved { x, y } => {
                self.handle_cursor_moved(*x, *y);
            }
            InputEvent::MouseWheel { delta_x, delta_y } => {
                self.handle_mouse_wheel(*delta_x, *delta_y);
            }
        }
    }
}
