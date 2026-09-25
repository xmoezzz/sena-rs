use std::sync::Arc;
use std::time::Duration;

use crate::runtime::RuntimeStatus;

#[derive(Clone, Debug)]
pub struct FrameScene {
    pub clear_color: [f64; 4],
    pub diagnostic_label: String,
    pub logical_width: u32,
    pub logical_height: u32,
    pub textures: Vec<SceneTexture>,
    pub commands: Vec<DrawCommand>,
}

impl FrameScene {
    pub const PAL_DEFAULT_WIDTH: u32 = 1280;
    pub const PAL_DEFAULT_HEIGHT: u32 = 720;

    pub fn boot() -> Self {
        Self {
            clear_color: [0.02, 0.02, 0.025, 1.0],
            diagnostic_label: "boot".to_owned(),
            logical_width: Self::PAL_DEFAULT_WIDTH,
            logical_height: Self::PAL_DEFAULT_HEIGHT,
            textures: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub fn from_runtime_status(status: &RuntimeStatus, elapsed: Duration) -> Self {
        let pulse = ((elapsed.as_secs_f64() * 2.0).sin() * 0.5) + 0.5;
        let clear_color = match status {
            RuntimeStatus::NotBooted => [0.02, 0.02, 0.025, 1.0],
            RuntimeStatus::Running { .. } => [0.015, 0.020 + pulse * 0.010, 0.045, 1.0],
            RuntimeStatus::WaitFrame { .. } => [0.020, 0.030, 0.055, 1.0],
            RuntimeStatus::WaitClick { .. } => [0.035 + pulse * 0.015, 0.020, 0.050, 1.0],
            RuntimeStatus::Halted { .. } => [0.015, 0.045, 0.020, 1.0],
            RuntimeStatus::UnsupportedCommand { .. } | RuntimeStatus::UnsupportedExtCall { .. } => {
                [0.065, 0.040 + pulse * 0.015, 0.015, 1.0]
            }
            RuntimeStatus::Faulted { .. } => [0.070, 0.010, 0.010, 1.0],
        };
        Self {
            clear_color,
            diagnostic_label: status.to_string(),
            logical_width: Self::PAL_DEFAULT_WIDTH,
            logical_height: Self::PAL_DEFAULT_HEIGHT,
            textures: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub fn with_logical_size(mut self, width: u32, height: u32) -> Self {
        self.logical_width = width.max(1);
        self.logical_height = height.max(1);
        self
    }

    pub fn with_texture(mut self, texture: SceneTexture) -> Self {
        self.textures.push(texture);
        self
    }

    pub fn with_command(mut self, command: DrawCommand) -> Self {
        self.commands.push(command);
        self
    }
}

impl Default for FrameScene {
    fn default() -> Self {
        Self::boot()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SceneTextureId(pub u64);

#[derive(Clone, Debug)]
pub struct SceneTexture {
    pub id: SceneTextureId,
    pub generation: u64,
    pub width: u32,
    pub height: u32,
    pub format: SceneTextureFormat,
    pub pixels: Arc<[u8]>,
}

impl SceneTexture {
    pub fn rgba8(
        id: SceneTextureId,
        generation: u64,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> Self {
        Self {
            id,
            generation,
            width,
            height,
            format: SceneTextureFormat::Rgba8,
            pixels: Arc::from(pixels),
        }
    }

    pub fn checked_rgba8(
        id: SceneTextureId,
        generation: u64,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> anyhow::Result<Self> {
        let expected = width as usize * height as usize * 4;
        if pixels.len() != expected {
            return Err(anyhow::anyhow!(
                "RGBA texture {:?} has {} bytes, expected {} for {}x{}",
                id,
                pixels.len(),
                expected,
                width,
                height
            ));
        }
        Ok(Self::rgba8(id, generation, width, height, pixels))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneTextureFormat {
    Rgba8,
}

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Sprite(SpriteDraw),
    SolidQuad(SolidQuad),
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteDraw {
    pub texture_id: SceneTextureId,
    /// Smooth UI surfaces when the window has more physical than PAL pixels.
    pub smooth_upscale: bool,
    pub priority: i32,
    pub dst: RectF,
    pub src: RectF,
    pub source_rect: [i32; 4],
    pub texture_size: [u32; 2],
    pub cell_size: [u32; 2],
    pub position: [f32; 3],
    pub offset: [i32; 2],
    pub color: [f32; 4],
    pub scale: f32,
    pub rotation: [f32; 3],
    pub center_offset: [f32; 2],
    pub render_mode: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct SolidQuad {
    pub dst: RectF,
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectF {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn is_drawable(self) -> bool {
        self.w.abs() > f32::EPSILON && self.h.abs() > f32::EPSILON
    }
}

pub fn rasterize_scene_rgba(scene: &FrameScene) -> Vec<u8> {
    let width = scene.logical_width.max(1);
    let height = scene.logical_height.max(1);
    let mut pixels = vec![0u8; width as usize * height as usize * 4];
    let bg = [
        (scene.clear_color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (scene.clear_color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (scene.clear_color[2].clamp(0.0, 1.0) * 255.0) as u8,
        255,
    ];
    for px in pixels.chunks_exact_mut(4) {
        px.copy_from_slice(&bg);
    }
    for command in &scene.commands {
        match command {
            DrawCommand::Sprite(sprite) => {
                if let Some(texture) = scene
                    .textures
                    .iter()
                    .find(|texture| texture.id == sprite.texture_id)
                {
                    blit_sprite(
                        &mut pixels,
                        width,
                        height,
                        texture,
                        sprite.dst,
                        sprite.source_rect,
                        sprite.color,
                    );
                }
            }
            DrawCommand::SolidQuad(quad) => {
                fill_rect(&mut pixels, width, height, quad.dst, quad.color);
            }
        }
    }
    pixels
}

fn blit_sprite(
    target: &mut [u8],
    target_width: u32,
    target_height: u32,
    texture: &SceneTexture,
    dst: RectF,
    source_rect: [i32; 4],
    color: [f32; 4],
) {
    let x0 = dst.x.floor().max(0.0) as i32;
    let y0 = dst.y.floor().max(0.0) as i32;
    let x1 = (dst.x + dst.w).ceil().min(target_width as f32) as i32;
    let y1 = (dst.y + dst.h).ceil().min(target_height as f32) as i32;
    if x1 <= x0 || y1 <= y0 || dst.w.abs() < f32::EPSILON || dst.h.abs() < f32::EPSILON {
        return;
    }
    let sx0 = source_rect[0].max(0) as f32;
    let sy0 = source_rect[1].max(0) as f32;
    let sw = source_rect[2].saturating_sub(source_rect[0]).max(1) as f32;
    let sh = source_rect[3].saturating_sub(source_rect[1]).max(1) as f32;
    for y in y0..y1 {
        let v = ((y as f32 + 0.5 - dst.y) / dst.h).clamp(0.0, 1.0);
        let sy = (sy0 + v * sh)
            .floor()
            .clamp(0.0, texture.height.saturating_sub(1) as f32) as u32;
        for x in x0..x1 {
            let u = ((x as f32 + 0.5 - dst.x) / dst.w).clamp(0.0, 1.0);
            let sx = (sx0 + u * sw)
                .floor()
                .clamp(0.0, texture.width.saturating_sub(1) as f32) as u32;
            let src_idx = (sy as usize * texture.width as usize + sx as usize) * 4;
            let src = [
                texture.pixels[src_idx] as f32 * color[0].clamp(0.0, 1.0),
                texture.pixels[src_idx + 1] as f32 * color[1].clamp(0.0, 1.0),
                texture.pixels[src_idx + 2] as f32 * color[2].clamp(0.0, 1.0),
                texture.pixels[src_idx + 3] as f32 * color[3].clamp(0.0, 1.0),
            ];
            blend_pixel(target, target_width, x as u32, y as u32, src);
        }
    }
}

fn fill_rect(
    target: &mut [u8],
    target_width: u32,
    target_height: u32,
    rect: RectF,
    color: [f32; 4],
) {
    let x0 = rect.x.floor().max(0.0) as i32;
    let y0 = rect.y.floor().max(0.0) as i32;
    let x1 = (rect.x + rect.w).ceil().min(target_width as f32) as i32;
    let y1 = (rect.y + rect.h).ceil().min(target_height as f32) as i32;
    let src = [
        color[0].clamp(0.0, 1.0) * 255.0,
        color[1].clamp(0.0, 1.0) * 255.0,
        color[2].clamp(0.0, 1.0) * 255.0,
        color[3].clamp(0.0, 1.0) * 255.0,
    ];
    for y in y0..y1 {
        for x in x0..x1 {
            blend_pixel(target, target_width, x as u32, y as u32, src);
        }
    }
}

fn blend_pixel(target: &mut [u8], target_width: u32, x: u32, y: u32, src: [f32; 4]) {
    let idx = (y as usize * target_width as usize + x as usize) * 4;
    let alpha = (src[3] / 255.0).clamp(0.0, 1.0);
    let inv_alpha = 1.0 - alpha;
    target[idx] = (src[0] * alpha + target[idx] as f32 * inv_alpha).round() as u8;
    target[idx + 1] = (src[1] * alpha + target[idx + 1] as f32 * inv_alpha).round() as u8;
    target[idx + 2] = (src[2] * alpha + target[idx + 2] as f32 * inv_alpha).round() as u8;
    target[idx + 3] = 255;
}
