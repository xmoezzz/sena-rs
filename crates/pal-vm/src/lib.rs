//! Main PAL engine crate.
//!
//! This crate is the integration layer above `pal-asset` and `pal-script`.
//! It owns the window/event loop boundary, the wgpu renderer, and the VM runtime.

pub mod animation;
pub mod app;
pub mod assets;
pub mod audio;
pub mod config;
pub mod debug;
pub mod effect;
pub mod engine;
pub mod event;
pub mod ffi;
pub mod font;
pub mod image;
pub mod input;
pub mod list;
pub mod memory;
pub mod msprite;
pub mod platform_time;
pub mod renderer;
pub mod runtime;
mod save_format;
pub mod scene;
pub mod sprite;
pub mod system;
pub mod task;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod wasm_entry;

pub use animation::{
    AnimationFrameRecord, AnimationHandle, PalAnimationFrameRecord, PalSequenceAnimationDesc,
    PalSheetAnimationDesc, SequenceAnimationDesc, SheetAnimationAxis, SheetAnimationDesc,
};
pub use app::{
    run_sena, run_sena_headless, DiagnosticAutoAdvance, DiagnosticClick,
    DiagnosticClickWhenHitEnabled, DiagnosticKeyEvent, DiagnosticPngAt, SenaConfig,
};
pub use assets::{CoreAssets, GraphicIndex};
pub use audio::{AudioConfig, AudioHandle, AudioSystem, PalSoundGroup, PalSoundStatus, PalVolume};
pub use config::{
    load_system_ini, parse_ini_nls, ConfigDat, EngineStartupConfig, IniFile, IniSection, IniValue,
    SystemDat,
};
pub use debug::{
    collect_frame_dump, pal_debug_enabled, DrawCmdDumpEntry, FrameDebugDump, RenderDumpEntry,
    SpriteDumpEntry,
};
pub use effect::{PalEffectState, PalEffectSystem};
pub use engine::{Engine, EngineConfig, EngineFrame, FrameTiming, TraceConfig};
pub use event::{InputEvent, PalEvent};
pub use font::{PalFontFallback, PalFontSystem};
pub use input::{PalInputState, PalKey, PalMouseButton};
pub use list::{PalListHandle, PalListSystem};
pub use memory::{PalMemoryHandle, PalMemorySystem};
pub use msprite::{
    MSpriteFrameUpdate, MSpriteHandle, MSpriteSystem, MoviePlayback, MSPRITE_STATE_FINISHED,
    MSPRITE_STATE_LOCKED, MSPRITE_STATE_LOOP, MSPRITE_STATE_PLAYING,
};
pub use renderer::{
    pal_device_rect_to_clip_quad, scaled_rect, sprite_transform_debug, RenderOutcome,
    RenderTargetMetrics, Renderer, RendererConfig, ShaderProgram, SpriteTransformDebug,
};
pub use runtime::{
    FrameEvent, RuntimeError, RuntimeStatus, RuntimeTick, ScriptRuntime, ScriptRuntimeConfig,
    WaitRequest,
};
pub use scene::{
    DrawCommand, FrameScene, RectF, SceneTexture, SceneTextureFormat, SceneTextureId, SolidQuad,
    SpriteDraw,
};
pub use sprite::{
    MSpriteDecoderHandle, PalAnimationAxis, PalAnimationFlags, PalColor, PalPoint2, PalPoint3,
    PalRect, PalRenderMode, PalSize, PalSprite, PalSpriteInfo, PalVec3, RenderNode, RenderNodeId,
    SpriteDesc, SpriteFxEffect, SpriteFxHandle, SpriteHandle, SpriteKind, SpriteOptionAux,
    SpriteSurface, SpriteSurfaceId, SpriteSystem, SpriteTransition, SpriteTransitionHandle,
};
pub use system::{
    PalGesture, PalRandomState, PalRectI, PalSystemPathKind, PalSystemPaths, PalSystemState,
    PalWindowRequest,
};
pub use task::{
    AnimationHandle as TaskAnimHandle, TaskDumpEntry, TaskHandle, TaskSystem, BLOCKING_FLAG,
    TASK_POOL_CAPACITY,
};
