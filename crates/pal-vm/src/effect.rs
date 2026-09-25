use crate::scene::{RectF, SolidQuad};

#[derive(Clone, Debug, PartialEq)]
pub struct PalEffectSystem {
    enabled: bool,
    active: Option<PalEffectState>,
    flash: Option<PalFlashState>,
    shake: Option<PalShakeState>,
}

impl Default for PalEffectSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PalEffectSystem {
    pub fn new() -> Self {
        Self {
            enabled: true,
            active: None,
            flash: None,
            shake: None,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn active(&self) -> bool {
        self.active.is_some()
    }

    /// Game.exe category 16 index 0 (`sub_412450`) moves one or more active
    /// native effect channels into their stop/fade-out state.  The portable
    /// effect model has one scene transition plus flash/shake overlays, so the
    /// equivalent operation is to finish the selected channels immediately.
    pub fn stop_selected(&mut self, flags: i32) {
        if (flags & 1) != 0 {
            self.active = None;
        }
        if (flags & 4) != 0 {
            self.shake = None;
        }
        if (flags & 8) != 0 || (flags & 0x10) != 0 {
            self.flash = None;
        }
    }

    /// PalEffectEx(id, duration, arg, wait): id 0 is a no-op, >0x31 fails.
    pub fn effect_ex(
        &mut self,
        effect_id: u32,
        duration_ms: u32,
        arg: i32,
        wait: bool,
        now_ms: u32,
    ) -> i32 {
        if !self.enabled || effect_id == 0 {
            return 1;
        }
        if effect_id > 0x31 || self.active.is_some() {
            return 0;
        }
        self.active = Some(PalEffectState {
            effect_id,
            duration_ms: duration_ms.max(1),
            arg,
            wait,
            start_ms: now_ms,
            state: 1,
        });
        1
    }

    pub fn effect(&mut self, effect_id: u32, duration_ms: u32, now_ms: u32) -> i32 {
        self.effect_ex(effect_id, duration_ms, 0, true, now_ms)
    }

    pub fn tick(&mut self, now_ms: u32) {
        let Some(active) = self.active else {
            self.tick_flash_and_shake(now_ms);
            return;
        };
        if now_ms.wrapping_sub(active.start_ms) >= active.duration_ms {
            self.active = None;
        }
        self.tick_flash_and_shake(now_ms);
    }

    fn tick_flash_and_shake(&mut self, now_ms: u32) {
        if let Some(flash) = self.flash {
            if now_ms.wrapping_sub(flash.start_ms) >= flash.duration_ms {
                self.flash = None;
            }
        }
        if let Some(shake) = self.shake {
            if now_ms.wrapping_sub(shake.start_ms) >= shake.duration_ms {
                self.shake = None;
            }
        }
    }

    /// Game.exe category 16 index 2 (`sub_4120C0`) creates a full-screen PAL
    /// sprite, paints it with RGB, records a start time and duration, then
    /// marks the effect pipeline dirty.  The portable renderer keeps the same
    /// script-visible timing and draws the colored quad in logical coordinates.
    pub fn flash_color(&mut self, r: i32, g: i32, b: i32, duration_ms: u32, now_ms: u32) {
        if !self.enabled {
            return;
        }
        self.flash = Some(PalFlashState {
            color: [
                color_byte_to_unit(r),
                color_byte_to_unit(g),
                color_byte_to_unit(b),
            ],
            duration_ms: duration_ms.max(1),
            start_ms: now_ms,
        });
    }

    /// Game.exe category 16 index 1 (`sub_4122A0`) stores a screen-shake
    /// request.  Applying the shake offset to the whole render tree is handled
    /// by later renderer integration; keeping the native timer/state here
    /// prevents the extcall from being a silent no-op.
    pub fn set_shake(&mut self, x_amp: i32, y_amp: i32, duration_ms: u32, phase: i32, now_ms: u32) {
        if !self.enabled || duration_ms == u32::MAX {
            return;
        }
        self.shake = Some(PalShakeState {
            x_amp,
            y_amp,
            duration_ms: duration_ms.max(1),
            phase,
            start_ms: now_ms,
        });
    }

    pub fn overlay_quad(
        &self,
        logical_width: u32,
        logical_height: u32,
        now_ms: u32,
    ) -> Option<SolidQuad> {
        if let Some(flash) = self.flash {
            let elapsed = now_ms.wrapping_sub(flash.start_ms).min(flash.duration_ms);
            let t = elapsed as f32 / flash.duration_ms.max(1) as f32;
            let alpha = (1.0 - t).clamp(0.0, 1.0);
            if alpha > 0.0 {
                return Some(SolidQuad {
                    dst: RectF::new(
                        0.0,
                        0.0,
                        logical_width.max(1) as f32,
                        logical_height.max(1) as f32,
                    ),
                    color: [flash.color[0], flash.color[1], flash.color[2], alpha],
                });
            }
        }

        let active = self.active?;
        let elapsed = now_ms.wrapping_sub(active.start_ms).min(active.duration_ms);
        let t = elapsed as f32 / active.duration_ms.max(1) as f32;
        let alpha = effect_alpha(active.effect_id, t);
        if alpha <= 0.0 {
            return None;
        }
        let color = if active.arg < 0 {
            [1.0, 1.0, 1.0, alpha]
        } else {
            [0.0, 0.0, 0.0, alpha]
        };
        Some(SolidQuad {
            dst: RectF::new(
                0.0,
                0.0,
                logical_width.max(1) as f32,
                logical_height.max(1) as f32,
            ),
            color,
        })
    }

    pub fn state(&self) -> Option<PalEffectState> {
        self.active
    }

    /// Returns the render-tree translation for the active screen shake.
    ///
    /// Native PAL uses a decaying random offset whose range reaches zero at
    /// the effect deadline.  The portable renderer keeps the same bounds and
    /// timing with a deterministic hash so headless frame dumps stay stable.
    pub fn shake_offset(&self, now_ms: u32) -> [i32; 2] {
        let Some(shake) = self.shake else {
            return [0, 0];
        };
        let elapsed = now_ms.wrapping_sub(shake.start_ms);
        if elapsed >= shake.duration_ms {
            return [0, 0];
        }

        let remaining = 1.0 - (elapsed as f32 / shake.duration_ms.max(1) as f32).clamp(0.0, 1.0);
        let span_x = shake_span(shake.x_amp, remaining);
        let span_y = shake_span(shake.y_amp, remaining);
        if span_x == 0 && span_y == 0 {
            return [0, 0];
        }

        // Native shake samples a new displacement periodically; 30 Hz keeps
        // the motion visible without making the frame output platform-timed.
        let tick = elapsed / 33;
        let seed = (shake.phase as u32)
            .wrapping_add(tick.wrapping_mul(0x9E37_79B9))
            .wrapping_add(shake.start_ms.rotate_left(7));
        [
            shake_sample(seed, span_x),
            shake_sample(seed ^ 0xA5A5_5A5A, span_y),
        ]
    }
}

fn shake_span(amplitude: i32, remaining: f32) -> i32 {
    if amplitude == 0 || remaining <= 0.0 {
        return 0;
    }
    let limit = (i32::MAX / 2) as f32;
    ((amplitude.unsigned_abs() as f32 * remaining)
        .round()
        .clamp(0.0, limit)) as i32
}

fn shake_sample(seed: u32, span: i32) -> i32 {
    if span == 0 {
        return 0;
    }
    let mut hash = seed;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7FEB_352D);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846C_A68B);
    hash ^= hash >> 16;

    let width = span as u64 * 2 + 1;
    (u64::from(hash) % width) as i32 - span
}

fn color_byte_to_unit(value: i32) -> f32 {
    (value.clamp(0, 255) as f32) / 255.0
}

fn effect_alpha(effect_id: u32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match effect_id % 4 {
        1 => 1.0 - t,
        2 => t,
        3 => (1.0 - ((t - 0.5).abs() * 2.0)).clamp(0.0, 1.0),
        _ => 1.0 - t,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalEffectState {
    pub effect_id: u32,
    pub duration_ms: u32,
    pub arg: i32,
    pub wait: bool,
    pub start_ms: u32,
    pub state: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalFlashState {
    pub color: [f32; 3],
    pub duration_ms: u32,
    pub start_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PalShakeState {
    pub x_amp: i32,
    pub y_amp: i32,
    pub duration_ms: u32,
    pub phase: i32,
    pub start_ms: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_offset_decays_and_expires() {
        let mut effects = PalEffectSystem::new();
        effects.set_shake(8, 4, 100, 7, 1_000);

        let start = effects.shake_offset(1_000);
        assert!(start[0].abs() <= 8);
        assert!(start[1].abs() <= 4);
        assert_ne!(start, [0, 0]);

        let late = effects.shake_offset(1_099);
        assert!(late[0].abs() <= 8);
        assert!(late[1].abs() <= 4);
        assert_eq!(effects.shake_offset(1_100), [0, 0]);
    }
}
