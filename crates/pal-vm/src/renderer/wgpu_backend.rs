//! GPU backend for the PAL scene renderer.
//!
//! The scene graph (`FrameScene`) is backend-neutral: sprites are axis-aligned
//! textured quads with a tint and an optional smooth-upscale hint, plus solid
//! color quads. This backend maps each `SceneTexture` to an RGBA8 texture
//! (re-uploaded only when its generation changes) and emits one draw call per
//! sprite, mirroring the software compositor's source-over blending.

use std::collections::HashMap;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::scene::{DrawCommand, FrameScene, SceneTexture, SceneTextureId, SpriteDraw};

use super::{pal_device_rect_to_clip_quad, scaled_rect, RenderOutcome};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl SpriteVertex {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 2,
            },
        ],
    };
}

const SPRITE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@group(0) @binding(0) var sprite_texture: texture_2d<f32>;
@group(0) @binding(1) var sprite_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(sprite_texture, sprite_sampler, in.uv);
    return vec4<f32>(texel.rgb * in.color.rgb, texel.a * in.color.a);
}
"#;

struct GpuTexture {
    generation: u64,
    width: u32,
    height: u32,
    bind_nearest: wgpu::BindGroup,
    bind_linear: wgpu::BindGroup,
}

#[derive(Clone, Copy)]
enum SpanBinding {
    Nearest(SceneTextureId),
    Linear(SceneTextureId),
    White,
}

struct DrawSpan {
    binding: SpanBinding,
    first_vertex: u32,
}

pub struct WgpuBackend {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    sampler_nearest: wgpu::Sampler,
    sampler_linear: wgpu::Sampler,
    texture_bind_layout: wgpu::BindGroupLayout,
    white_bind_nearest: wgpu::BindGroup,
    textures: HashMap<SceneTextureId, GpuTexture>,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: u64,
}

impl WgpuBackend {
    pub async fn new(window: Arc<Window>, size: PhysicalSize<u32>) -> anyhow::Result<Self> {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|err| anyhow::anyhow!("no compatible GPU adapter: {err}"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|err| anyhow::anyhow!("GPU device request failed: {err}"))?;
        let capabilities = surface.get_capabilities(&adapter);
        // Prefer non-sRGB formats so blending matches the software compositor
        // (which blends the already-sRGB-encoded scene values directly).
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| {
                matches!(
                    format,
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                )
            })
            .or_else(|| {
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(|format| !format.is_srgb())
            })
            .unwrap_or(capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("pal nearest sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("pal linear sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("pal sprite texture layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pal sprite pipeline layout"),
            bind_group_layouts: &[Some(&texture_bind_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pal sprite shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pal sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[SpriteVertex::LAYOUT],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let white_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pal solid quad texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &white_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let white_view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_bind_nearest = Self::bind_texture(
            &device,
            &texture_bind_layout,
            &white_view,
            &sampler_nearest,
        );

        let vertex_capacity = 4096_u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pal sprite vertex buffer"),
            size: vertex_capacity * std::mem::size_of::<SpriteVertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        log::info!(
            "wgpu renderer initialized: adapter={:?} format={format:?} size={}x{}",
            adapter.get_info().name,
            config.width,
            config.height
        );
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            sampler_nearest,
            sampler_linear,
            texture_bind_layout,
            white_bind_nearest,
            textures: HashMap::new(),
            vertex_buffer,
            vertex_capacity,
        })
    }

    fn bind_texture(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pal sprite texture bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let width = size.width.max(1);
        let height = size.height.max(1);
        if width == self.config.width && height == self.config.height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    fn sync_textures(&mut self, scene: &FrameScene) {
        self.textures.retain(|texture_id, _| {
            scene
                .textures
                .iter()
                .any(|texture| texture.id == *texture_id)
        });
        for texture in &scene.textures {
            let needs_upload = self.textures.get(&texture.id).is_none_or(|cached| {
                cached.generation != texture.generation
                    || cached.width != texture.width
                    || cached.height != texture.height
            });
            if !needs_upload {
                continue;
            }
            match self.upload_texture(texture) {
                Ok(gpu_texture) => {
                    self.textures.insert(texture.id, gpu_texture);
                }
                Err(err) => {
                    log::error!("failed to upload scene texture {:?}: {err}", texture.id);
                }
            }
        }
    }

    fn upload_texture(&self, texture: &SceneTexture) -> anyhow::Result<GpuTexture> {
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
        let size = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };
        let gpu_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("pal scene texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(texture.width * 4),
                rows_per_image: Some(texture.height),
            },
            size,
        );
        let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok(GpuTexture {
            generation: texture.generation,
            width: texture.width,
            height: texture.height,
            bind_nearest: Self::bind_texture(
                &self.device,
                &self.texture_bind_layout,
                &view,
                &self.sampler_nearest,
            ),
            bind_linear: Self::bind_texture(
                &self.device,
                &self.texture_bind_layout,
                &view,
                &self.sampler_linear,
            ),
        })
    }

    pub fn render(
        &mut self,
        scene: &FrameScene,
        fallback_clear: wgpu::Color,
    ) -> anyhow::Result<RenderOutcome> {
        self.sync_textures(scene);
        let scale = [
            self.config.width as f32 / scene.logical_width.max(1) as f32,
            self.config.height as f32 / scene.logical_height.max(1) as f32,
        ];
        let surface_size = [self.config.width, self.config.height];
        let mut vertices: Vec<SpriteVertex> = Vec::with_capacity(scene.commands.len() * 6);
        let mut spans: Vec<DrawSpan> = Vec::with_capacity(scene.commands.len());
        for command in &scene.commands {
            match command {
                DrawCommand::Sprite(sprite) => {
                    self.push_sprite(&mut vertices, &mut spans, scale, surface_size, sprite);
                }
                DrawCommand::SolidQuad(quad) => {
                    if !quad.dst.is_drawable() {
                        continue;
                    }
                    let dst_device = scaled_rect(quad.dst, scale);
                    push_quad_vertices(
                        &mut vertices,
                        &mut spans,
                        SpanBinding::White,
                        dst_device,
                        surface_size,
                        [0.0, 0.0, 1.0, 1.0],
                        quad.color,
                    );
                }
            }
        }
        if vertices.is_empty() {
            // Still present the clear color so window flashes are visible.
        }
        let required = vertices.len() as u64;
        if required > self.vertex_capacity {
            self.vertex_capacity = required.next_power_of_two();
            self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("pal sprite vertex buffer"),
                size: self.vertex_capacity * std::mem::size_of::<SpriteVertex>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if required > 0 {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.reconfigure();
                return Ok(RenderOutcome::Reconfigured);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(RenderOutcome::Skipped);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                anyhow::bail!("surface validation error while acquiring the next frame");
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let clear = scene_clear_color(scene, fallback_clear);
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("pal frame encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pal frame pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            for span in &spans {
                let bind_group = match span.binding {
                    SpanBinding::Nearest(texture_id) => self
                        .textures
                        .get(&texture_id)
                        .map(|texture| &texture.bind_nearest),
                    SpanBinding::Linear(texture_id) => self
                        .textures
                        .get(&texture_id)
                        .map(|texture| &texture.bind_linear),
                    SpanBinding::White => Some(&self.white_bind_nearest),
                };
                let Some(bind_group) = bind_group else {
                    continue;
                };
                pass.set_bind_group(0, bind_group, &[]);
                pass.draw(span.first_vertex..span.first_vertex + 6, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(RenderOutcome::Rendered)
    }

    fn push_sprite(
        &self,
        vertices: &mut Vec<SpriteVertex>,
        spans: &mut Vec<DrawSpan>,
        scale: [f32; 2],
        surface_size: [u32; 2],
        sprite: &SpriteDraw,
    ) {
        if !sprite.dst.is_drawable() || !sprite.src.is_drawable() {
            return;
        }
        let Some(texture) = self.textures.get(&sprite.texture_id) else {
            log::warn!(
                "skipping sprite with missing texture {:?}; no diagnostic fallback drawn",
                sprite.texture_id
            );
            return;
        };
        let dst_device = scaled_rect(sprite.dst, scale);
        let src_w = sprite.src.w * texture.width as f32;
        let src_h = sprite.src.h * texture.height as f32;
        // Match the software compositor: smooth sampling only kicks in when
        // the sprite is actually upscaled past its native texel density.
        let smooth = sprite.smooth_upscale
            && (dst_device.w > src_w * 1.1 || dst_device.h > src_h * 1.1);
        let mut uvs = [sprite.src.x, sprite.src.y, sprite.src.w, sprite.src.h];
        if smooth {
            // Half-texel inset keeps linear sampling inside the source cell so
            // neighboring button animation cells do not bleed in.
            let inset_u = (0.5 / texture.width as f32).min(sprite.src.w * 0.25);
            let inset_v = (0.5 / texture.height as f32).min(sprite.src.h * 0.25);
            uvs[0] += inset_u;
            uvs[1] += inset_v;
            uvs[2] -= inset_u * 2.0;
            uvs[3] -= inset_v * 2.0;
        }
        let binding = if smooth {
            SpanBinding::Linear(sprite.texture_id)
        } else {
            SpanBinding::Nearest(sprite.texture_id)
        };
        push_quad_vertices(
            vertices,
            spans,
            binding,
            dst_device,
            surface_size,
            uvs,
            sprite.color,
        );
    }
}

fn push_quad_vertices(
    vertices: &mut Vec<SpriteVertex>,
    spans: &mut Vec<DrawSpan>,
    binding: SpanBinding,
    dst_device: crate::scene::RectF,
    surface_size: [u32; 2],
    uvs: [f32; 4],
    color: [f32; 4],
) {
    let quad = pal_device_rect_to_clip_quad(dst_device, surface_size);
    let color = [
        color[0].clamp(0.0, 1.0),
        color[1].clamp(0.0, 1.0),
        color[2].clamp(0.0, 1.0),
        color[3].clamp(0.0, 1.0),
    ];
    let (u0, v0) = (uvs[0], uvs[1]);
    let (u1, v1) = (uvs[0] + uvs[2], uvs[1] + uvs[3]);
    let corners = [
        (quad[0], [u0, v0]),
        (quad[1], [u1, v0]),
        (quad[2], [u1, v1]),
        (quad[3], [u0, v1]),
    ];
    spans.push(DrawSpan {
        binding,
        first_vertex: vertices.len() as u32,
    });
    for index in [0_usize, 1, 2, 0, 2, 3] {
        let (position, uv) = corners[index];
        vertices.push(SpriteVertex {
            position,
            uv,
            color,
        });
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
