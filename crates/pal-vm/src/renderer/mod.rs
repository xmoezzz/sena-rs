use std::collections::HashMap;
use std::io::Write;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use crate::scene::{
    rasterize_scene_rgba, DrawCommand, FrameScene, RectF, SceneTexture, SceneTextureFormat,
    SceneTextureId, SolidQuad, SpriteDraw,
};

mod shader;
mod wgpu_backend;

pub use shader::{shader_source, ShaderProgram};
use wgpu_backend::WgpuBackend;

#[derive(Clone, Copy, Debug)]
pub struct RendererConfig {
    pub clear_color: wgpu::Color,
    pub virtual_width: u32,
    pub virtual_height: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            clear_color: wgpu::Color {
                r: 0.02,
                g: 0.02,
                b: 0.025,
                a: 1.0,
            },
            virtual_width: FrameScene::PAL_DEFAULT_WIDTH,
            virtual_height: FrameScene::PAL_DEFAULT_HEIGHT,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderOutcome {
    Rendered,
    Skipped,
    Reconfigured,
}

pub struct Renderer {
    window: Arc<Window>,
    backend: Backend,
    size: PhysicalSize<u32>,
    virtual_size: PhysicalSize<u32>,
    clear_color: wgpu::Color,
    frame_dump_path: Option<String>,
    frame_dump_written: bool,
}

enum Backend {
    Software(SoftwareBackend),
    Wgpu(WgpuBackend),
}

struct SoftwareBackend {
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    scene_textures: HashMap<SceneTextureId, CachedTexture>,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        _display_handle: OwnedDisplayHandle,
        renderer_config: RendererConfig,
    ) -> anyhow::Result<Self> {
        let size = nonzero_size(window.inner_size());
        let virtual_size = PhysicalSize::new(
            renderer_config.virtual_width.max(1),
            renderer_config.virtual_height.max(1),
        );
        let frame_dump_path = std::env::var("PAL_RENDER_DUMP").ok();
        let force_software = std::env::var("PAL_RENDERER")
            .ok()
            .as_deref()
            .is_some_and(|value| value.eq_ignore_ascii_case("software"));
        if !force_software {
            match WgpuBackend::new(window.clone(), size).await {
                Ok(backend) => {
                    return Ok(Self {
                        window,
                        backend: Backend::Wgpu(backend),
                        size,
                        virtual_size,
                        clear_color: renderer_config.clear_color,
                        frame_dump_path,
                        frame_dump_written: false,
                    });
                }
                Err(err) => {
                    log::warn!(
                        "GPU renderer unavailable ({err:#}); falling back to the software compositor"
                    );
                }
            }
        }
        let context =
            softbuffer::Context::new(window.clone()).map_err(|err| softbuffer_error(err))?;
        let mut surface = softbuffer::Surface::new(&context, window.clone())
            .map_err(|err| softbuffer_error(err))?;
        surface
            .resize(nonzero(size.width), nonzero(size.height))
            .map_err(|err| softbuffer_error(err))?;
        Ok(Self {
            window,
            backend: Backend::Software(SoftwareBackend {
                surface,
                scene_textures: HashMap::new(),
            }),
            size,
            virtual_size,
            clear_color: renderer_config.clear_color,
            frame_dump_path,
            frame_dump_written: false,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let size = nonzero_size(size);
        if size == self.size {
            return;
        }
        self.size = size;
        match &mut self.backend {
            Backend::Software(backend) => {
                if let Err(err) = backend
                    .surface
                    .resize(nonzero(size.width), nonzero(size.height))
                {
                    log::error!("failed to resize software renderer surface: {err}");
                }
            }
            Backend::Wgpu(backend) => backend.resize(size),
        }
    }

    pub fn render(&mut self, scene: &FrameScene) -> RenderOutcome {
        self.render_with_png_dump(scene, None)
    }

    pub fn render_with_png_dump(
        &mut self,
        scene: &FrameScene,
        dump_path: Option<&Path>,
    ) -> RenderOutcome {
        if self.size.width == 0 || self.size.height == 0 {
            return RenderOutcome::Skipped;
        }
        self.virtual_size = PhysicalSize::new(
            scene.logical_width.max(1),
            scene.logical_height.max(1),
        );
        let frame_dump = if self.frame_dump_written {
            None
        } else {
            self.frame_dump_path.as_deref()
        };
        match &mut self.backend {
            Backend::Software(backend) => {
                match backend.render(scene, self.clear_color, self.size, frame_dump, dump_path) {
                    Ok(wrote_frame_dump) => {
                        self.frame_dump_written |= wrote_frame_dump;
                        RenderOutcome::Rendered
                    }
                    Err(err) => {
                        log::error!("software renderer failed: {err}");
                        RenderOutcome::Skipped
                    }
                }
            }
            Backend::Wgpu(backend) => {
                let outcome = match backend.render(scene, self.clear_color) {
                    Ok(outcome) => outcome,
                    Err(err) => {
                        log::error!("wgpu renderer failed: {err}");
                        RenderOutcome::Skipped
                    }
                };
                if frame_dump.is_some() || dump_path.is_some() {
                    // GPU readback is not wired up; diagnostic dumps are
                    // rasterized on the CPU at logical resolution instead, and
                    // do not depend on the frame having been presented.
                    let rgba = rasterize_scene_rgba(scene);
                    let (width, height) = (scene.logical_width.max(1), scene.logical_height.max(1));
                    if let Some(path) = frame_dump {
                        match write_rgba_png(Path::new(path), &rgba, width, height) {
                            Ok(()) => {
                                self.frame_dump_written = true;
                                log::info!("wrote renderer frame dump to {path}");
                            }
                            Err(err) => log::error!("failed to write frame dump {path}: {err}"),
                        }
                    }
                    if let Some(path) = dump_path {
                        if let Err(err) = write_rgba_png(path, &rgba, width, height) {
                            log::error!("failed to write frame dump {}: {err}", path.display());
                        }
                    }
                }
                outcome
            }
        }
    }
}

impl SoftwareBackend {
    fn upload_scene_textures(&mut self, scene: &FrameScene) {
        self.scene_textures.retain(|texture_id, _| {
            scene
                .textures
                .iter()
                .any(|texture| texture.id == *texture_id)
        });
        for texture in &scene.textures {
            let needs_upload = self.scene_textures.get(&texture.id).is_none_or(|cached| {
                cached.generation != texture.generation
                    || cached.width != texture.width
                    || cached.height != texture.height
            });
            if needs_upload {
                match CachedTexture::from_scene(texture) {
                    Ok(cached) => {
                        self.scene_textures.insert(texture.id, cached);
                    }
                    Err(err) => {
                        log::error!("failed to cache scene texture {:?}: {err}", texture.id);
                    }
                }
            }
        }
    }

    fn render(
        &mut self,
        scene: &FrameScene,
        fallback_clear: wgpu::Color,
        size: PhysicalSize<u32>,
        frame_dump_path: Option<&str>,
        dump_path: Option<&Path>,
    ) -> anyhow::Result<bool> {
        self.upload_scene_textures(scene);
        let width = size.width as usize;
        let height = size.height as usize;
        let clear = color_to_rgb(scene_clear_color(scene, fallback_clear));
        let metrics = RenderTargetMetrics::new(
            [width as u32, height as u32],
            [scene.logical_width.max(1), scene.logical_height.max(1)],
        );
        let scene_textures = &self.scene_textures;
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|err| softbuffer_error(err))?;
        if buffer.len() != width.saturating_mul(height) {
            anyhow::bail!(
                "software surface has {} pixels, expected {}",
                buffer.len(),
                width.saturating_mul(height)
            );
        }
        buffer.fill(clear);
        for command in &scene.commands {
            match command {
                DrawCommand::Sprite(sprite) => draw_sprite(
                    scene_textures,
                    &mut buffer,
                    width,
                    height,
                    metrics.scale,
                    sprite,
                ),
                DrawCommand::SolidQuad(quad) => {
                    draw_solid_quad(&mut buffer, width, height, metrics.scale, *quad)
                }
            }
        }
        if pal_debug_enabled() {
            eprintln!(
                "[PAL_DEBUG] render_target: window={}x{} logical={}x{} surface={}x{} scale=({:.6},{:.6}) viewport=({}, {}, {}x{})",
                metrics.window_physical_size[0],
                metrics.window_physical_size[1],
                metrics.logical_size[0],
                metrics.logical_size[1],
                metrics.surface_size[0],
                metrics.surface_size[1],
                metrics.scale[0],
                metrics.scale[1],
                metrics.viewport.x,
                metrics.viewport.y,
                metrics.viewport.w,
                metrics.viewport.h,
            );
            for (index, command) in scene.commands.iter().enumerate() {
                if let DrawCommand::Sprite(sprite) = command {
                    let transform = sprite_transform_debug(sprite, &metrics);
                    eprintln!(
                        "[PAL_DEBUG] render_sprite[{index}]: dst_pal=({:.3},{:.3},{:.3}x{:.3}) dst_device=({:.3},{:.3},{:.3}x{:.3}) uv=({:.6},{:.6},{:.6}x{:.6}) quad=({:.6},{:.6}) ({:.6},{:.6}) ({:.6},{:.6}) ({:.6},{:.6}) prio={}",
                        sprite.dst.x,
                        sprite.dst.y,
                        sprite.dst.w,
                        sprite.dst.h,
                        transform.dst_device.x,
                        transform.dst_device.y,
                        transform.dst_device.w,
                        transform.dst_device.h,
                        transform.uv.x,
                        transform.uv.y,
                        transform.uv.w,
                        transform.uv.h,
                        transform.clip_quad[0][0],
                        transform.clip_quad[0][1],
                        transform.clip_quad[1][0],
                        transform.clip_quad[1][1],
                        transform.clip_quad[2][0],
                        transform.clip_quad[2][1],
                        transform.clip_quad[3][0],
                        transform.clip_quad[3][1],
                        sprite.priority,
                    );
                }
            }
        }
        let mut wrote_frame_dump = false;
        if let Some(path) = frame_dump_path {
            write_ppm(path, &buffer, width, height)?;
            wrote_frame_dump = true;
            log::info!("wrote software renderer frame dump to {path}");
        }
        if let Some(path) = dump_path {
            write_surface_png(path, &buffer, width, height)?;
        }
        buffer.present().map_err(|err| softbuffer_error(err))?;
        Ok(wrote_frame_dump)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderTargetMetrics {
    pub window_physical_size: [u32; 2],
    pub logical_size: [u32; 2],
    pub surface_size: [u32; 2],
    pub scale: [f32; 2],
    pub viewport: RectF,
}

impl RenderTargetMetrics {
    pub fn new(surface_size: [u32; 2], logical_size: [u32; 2]) -> Self {
        let surface_w = surface_size[0].max(1);
        let surface_h = surface_size[1].max(1);
        let logical_w = logical_size[0].max(1);
        let logical_h = logical_size[1].max(1);
        let scale = [
            surface_w as f32 / logical_w as f32,
            surface_h as f32 / logical_h as f32,
        ];
        Self {
            window_physical_size: [surface_w, surface_h],
            logical_size: [logical_w, logical_h],
            surface_size: [surface_w, surface_h],
            scale,
            viewport: RectF::new(0.0, 0.0, surface_w as f32, surface_h as f32),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteTransformDebug {
    pub dst_device: RectF,
    pub uv: RectF,
    pub clip_quad: [[f32; 2]; 4],
}

pub fn sprite_transform_debug(
    sprite: &SpriteDraw,
    metrics: &RenderTargetMetrics,
) -> SpriteTransformDebug {
    let dst_device = scaled_rect(sprite.dst, metrics.scale);
    let uv = sprite.src;
    SpriteTransformDebug {
        dst_device,
        uv,
        clip_quad: pal_device_rect_to_clip_quad(dst_device, metrics.surface_size),
    }
}

pub fn scaled_rect(rect: RectF, scale: [f32; 2]) -> RectF {
    RectF::new(
        rect.x * scale[0],
        rect.y * scale[1],
        rect.w * scale[0],
        rect.h * scale[1],
    )
}

pub fn pal_device_rect_to_clip_quad(rect: RectF, surface_size: [u32; 2]) -> [[f32; 2]; 4] {
    let w = surface_size[0].max(1) as f32;
    let h = surface_size[1].max(1) as f32;
    let x0 = (rect.x / w) * 2.0 - 1.0;
    let x1 = ((rect.x + rect.w) / w) * 2.0 - 1.0;
    let y0 = 1.0 - (rect.y / h) * 2.0;
    let y1 = 1.0 - ((rect.y + rect.h) / h) * 2.0;
    [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
}

#[derive(Clone, Debug)]
struct CachedTexture {
    generation: u64,
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl CachedTexture {
    fn from_scene(texture: &SceneTexture) -> anyhow::Result<Self> {
        if texture.width == 0 || texture.height == 0 {
            anyhow::bail!("texture {:?} has an empty size", texture.id);
        }
        let expected = texture.width as usize * texture.height as usize * 4;
        if texture.pixels.len() != expected {
            anyhow::bail!(
                "texture {:?} has {} bytes, expected {}",
                texture.id,
                texture.pixels.len(),
                expected
            );
        }
        match texture.format {
            SceneTextureFormat::Rgba8 => {}
        }
        Ok(Self {
            generation: texture.generation,
            width: texture.width,
            height: texture.height,
            pixels: texture.pixels.clone(),
        })
    }
}

fn scene_clear_color(scene: &FrameScene, fallback: wgpu::Color) -> wgpu::Color {
    let [r, g, b, a] = scene.clear_color;
    if [r, g, b, a].iter().all(|v| v.is_finite()) {
        wgpu::Color { r, g, b, a }
    } else {
        fallback
    }
}

fn pal_debug_enabled() -> bool {
    std::env::var("PAL_DEBUG")
        .ok()
        .as_deref()
        .is_some_and(|v| v == "1")
}

fn draw_sprite(
    textures: &HashMap<SceneTextureId, CachedTexture>,
    dst: &mut [u32],
    width: usize,
    height: usize,
    coord_scale: [f32; 2],
    sprite: &SpriteDraw,
) {
    if !sprite.dst.is_drawable() || !sprite.src.is_drawable() {
        return;
    }
    let Some(texture) = textures.get(&sprite.texture_id) else {
        log::warn!(
            "skipping sprite with missing texture {:?}; no diagnostic fallback drawn",
            sprite.texture_id
        );
        return;
    };
    draw_textured_rect(dst, width, height, texture, coord_scale, sprite);
}

fn draw_textured_rect(
    dst: &mut [u32],
    width: usize,
    height: usize,
    texture: &CachedTexture,
    coord_scale: [f32; 2],
    sprite: &SpriteDraw,
) {
    let dst_rect = RectF::new(
        sprite.dst.x * coord_scale[0],
        sprite.dst.y * coord_scale[1],
        sprite.dst.w * coord_scale[0],
        sprite.dst.h * coord_scale[1],
    );
    let x0 = dst_rect.x.floor().max(0.0) as i32;
    let y0 = dst_rect.y.floor().max(0.0) as i32;
    let x1 = (dst_rect.x + dst_rect.w).ceil().min(width as f32) as i32;
    let y1 = (dst_rect.y + dst_rect.h).ceil().min(height as f32) as i32;
    if x0 >= x1 || y0 >= y1 {
        return;
    }

    let src_x = sprite.src.x * texture.width as f32;
    let src_y = sprite.src.y * texture.height as f32;
    let src_w = sprite.src.w * texture.width as f32;
    let src_h = sprite.src.h * texture.height as f32;
    let smooth_upscale =
        sprite.smooth_upscale && (dst_rect.w > src_w * 1.1 || dst_rect.h > src_h * 1.1);
    let source_bounds = [
        sprite.source_rect[0].clamp(0, texture.width as i32 - 1),
        sprite.source_rect[1].clamp(0, texture.height as i32 - 1),
        sprite.source_rect[2]
            .saturating_sub(1)
            .clamp(0, texture.width as i32 - 1),
        sprite.source_rect[3]
            .saturating_sub(1)
            .clamp(0, texture.height as i32 - 1),
    ];
    if smooth_upscale
        && (source_bounds[0] > source_bounds[2] || source_bounds[1] > source_bounds[3])
    {
        return;
    }
    let tint = sprite.color;
    for y in y0..y1 {
        let v = ((y as f32 + 0.5 - dst_rect.y) / dst_rect.h).clamp(0.0, 1.0);
        for x in x0..x1 {
            let u = ((x as f32 + 0.5 - dst_rect.x) / dst_rect.w).clamp(0.0, 1.0);
            let sample = if smooth_upscale {
                sample_bilinear(
                    texture,
                    src_x + u * src_w - 0.5,
                    src_y + v * src_h - 0.5,
                    source_bounds,
                )
            } else {
                let sx = (src_x + u * src_w)
                    .floor()
                    .clamp(0.0, texture.width.saturating_sub(1) as f32)
                    as usize;
                let sy = (src_y + v * src_h)
                    .floor()
                    .clamp(0.0, texture.height.saturating_sub(1) as f32)
                    as usize;
                let src_index = (sy * texture.width as usize + sx) * 4;
                [
                    texture.pixels[src_index],
                    texture.pixels[src_index + 1],
                    texture.pixels[src_index + 2],
                    texture.pixels[src_index + 3],
                ]
            };
            let r = (sample[0] as f32 * tint[0].clamp(0.0, 1.0)) as u8;
            let g = (sample[1] as f32 * tint[1].clamp(0.0, 1.0)) as u8;
            let b = (sample[2] as f32 * tint[2].clamp(0.0, 1.0)) as u8;
            let a = (sample[3] as f32 * tint[3].clamp(0.0, 1.0)) as u8;
            if a == 0 {
                continue;
            }
            let dst_index = y as usize * width + x as usize;
            dst[dst_index] = blend_over(dst[dst_index], r, g, b, a);
        }
    }
}

/// Bilinear UI sampling in premultiplied alpha avoids dark fringes around
/// outlined glyphs and transparent button art. Bounds keep neighboring button
/// animation cells out of the interpolation footprint.
fn sample_bilinear(texture: &CachedTexture, x: f32, y: f32, bounds: [i32; 4]) -> [u8; 4] {
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let mut alpha = 0.0_f32;
    let mut premul = [0.0_f32; 3];
    for (ix, wx) in [(x0 as i32, 1.0 - fx), (x0 as i32 + 1, fx)] {
        for (iy, wy) in [(y0 as i32, 1.0 - fy), (y0 as i32 + 1, fy)] {
            let sx = ix.clamp(bounds[0], bounds[2]) as usize;
            let sy = iy.clamp(bounds[1], bounds[3]) as usize;
            let idx = (sy * texture.width as usize + sx) * 4;
            let weight = wx * wy;
            let a = texture.pixels[idx + 3] as f32 * weight;
            alpha += a;
            for (channel, value) in premul.iter_mut().enumerate() {
                *value += texture.pixels[idx + channel] as f32 * a;
            }
        }
    }
    if alpha <= 0.0 {
        return [0; 4];
    }
    [
        (premul[0] / alpha).round().clamp(0.0, 255.0) as u8,
        (premul[1] / alpha).round().clamp(0.0, 255.0) as u8,
        (premul[2] / alpha).round().clamp(0.0, 255.0) as u8,
        alpha.round().clamp(0.0, 255.0) as u8,
    ]
}

fn draw_solid_quad(
    dst: &mut [u32],
    width: usize,
    height: usize,
    coord_scale: [f32; 2],
    quad: SolidQuad,
) {
    if !quad.dst.is_drawable() {
        return;
    }
    let x0 = (quad.dst.x * coord_scale[0]).floor().max(0.0) as i32;
    let y0 = (quad.dst.y * coord_scale[1]).floor().max(0.0) as i32;
    let x1 = ((quad.dst.x + quad.dst.w) * coord_scale[0])
        .ceil()
        .min(width as f32) as i32;
    let y1 = ((quad.dst.y + quad.dst.h) * coord_scale[1])
        .ceil()
        .min(height as f32) as i32;
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let r = float_channel(quad.color[0]);
    let g = float_channel(quad.color[1]);
    let b = float_channel(quad.color[2]);
    let a = float_channel(quad.color[3]);
    for y in y0..y1 {
        for x in x0..x1 {
            let index = y as usize * width + x as usize;
            dst[index] = blend_over(dst[index], r, g, b, a);
        }
    }
}

fn blend_over(dst: u32, src_r: u8, src_g: u8, src_b: u8, src_a: u8) -> u32 {
    if src_a == 255 {
        return pack_rgb(src_r, src_g, src_b);
    }
    let inv_a = 255u32.saturating_sub(u32::from(src_a));
    let dst_r = (dst >> 16) & 0xFF;
    let dst_g = (dst >> 8) & 0xFF;
    let dst_b = dst & 0xFF;
    let r = (u32::from(src_r) * u32::from(src_a) + dst_r * inv_a + 127) / 255;
    let g = (u32::from(src_g) * u32::from(src_a) + dst_g * inv_a + 127) / 255;
    let b = (u32::from(src_b) * u32::from(src_a) + dst_b * inv_a + 127) / 255;
    pack_rgb(r as u8, g as u8, b as u8)
}

fn color_to_rgb(color: wgpu::Color) -> u32 {
    pack_rgb(
        float_channel(color.r as f32),
        float_channel(color.g as f32),
        float_channel(color.b as f32),
    )
}

fn float_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

fn nonzero_size(size: PhysicalSize<u32>) -> PhysicalSize<u32> {
    PhysicalSize::new(size.width.max(1), size.height.max(1))
}

fn nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value was clamped to non-zero")
}

fn softbuffer_error(err: softbuffer::SoftBufferError) -> anyhow::Error {
    anyhow::anyhow!("{err:?}")
}

fn write_ppm(path: &str, pixels: &[u32], width: usize, height: usize) -> anyhow::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "P6\n{} {}\n255", width, height)?;
    for pixel in pixels {
        file.write_all(&[
            ((pixel >> 16) & 0xFF) as u8,
            ((pixel >> 8) & 0xFF) as u8,
            (pixel & 0xFF) as u8,
        ])?;
    }
    Ok(())
}

fn write_surface_png(
    path: &Path,
    pixels: &[u32],
    width: usize,
    height: usize,
) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    let mut rgba = Vec::with_capacity(width.saturating_mul(height).saturating_mul(4));
    for pixel in pixels {
        rgba.push(((pixel >> 16) & 0xFF) as u8);
        rgba.push(((pixel >> 8) & 0xFF) as u8);
        rgba.push((pixel & 0xFF) as u8);
        rgba.push(0xFF);
    }
    writer.write_image_data(&rgba)?;
    Ok(())
}

fn write_rgba_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width.max(1), height.max(1));
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sprite(smooth_upscale: bool) -> SpriteDraw {
        SpriteDraw {
            texture_id: SceneTextureId(1),
            smooth_upscale,
            priority: 0,
            dst: RectF::new(0.0, 0.0, 2.0, 1.0),
            src: RectF::new(0.0, 0.0, 1.0, 1.0),
            source_rect: [0, 0, 2, 1],
            texture_size: [2, 1],
            cell_size: [2, 1],
            position: [0.0; 3],
            offset: [0; 2],
            color: [1.0; 4],
            scale: 1.0,
            rotation: [0.0; 3],
            center_offset: [0.0; 2],
            render_mode: 0,
        }
    }

    #[test]
    fn hidpi_ui_pixels_are_interpolated_without_repeating_pairs() {
        let texture = CachedTexture {
            generation: 1,
            width: 2,
            height: 1,
            pixels: Arc::from([255, 0, 0, 255, 0, 0, 255, 255]),
        };
        let mut smooth = [0; 4];
        draw_textured_rect(&mut smooth, 4, 1, &texture, [2.0, 1.0], &test_sprite(true));
        assert_eq!(smooth[0], 0xFF0000);
        assert_eq!(smooth[3], 0x0000FF);
        assert_ne!(smooth[0], smooth[1]);
        assert_ne!(smooth[2], smooth[3]);

        let mut nearest = [0; 4];
        draw_textured_rect(
            &mut nearest,
            4,
            1,
            &texture,
            [2.0, 1.0],
            &test_sprite(false),
        );
        assert_eq!(nearest, [0xFF0000, 0xFF0000, 0x0000FF, 0x0000FF]);
    }

    #[test]
    fn hidpi_ui_sampling_avoids_transparent_color_bleed_and_adjacent_cells() {
        let texture = CachedTexture {
            generation: 1,
            width: 2,
            height: 1,
            pixels: Arc::from([255, 0, 0, 255, 0, 0, 255, 0]),
        };
        let middle = sample_bilinear(&texture, 0.5, 0.0, [0, 0, 1, 0]);
        assert_eq!(&middle[..3], &[255, 0, 0]);
        assert!(middle[3] > 0 && middle[3] < 255);

        let clipped = sample_bilinear(&texture, 0.75, 0.0, [0, 0, 0, 0]);
        assert_eq!(clipped, [255, 0, 0, 255]);
    }
}
