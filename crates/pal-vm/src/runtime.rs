use std::sync::Arc;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use pal_asset::{AssetSource, LoadedAsset, Nls, ResourceManager};
use pal_script::extsig::{lookup_registered_sig, lookup_sig, observed_pop_count};
use pal_script::opcodes::{ext_opcode, primary_opcode};
use pal_script::{Operand, OperandKind, PointTable, ScriptImage};

use crate::assets::{CoreAssets, FileDatLayout};
use crate::audio::{
    audio_lookup_key, decode_game_audio, parse_bgm_csv, parse_mix_playlist, AudioHandle, AudioSystem, BgmLoop,
    PalSoundGroup, PalVolume,
};
use crate::config::{
    ini_graphics_size, parse_system_ini, IniFile, IniValue, TextLang, TEXT_FIELD_SEPARATOR,
};
use crate::effect::PalEffectSystem;
use crate::font::{PalFontProfile, PalFontSystem};
use crate::image::{decode_image, decode_image_with_resolver, DecodedImage};
use crate::input::{PalInputState, PalKey, PalMouseButton};
use crate::msprite::{MSpriteHandle, MSpriteSystem, MSPRITE_STATE_FINISHED};
use crate::save_format::{
    composite_thumbnail, decode_original_save, encode_original_save, mosaic_rgba,
    original_prefix_len, original_save_filename, original_save_path, read_lock_dword,
    read_original_text_value, OriginalSavePrefix, ThumbnailSprite, DEFAULT_THUMB_HEIGHT,
    DEFAULT_THUMB_WIDTH, HEADER_BEFORE_PIXELS, LOAD_THUMBNAIL_CAPTURE_SENTINEL, MOSAIC_FACTOR,
};
use crate::scene::{FrameScene, SceneTextureId, SolidQuad};
use crate::sprite::{
    PalAnimationFlags, PalColor, PalPoint2, PalRect, PalRenderMode, PalVec3, SpriteDesc,
    SpriteHandle, SpriteKind, SpriteSurface, SpriteSystem, SpriteTransitionHandle,
};
use crate::system::PalRandomState;
use crate::system::PalSystemState;
use crate::task::{TaskHandle, TaskSystem};
use crate::PalSheetAnimationDesc;

mod actions;
use actions::ActionSubsystemState;
mod history_query;
mod host_queries;
mod text_animation;
use text_animation::{TextAnimationMode, TextThresholdMap};
mod registered_system;
mod registered_audio;
mod registered_images;
mod children;
use registered_system::SePrefixInfo;
mod input_control;
mod movies;
use movies::FullMovieState;
use input_control::InputControlState;
use history_query::FrozenHistoryText;
mod bgv;
use bgv::{BgvState, SavedBgvTrack};
mod native_helpers;
mod native_save;
mod primary_aliases;
use primary_aliases::primary_extcall;
mod registered_primary_aliases;
#[cfg(test)]
mod registered_tests;

fn primary_extcall_for_profile(opcode: u16, registered: bool) -> Option<(u16, u16)> {
    if registered {
        registered_primary_aliases::registered_primary_extcall(opcode)
    } else {
        primary_extcall(opcode)
    }
}
mod select;
mod sprite_surface;
mod font_resource;
#[cfg(test)]
mod button_mode_tests;
mod buttons;
mod button_extensions;
mod history_icons;
mod ui_snapshot;
mod host_checkpoint;
mod history_navigation;
use buttons::{ButtonSounds, PendingButtonReaction};
mod tips;
use tips::{SavedTipsState, TipsState};
use select::SelectSubsystemState;

const DEFAULT_VAR_COUNT: usize = 0x10000;
const DEFAULT_STACK_LIMIT: usize = 0x10000;
/// Size of user_mem, system_mem, and temp_mem arrays (matches original engine).
const DEFAULT_MEM_SIZE: usize = 0x10000;
/// Maximum per-frame events recorded for PAL_DEBUG dump.
const MAX_FRAME_EVENTS: usize = 64;

fn debug_vm_enabled() -> bool {
    std::env::var("DEBUG_VM")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

fn debug_memdat_write_enabled() -> bool {
    std::env::var("DEBUG_MEMDAT_WRITE")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

fn debug_vm_pc_bound(name: &str) -> Option<u32> {
    let value = std::env::var(name).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<u32>().ok()
    }
}

fn pal_boot_random_seed() -> u32 {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    { (js_sys::Date::now() / 1000.0) as u64 as u32 }
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as u32 }
}

fn clamp_percent(value: i32) -> i32 {
    value.clamp(0, 100)
}

fn percent_to_volume(value: i32) -> PalVolume {
    PalVolume::from_raw(clamp_percent(value).saturating_mul(100))
}

fn volume_to_percent(volume: PalVolume) -> i32 {
    (volume.raw() / 100).clamp(0, 100)
}

fn pal_scale_to_factor(raw_scale: i32) -> f32 {
    (raw_scale as f32 / 100.0).max(0.0)
}

fn factor_to_pal_scale(scale: f32) -> i32 {
    (scale * 100.0).round() as i32
}

fn decode_pal_sprite_slot(slot: i32) -> Option<(i32, Option<u8>)> {
    if (0..10_000).contains(&slot) {
        return Some((slot, None));
    }
    let raw = slot as u32;
    let layer = (raw >> 24) as u8;
    match layer {
        // PAL script frequently addresses engine-owned sprite layers as
        // 0x010000NN / 0x020000NN.  Game-side wrappers pass those values to the
        // same PalSpriteSetColor path as ordinary slots; the low word is the
        // sprite index, while the high byte selects a native layer namespace.
        // The portable renderer currently stores all script-visible sprite
        // indices in one table, so resolve the tagged slot to its low index and
        // preserve the layer only for tracing.
        1 | 2 => {
            let index = (raw & 0x0000_FFFF) as i32;
            if (0..10_000).contains(&index) {
                Some((index, Some(layer)))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn apply_graphic_record_lanes(
    desc: &mut SpriteDesc,
    record: &crate::assets::GraphicRecord,
    fade_replace: bool,
) {
    if record.priority_lane != 0 {
        desc.base_priority = desc
            .base_priority
            .saturating_sub(record.priority_lane.saturating_mul(134));
    }
    if record.offset_x != 0 || record.offset_y != 0 {
        desc.position.x += record.offset_x as f32;
        desc.position.y += record.offset_y as f32;
    }
    // A zero scale dword with the 0x100000 bit set collapses the sprite. Native
    // files in these installs store 0 there, so only a positive percent overrides.
    if record.scale_percent > 0 && record.scale_percent != 100 {
        desc.scale *= record.scale_percent as f32 / 100.0;
    }
    if !fade_replace && record.flags & 0x80000 != 0 {
        let rgb = (record.alpha as u32) & 0x00FF_FFFF;
        desc.color = PalColor::from_argb(0xFF00_0000 | rgb);
    }
}

// Both Game.exe variants assign a depth from the shared priority cursor when
// each entry is created: ordinary sprites use 0x1387 - slot (Koikake 0x411150,
// TotsuLover 0x42805B), buttons use 0x1302 - index (Koikake 0x40F5C4,
// TotsuLover 0x4251E8). PAL paints smaller native depths in front; our render
// tree paints larger priorities last, so reverse the sign here.
fn game_sprite_priority(slot: i32, cursor: i32) -> i32 {
    slot.saturating_sub(0x1387i32.saturating_add(cursor))
}

fn button_render_priority(index: i32, cursor: i32) -> i32 {
    index.saturating_sub(0x1302i32.saturating_add(cursor))
}

// The PAL text window sits between the two native layers: in front of scene
// sprites (0x1387 - slot) and behind the button layer (0x1302 - index), so
// native depth 0x1330 keeps scene slots below 87 under the window while the
// ADV chrome stays clickable-looking above it.  Only a positive cursor moves
// the window: negative popup lanes must stay above the text even if a reveal
// recomposes while a menu is open.
fn adv_text_render_priority(cursor: i32) -> i32 {
    0i32.saturating_sub(0x1330).saturating_sub(cursor.max(0))
}

// The backlog overlay is drawn while the log screen is open, and the script
// has already moved the priority cursor down for that screen (Koikake opens
// it at cursor -134, where the background sprite sits at depth 4801 and the
// screen buttons at 4710..4712).  Unlike the ADV window the overlay must
// follow negative lanes, or the log background covers the backlog text.
fn history_text_render_priority(cursor: i32) -> i32 {
    0i32.saturating_sub(0x1330)
        .saturating_sub(cursor)
        .saturating_add(1)
}

// Kept for tests that need a neutral priority above the synthetic bands.
#[cfg(test)]
const BUTTON_RENDER_PRIORITY: i32 = 100;

#[derive(Clone, Debug)]
pub enum FrameEvent {
    ExtCallSkipped {
        pc: u32,
        category: u16,
        index: u16,
        name: Option<String>,
    },
    WaitEmitted {
        pc: u32,
        kind: WaitRequest,
    },
    UnsupportedCmd {
        pc: u32,
        opcode: u16,
        name: Option<String>,
    },
    UnsupportedExt {
        pc: u32,
        category: u16,
        index: u16,
        name: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct ScriptRuntimeConfig {
    pub instructions_per_frame: usize,
    pub trace: bool,
}

impl Default for ScriptRuntimeConfig {
    fn default() -> Self {
        Self {
            // Keep each frame finite while still allowing menu/intro scripts
            // to reach their next PAL wait/run submission in one slice.
            instructions_per_frame: 16_384,
            trace: false,
        }
    }
}

/// Outcome returned by extcall dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExtCallOutcome {
    /// Native wrapper completed without writing its destination slot.
    Continue,
    /// Handler ran and produced a return value.
    Value(i32),
    /// Native control return ends this VM slice without an unsupported call.
    Yield { value: Option<i32> },
    /// Handler produced a value and blocked the VM on a PAL wait task.
    Wait { value: i32, request: WaitRequest },
    /// No handler found; caller should write 0 to dst and continue.
    Skip,
    /// Handler cannot proceed; caller should set UnsupportedExtCall status.
    Block,
}

/// Request for a blocking wait task emitted by the script VM.
/// Engine creates the corresponding TaskSystem task after run_frame returns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitRequest {
    /// Wait for N frame-ticks. 1 = one engine frame. -1 = forever.
    Frame(i32),
    /// Wait for N PAL milliseconds using the cached PAL clock.
    Time(u32),
    /// Wait for any key or mouse button push.
    Click,
    /// Wait until either a click/input push arrives or N PAL milliseconds elapse.
    ClickOrTime(u32),
    /// ADV auto-mode hold: like `ClickOrTime`, but the timeout only finishes
    /// the wait after voice playback has also ended (native auto advance
    /// requires the voice channels idle, koikake.exe 0x42F9D0).
    AutoClickOrTime(u32),
    /// Wait for the active ADV text reveal task to finish before allowing the
    /// script to advance. Kept for older traces; current Game.exe evidence shows
    /// text_w/text_wa only update reveal state and the following wait syscall owns
    /// the blocking behavior.
    TextReveal(u32),
    /// Wait for a sprite FX ending phase, or the native cancel input.
    SpriteFx(SpriteHandle),
}

#[derive(Debug)]
pub struct ScriptRuntime {
    pc: u32,
    entry_pc: u32,
    vars: Vec<i32>,
    stack: Vec<i32>,
    argument_stack: Vec<i32>,
    call_stack: Vec<u32>,
    status: RuntimeStatus,
    trace: bool,
    /// Handle to the active blocking wait task created for WaitFrame/WaitClick opcodes.
    /// Engine checks this handle after task_system.process() to detect task completion.
    wait_task_handle: Option<TaskHandle>,
    wait_task_kind: Option<WaitRequest>,
    /// Set only by the ADV `wait_click(-1)` path for the current blocked step.
    adv_wait_checkpoint_pending: bool,
    /// Cached PAL time in milliseconds, injected by Engine once per frame.
    pal_time_ms: u32,
    host_active: bool,
    host_gpu_renderer: bool,
    registered_text_animation: Option<TextAnimationMode>,
    se_prefix_info: [SePrefixInfo; 4],
    registered_se_flags: [i32; 16],
    registered_se_controls: [registered_audio::SeChannelControl; 16],
    registered_se_pan: i32,
    pending_registered_se_wait: Option<i32>,
    host_touch_available: bool,
    observed_fullscreen: bool,
    last_window_mode_change_ms: u32,
    pending_global_fx_release: bool,
    /// Game.exe wait_sync_begin stores PaltimeGetTime() at runtime offset +655248.
    wait_sync_begin_ms: u32,
    /// Game.exe wait_sync_release stores its active duration/start at +655244/+655248.
    wait_sync_release: Option<WaitSyncRelease>,
    /// Conservative model for wait_time_push/wait_time_pop extcalls.
    wait_time_stack: Vec<u32>,
    /// Game.exe category 22 run/run_stack state recovered from offsets around +804292.
    run_pipeline: RunPipelineState,
    /// Game.exe category 18 attach-work-process flag at VM offset +804088.
    /// Native sub_417A50 sets it and posts sub_44A080 through PalAttachWorkProcess;
    /// sub_417A30 clears it. The portable VM runs on one engine thread, so this
    /// flag records the observable script state without spawning a Windows worker.
    work_process_attached: bool,
    /// Portable access/read flags updated by category 18 `update_access` and
    /// `access_clear`. Native stores packed bits in PAL task data and a file
    /// access table; Rust keeps the resolved key/id for menu/save predicates.
    access_updates: BTreeMap<String, u8>,
    action_state: ActionSubsystemState,
    action_audio_requests: Vec<(i32, bool, i32)>,
    effect_system: PalEffectSystem,
    msprite_system: MSpriteSystem,
    full_movie: Option<FullMovieState>,
    movie_volume_type: u32,
    movie_volume_raw: i32,
    movie_muted: bool,
    movie_cancel_fade_enabled: bool,
    /// argument_base: saved/restored by call/ret (opcode 24). kind=0x9 reads/writes this.
    argument_base: i32,
    /// user_mem: kind 0x1 indirect — user_mem[vars[lo]].
    user_mem: Vec<i32>,
    /// system_mem: kind 0x2 indirect — system_mem[vars[lo]].
    system_mem: Vec<i32>,
    /// temp_mem: kind 0x5 indirect — temp_mem[(bank+argument_base)*bank_nonzero + vars[lo]].
    temp_mem: Vec<i32>,
    /// mem_dat_words: writable shadow of Mem.dat as i32 words (for MemDatDirect writes).
    mem_dat_words: Vec<i32>,
    /// Portable model for Game.exe category 9 memory_stack_push/pop. Native
    /// snapshots one 0x4000-byte VM work bank (ctx+0x1500); it does not restore
    /// user_mem, system_mem, or Mem.dat's mutable shadow, which scripts use for
    /// persistent settings and menu page requests such as memdat[158].
    memory_state_stack: Vec<ScriptMemorySnapshot>,
    /// Portable model for category 9 list_stack_push_point/list_stack_pop_count.
    /// Native stores resolved script addresses in a PalList; Rust stores point
    /// ids and resolves them through the script point table when needed.
    list_point_stack: Vec<u32>,
    /// Raw operand word for the current extcall's return destination (a1[184629]).
    extcall_dst_raw: u32,
    extcall_instruction_size: u32,
    input_control: InputControlState,
    /// PAL file handles opened by category 18 file extcalls.
    file_handles: Vec<Option<RuntimeFile>>,
    /// Game script image slots mapped to PAL sprite handles.
    game_sprites: BTreeMap<i32, SpriteHandle>,
    /// Sprite surface copied by `get_backbuffer` (`PalSpriteBackBafferCopy`).
    backbuffer_sprite: Option<SpriteHandle>,
    /// PAL transition handles keyed by script transition slot.
    game_sprite_transitions: BTreeMap<i32, SpriteTransitionHandle>,
    /// Native sprite transition source image lane. Game.exe keeps the previous
    /// PAL sprite pointer in the wrapper while `sp_set_transition` installs the
    /// new image; PAL then blends previous -> current and releases/cancels the
    /// old lane when the transition completes.
    game_sprite_transition_sources: BTreeMap<i32, SpriteHandle>,
    /// PalAnimation tasks attached to Game script image slots.
    game_sprite_animations: BTreeMap<i32, TaskHandle>,
    /// Original placement-mode arguments for script sprites.
    game_sprite_placements: BTreeMap<i32, GameSpritePlacement>,
    /// Native Game wrapper scale lane. `sp_set_scale` stores raw_scale/100 in
    /// Game.exe (`sub_428000`); the portable renderer may project that value to
    /// the configured logical stage, but script queries must still see the
    /// original PAL raw scale.
    game_sprite_native_scales: BTreeMap<i32, i32>,
    /// Native Game wrapper visual lanes before PAL submit/projection.
    ///
    /// Game.exe does not animate already-projected PalSprite coordinates.
    /// `sp_set`, `sp_set_pos_move`, action bytecode, and `sp_set_transition`
    /// mutate wrapper lanes, then `sub_4494D0` sums those lanes and calls
    /// PAL.dll. Keeping this state separate prevents action and transition
    /// updates from permanently baking logical-window coordinates back into the
    /// wrapper.
    game_sprite_wrapper_visuals: BTreeMap<i32, GameSpriteWrapperVisual>,
    /// Parent/lane -> child mapping written by category 3 index 49
    /// `sp_set_child` (`Game.exe` `sub_424A20`).
    game_sprite_child_lanes: BTreeMap<(i32, i32), GameSpriteChildLane>,
    game_sprite_child_offsets: BTreeMap<i32, children::ChildVisualOffsets>,
    game_sprite_render_lanes: BTreeMap<(i32, i32), children::ChildRenderLane>,
    /// Native sprite field at ctx+666396, used by category 3 index 34/35.
    game_sprite_anim_params: BTreeMap<i32, i32>,
    game_sprite_aspect_position_types: BTreeMap<i32, i32>,
    /// Slots with Game wrapper flag 0x01000000 set by category 3 index 44
    /// (`sub_424FD0`). Native Game stores this on wrapper dword +12 and later
    /// submits the sprite through the normal wrapper commit path. We keep it as
    /// per-slot state so clipping/aspect-sensitive sprites are not silently
    /// treated like ordinary unlocked sprites.
    game_sprite_vis_clip_slots: BTreeSet<i32>,
    /// Game.exe face table at VM offsets +710420..+710432. Index 19 writes
    /// this table, index 20 materializes/replaces the part sprite, and index 23
    /// clears it through the mapped sprite slot.
    game_face_slots: BTreeMap<i32, GameFaceSlot>,
    /// Game.exe sprite priority cursor at VM offset +804232, advanced by
    /// category 3 index 7 `set_priority`.
    sprite_priority_cursor: i32,
    /// Named ANI records that target a sprite slot before that slot has a concrete surface.
    game_sprite_pending_named_animations: BTreeMap<i32, PendingNamedSpriteAnimation>,
    /// Alpha actions that targeted a normal script sprite before it had a surface.
    game_sprite_pending_alpha: BTreeMap<i32, Vec<PendingAlphaAction>>,
    /// Base alpha lane written by `sp_set_alpha`/sprite creation.
    ///
    /// IDB evidence: Game.exe `sub_4281D0` writes the public alpha byte into
    /// wrapper color lane +92 and clears the action-temp alpha lane at +96.
    /// `sub_4494D0` later submits `base + temp + final + ...` to
    /// `PalSpriteSetColor`. Keeping the base lane here prevents timed alpha
    /// actions from baking their destination into the sprite before the action
    /// has visually advanced.
    game_sprite_base_alpha: BTreeMap<i32, i32>,
    /// Completed action alpha lane written by `sub_402F30`/`sub_402F00`.
    game_sprite_final_alpha_delta: BTreeMap<i32, i32>,
    /// Running action alpha lanes. Native action processing writes these to
    /// wrapper offset +244 while active and moves the final delta to +420 when
    /// the section completes.
    game_sprite_active_alpha: Vec<PendingSpriteAlphaAction>,
    /// Final wrapper-lane commits for Game action position sections.
    ///
    /// IDB evidence: `sub_403490` writes running deltas to temporary lanes
    /// (+168/+172/+176) and only writes final lanes (+376/+380/+384) when the
    /// section reaches its duration.  Keeping these pending prevents a queued
    /// position action from polluting the base wrapper coordinates before the
    /// tween has visually finished.
    game_sprite_pending_position: Vec<PendingSpritePositionAction>,
    /// Old face/part sprites fading out after a slot replacement.
    retired_sprites: Vec<RetiredSprite>,
    /// Game.exe MSprite wrapper state keyed by script slot.
    game_msprites: BTreeMap<i32, GameMSpriteState>,
    /// Native msp_wait stores the slot state and rewinds PC until PalMSpriteGetState has bit 4.
    pending_msp_wait_slot: Option<i32>,
    /// Game button entries keyed by (button group, entry index).
    game_buttons: BTreeMap<(i32, i32), GameButtonEntry>,
    next_button_registration: u64,
    /// Native `btn_init` group records: normal/hover resource ids plus the
    /// current onmouse index returned by category 8 index 23.
    button_groups: BTreeMap<i32, GameButtonGroup>,
    /// Latched button pushes keyed by group; consumed by btn_get_push(group).
    button_push_queue: BTreeMap<i32, VecDeque<i32>>,
    /// Button whose mouse press is still held down.  PAL buttons capture the
    /// mouse while pressed; slider drag loops poll this through btn_on_check,
    /// which reports -1 once the press ends.
    pressed_button: Option<(i32, i32)>,
    /// Game.exe ctx+12956 / PAL Block+616: react on release when true.
    button_release_mode: bool,
    /// PAL button group+48: hovered button while the left mouse is held.
    button_release_capture: Option<(i32, i32)>,
    button_group_release_capture: BTreeMap<i32, i32>,
    adv_menu_expanded: bool,
    /// Game category 12 system/menu button table.
    system_buttons: BTreeMap<i32, GameSystemButtonEntry>,
    system_button_blocked_at: Option<u32>,
    button_input_block_duration: u32,
    button_run_skip: i32,
    button_children: BTreeMap<(i32, i32), i32>,
    history_navigation: history_navigation::HistoryNavigation,
    button_sounds: BTreeMap<i32, ButtonSounds>,
    retired_button_sounds: Vec<AudioHandle>,
    pending_button_reactions: BTreeMap<i32, PendingButtonReaction>,
    button_sound_hover: BTreeMap<i32, i32>,
    button_callback_hover: BTreeMap<i32, i32>,
    input_registered_profile: bool,
    input_release_shortcuts: bool,
    /// Game script sound slots mapped to PAL audio handles. Key is (script category, slot).
    game_audio: BTreeMap<(u16, i32), AudioHandle>,
    bgv_state: BgvState,
    /// Game.exe audio control fields around ctx[164901..164911].  The native
    /// mute extcalls preserve the configured percent and only gate the value
    /// written into PAL sound groups, so the portable VM keeps both the percent
    /// and mute latch instead of losing the user's slider value.
    master_volume_percent: i32,
    master_muted: bool,
    bgm_volume_percent: i32,
    bgm_muted: bool,
    bgm_auto_volume_percent: i32,
    bgm_auto_muted: bool,
    /// `BGM.CSV` sample loop points, loaded once. `None` means not read yet.
    bgm_loops: Option<BTreeMap<String, BgmLoop>>,
    /// Live category-4 slot state mirrored into save snapshots (version 5).
    bgm_slots: BTreeMap<i32, BgmSlotState>,
    /// Set by restore_save_snapshot; the next run_frame replays the restored
    /// tracks through the audio backend.
    bgm_replay_pending: bool,
    se_volume_percent: BTreeMap<i32, i32>,
    se_enabled: BTreeMap<i32, bool>,
    /// Per-character voice volumes set from the SOUND menu's right-hand unit
    /// grid (category 13 indexes 11/12 take the character slot).
    voice_ex_volume_percent: BTreeMap<i32, i32>,
    /// Per-character voice enable latches from the unit grid's mute checks
    /// (category 13 index 5 takes (slot, enabled, extra); index 6 reads slot).
    voice_ex_enabled: BTreeMap<i32, bool>,
    /// `Sound.csv` voice-header rows in unit-grid slot order, loaded once.
    voice_unit_headers: Option<Vec<String>>,
    se_muted: BTreeMap<i32, bool>,
    /// Native voice_wait stores a wait mask and rewinds PC until the voice checker reports idle.
    pending_voice_wait_slot: Option<i32>,
    font_state: PalFontSystem,
    text_state: TextSubsystemState,
    tips_state: TipsState,
    /// Text skip latch (koikake.exe ADVctx+0x31828, written by skip_set
    /// 0x428E10, read through GetSkipState 0x437800 bit 0).  The ADV text
    /// task consults it every frame while deciding whether to bypass waits.
    text_skip_enabled: bool,
    /// Text auto latch (koikake.exe ADVctx+0x3182C, auto_set 0x428CD0).
    /// When set, the post-reveal ADV text state finishes by timer instead of
    /// by input, gated on voice playback having finished.
    text_auto_enabled: bool,
    /// Scene-skip latch (koikake.exe ADVctx+0x31830), set by the category 9
    /// index 51 `scene_skip` extcall (0x427BA0) and cleared by
    /// `cancel_scene_skip` (0x427B40), select entry, or arrival at unread
    /// text.  While set, native GetSkipState returns -1 so every wait and
    /// reveal behaves as skipped.
    scene_skip_active: bool,
    /// Last frame's evaluated GetSkipState value, cached at the top of
    /// run_frame so extcall handlers that receive no input state (run,
    /// effect_stop, sp_transition) can mirror the native per-frame skip
    /// checks.
    last_skip_state: i32,
    /// Read-text bitmap (koikake.exe ctx+0xC6760): one entry per Text.dat
    /// pool record id.  Native sets the bit as soon as a line is displayed
    /// (0x437920) and consults it (0x437990) in the skip gates and the
    /// scene-skip display skip.  Persisted across sessions like the native
    /// system.dat bitmap.
    read_text_ids: BTreeSet<u32>,
    /// Whether the currently displayed line was already read (native
    /// text+0x858), refreshed by every text submit.  skip_set refuses to
    /// latch while an unread line is on screen in read-only mode.
    current_line_read: bool,
    select_state: SelectSubsystemState,
    save_state: SaveSubsystemState,
    history_state: HistorySubsystemState,
    thread_state: ThreadSubsystemState,
    message_state: MessageSubsystemState,
    /// Dynamic string slots used by startup string-buffer extcalls.
    dynamic_strings: Vec<String>,
    dynamic_string_cursor: usize,
    /// SYSTEM.INI parsed through the selected external NLS.
    system_ini: Option<IniFile>,
    /// Language field read out of multi-language text records.
    text_lang: TextLang,
    /// TrueType faces the release ships as loose files, in path order.
    ///
    /// Kept past boot so a later font-type change (`set_language`, category 9
    /// index 17) can switch the text face without the game root at hand.
    font_face_candidates: Vec<PathBuf>,
    /// The loose face currently loaded as the text face, when one is.
    loaded_face_path: Option<PathBuf>,
    /// Whether this release lets the PAL font type select among its own faces.
    ///
    /// Only a localized release that ships several faces offers the choice;
    /// the original installs keep their bitmap and embedded faces.
    style_faces_enabled: bool,
    system_state: PalSystemState,
    /// PAL random seed ring used by PalRandomEx and Game category 20 random.
    random_state: PalRandomState,
    /// Per-frame events accumulated during run_frame(); cleared at the start of each frame.
    frame_events: Vec<FrameEvent>,
    /// Button callback gosubs to inject at the top of the next script frame
    /// (point IDs, not PCs).  A burst of clicks while a modal gosub is still
    /// running must queue in order rather than overwrite a single slot.
    pending_gosub_points: VecDeque<u32>,
    /// ADV click waits parked while a modal menu gosub runs on top of them.
    modal_wait_suspensions: Vec<ModalWaitSuspension>,
    /// Category 9:23 continuation target.  Unlike button callbacks, this is a
    /// process/menu jump and must not push a return PC or the cleanup routine
    /// returns to the old per-frame wait loop.
    pending_jump_point: Option<u32>,
    /// Game category 9:24 stores the menu transition mode consumed by the
    /// subsequent category 9:23 continuation-point switch.  The script still
    /// owns the selected page state in Mem.dat[158].
    menu_transition_mode: i32,
    /// Game category 9 indices 25/26 use ctx+804244 as a one-word scratch
    /// latch. Index 25 clears it; index 26 copies it to the extcall return
    /// destination.
    system_scratch_value: i32,
    /// Portable mirror of PalDebugWindowGetState/SetState used by Game category
    /// 15:5/15:6.  The original pushes the previous debug-window state into the
    /// extcall destination; even release scripts use it around menu loops.
    debug_window_state: i32,
}

#[derive(Clone, Debug)]
struct RuntimeFile {
    name: String,
    bytes: Vec<u8>,
    cursor: usize,
    table_cursor: usize,
    parsed_table: Option<ParsedFileTable>,
}

#[derive(Clone, Debug)]
struct ParsedFileTable {
    entries: Vec<i32>,
    strings: BTreeMap<i32, String>,
}

#[derive(Clone, Debug)]
struct ScriptMemorySnapshot {
    temp_mem: Vec<i32>,
}

#[derive(Clone, Copy, Debug)]
struct WaitSyncRelease {
    duration_ms: u32,
    start_ms: u32,
}

#[derive(Clone, Debug)]
struct GameButtonEntry {
    registration_order: u64,
    native_flags: u32,
    hover_callbacks: [Option<u32>; 2],
    handle: SpriteHandle,
    name: String,
    /// Script-level visibility from btn_set/btn_show/btn_hide. Group-0 ADV
    /// chrome can be temporarily suppressed by text_hide/history without
    /// changing this native button visibility bit.
    visible: bool,
    enabled: bool,
    locked: bool,
    toggle: i32,
    alpha: u8,
    slider_offset: i32,
    base_position: [i32; 2],
    slider_position: [i32; 2],
    hit_rect: Option<[i32; 4]>,
    /// Callback point registered by btn_set arg[3]; injected as gosub on click.
    gosub_point: Option<u32>,
    /// Latest animation resource bound by Game.exe `btn_set_anim`.
    anim_resource: Option<String>,
    anim_play_flag: i32,
}

#[derive(Clone, Debug, Default)]
struct GameButtonGroup {
    normal_image: i32,
    hover_image: i32,
    onmouse_index: i32,
    offmouse_index: i32,
    push_index: i32,
    slider_ended: bool,
    raw_toggle: i32,
    locked: bool,
    lock_started_ms: u32,
    lock_duration_ms: u32,
}

#[derive(Clone, Debug, Default)]
struct GameSystemButtonEntry {
    image: i32,
    state: i32,
    enabled: bool,

}

/// Outcome of per-frame button/shortcut input routing, telling the engine
/// which input edges were consumed so wait tasks never see them twice.
#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonInputOutcome {
    /// A mouse push edge was consumed (button reaction or mouse-bound
    /// system-button shortcut).
    pub consumed_mouse_push: bool,
    /// A release-mode press was captured, without activating its callback.
    pub consumed_mouse_capture: bool,
    /// A release edge activated a button and must not reach another consumer.
    pub consumed_mouse_pull: bool,
    /// A key or wheel push edge was consumed by a system-button shortcut.
    pub consumed_push_edge: bool,
    /// A hover callback is ready without consuming a physical input edge.
    pub callback_queued: bool,
}

impl ButtonInputOutcome {
    pub fn consumed_any(self) -> bool {
        self.consumed_mouse_push || self.consumed_mouse_pull || self.consumed_push_edge
    }
}

#[derive(Clone, Debug)]
struct PendingNamedSpriteAnimation {
    asset_name: String,
    bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct PendingAlphaAction {
    alpha_delta: i32,
    duration_ms: u32,
    started_ms: u32,
}

#[derive(Clone, Copy, Debug)]
struct PendingSpriteAlphaAction {
    slot: i32,
    handle: SpriteHandle,
    alpha_delta: i32,
    started_ms: u32,
    duration_ms: u32,
}

#[derive(Clone, Copy, Debug)]
struct PendingSpritePositionAction {
    slot: i32,
    handle: SpriteHandle,
    native_dx: f32,
    native_dy: f32,
    native_dz: f32,
    started_ms: u32,
    duration_ms: u32,
}

#[derive(Clone, Copy, Debug)]
struct RetiredSprite {
    slot: i32,
    handle: SpriteHandle,
    release_at_ms: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GameSpritePlacement {
    arg_count: usize,
    raw_x: i32,
    raw_y: i32,
    raw_z: i32,
    width: u32,
    height: u32,
    default_x: i32,
    default_y: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct GameSpriteWrapperVisual {
    native_x: f32,
    native_y: f32,
    native_z: f32,
    raw_scale: i32,
    width: u32,
    height: u32,
    arg_count: usize,
    project_native_draw: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GameSpriteChildLane {
    child_slot: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct GameFaceSlot {
    sprite_slot: i32,
    center_x: i32,
    center_y: i32,
    priority_lane: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RunPipelineState {
    run_stack_enabled: bool,
    pending_kind: i32,
    pending_effect_id: i32,
    pending_arg1: i32,
    pending_arg2: i32,
    effect_active: bool,
    no_wait_latch: bool,
    last_run_time_ms: u32,
    last_run_arg1: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct GameMSpriteState {
    handle: Option<MSpriteHandle>,
    playing: bool,
    locked: bool,
    loop_mode: i32,
    loop_start: i32,
    loop_end: i32,
    last_play: i32,
    finished: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct VoiceAutopanEntry {
    target: i32,
    name_value: i32,
    mode: i32,
    name: String,
}

#[derive(Clone, Debug)]
struct TextSubsystemState {
    initialized: bool,
    visible: bool,
    rect: [i32; 4],
    alpha: i32,
    mode: i32,
    base: i32,
    icon: i32,
    button: i32,
    history_enabled: bool,
    voice_cut_enabled: bool,
    voice_enabled: bool,
    voice_volume: i32,
    voice_muted: bool,
    voice_info: BTreeMap<i32, native_helpers::VoiceInfo>,
    bgv_enabled: bool,
    bgv_volume: i32,
    bgv_muted: bool,
    voice_autopan_enabled: bool,
    voice_autopan_limit: i32,
    voice_autopan_playing: BTreeMap<i32, (AudioHandle, VoiceAutopanEntry)>,
    voice_autopan_entries: BTreeMap<i32, VoiceAutopanEntry>,
    voice_info_enabled: Option<bool>,
    /// Category 2 ADV text owns its color independently from category 9 UI font
    /// state. Title/menu scripts freely change the shared UI font color, while
    /// native message-window rendering keeps dialogue text black unless
    /// TextSetColor changes it.
    text_color: u32,
    text_effect_color: u32,
    last_text_value: i32,
    last_text_args: [i32; 4],
    /// Script offset of the most recent `text` command. The native engine
    /// mirrors this into the save image header (`c20+0x20`, file offset
    /// `0x20C`) on every text command and re-enters the script there on load.
    last_text_pc: u32,
    init_args: [i32; 8],
    last_event_time_ms: u32,
    reveal_start_ms: u32,
    reveal_duration_ms: u32,
    reveal_enabled: bool,
    /// Wait geometry of the current line captured at submit time: the full
    /// typewriter reveal span and the auto-mode post-reveal hold.  A modal
    /// gosub that flips auto/skip mid-line rebuilds the parked wait from
    /// these instead of re-resolving the text.
    current_reveal_ms: u32,
    current_auto_hold_ms: u32,
    sprite: Option<SpriteHandle>,
    name_sprite: Option<SpriteHandle>,
    base_image_value: i32,
    base_image: Option<DecodedImage>,
    /// Slot 255 alpha actions can arrive while text_clear has removed the
    /// concrete text sprites; keep them until the next ADV text surface exists.
    pending_alpha: Vec<PendingAlphaAction>,
    dirty: bool,
    show_wait_mark: bool,
    wait_mark_sheet: Option<DecodedImage>,
    wait_mark_missing: bool,
    /// Fully rasterized ADV panel + body text reused across reveal and
    /// wait-mark frames, so a running reveal clips cached pixels instead of
    /// re-rasterizing every glyph each frame.
    render_cache: Option<AdvTextPanelCache>,
    /// What the current text sprite surface already shows; identical frames
    /// are skipped instead of re-uploading the same pixels.
    presented_frame: Option<AdvTextPresentedFrame>,
}

impl TextSubsystemState {
    /// Source-order `text_init` arguments are pushed as
    /// (mode, name_y, name_x, height, width, body_y, body_x, font_size), but
    /// Game.exe `VmExtcall_TextInit` pops from the stack top and stores them as
    /// ctx+16 font size, ctx+24 body x, ctx+28 body y, ctx+32 width,
    /// ctx+36 height, ctx+40 name x, ctx+44 name y, ctx+20 mode.
    fn init_font_size(&self) -> i32 {
        self.init_args[7]
    }

    fn init_mode(&self) -> i32 {
        self.init_args[0]
    }

    fn init_name_y(&self) -> i32 {
        self.init_args[1]
    }

    fn init_name_x(&self) -> i32 {
        self.init_args[2]
    }

    fn init_text_height(&self) -> i32 {
        self.init_args[3]
    }

    fn init_text_width(&self) -> i32 {
        self.init_args[4]
    }

    fn init_body_y(&self) -> i32 {
        self.init_args[5]
    }

    fn init_body_x(&self) -> i32 {
        self.init_args[6]
    }
}

impl Default for TextSubsystemState {
    fn default() -> Self {
        Self {
            initialized: false,
            visible: false,
            rect: [0; 4],
            alpha: 0,
            mode: 0,
            base: 0,
            icon: 0,
            button: 0,
            history_enabled: true,
            voice_cut_enabled: false,
            voice_enabled: true,
            // VmCtx_Init initializes PAL sound volume fields to 5000; native
            // voice_get_volume returns 100 * field / 10000, so script-visible
            // default volume is 50.
            voice_volume: 50,
            voice_muted: false,
            voice_info: BTreeMap::new(),
            bgv_enabled: true,
            bgv_volume: 50,
            bgv_muted: false,
            voice_autopan_enabled: false,
            voice_autopan_limit: 0,
            voice_autopan_playing: BTreeMap::new(),
            voice_autopan_entries: BTreeMap::new(),
            voice_info_enabled: None,
            text_color: 0xFF00_0000,
            text_effect_color: 0xFFFF_FFFF,
            last_text_value: 0,
            last_text_args: [0; 4],
            last_text_pc: 0,
            init_args: [0; 8],
            last_event_time_ms: 0,
            reveal_start_ms: 0,
            reveal_duration_ms: 0,
            reveal_enabled: false,
            current_reveal_ms: 0,
            current_auto_hold_ms: 0,
            sprite: None,
            name_sprite: None,
            base_image_value: 0,
            base_image: None,
            pending_alpha: Vec::new(),
            dirty: false,
            show_wait_mark: false,
            wait_mark_sheet: None,
            wait_mark_missing: false,
            render_cache: None,
            presented_frame: None,
        }
    }
}

/// Per-line geometry of the cached ADV text block. `char_x` holds the
/// cumulative pixel x of every character boundary (len == char_count + 1) so
/// the smooth reveal can clip inside a glyph instead of snapping per char.
#[derive(Clone, Debug)]
struct AdvTextLineLayout {
    y: u32,
    height: u32,
    char_start: usize,
    char_count: usize,
    char_x: Vec<u32>,
}

#[derive(Clone, Debug)]
struct AdvTextPanelCache {
    /// Window base panel with no body text and no wait mark.
    panel_width: u32,
    panel_height: u32,
    panel_rgba: Vec<u8>,
    /// Fully rasterized body text block (all characters).
    text_width: u32,
    text_rgba: Vec<u8>,
    thresholds: Option<TextThresholdMap>,
    text_origin_x: u32,
    text_origin_y: u32,
    lines: Vec<AdvTextLineLayout>,
    full_char_count: usize,
    sprite_x: i32,
    sprite_y: i32,
}

impl AdvTextPanelCache {
    /// Pixel position right after the last visible character, in text-block
    /// coordinates. This is where the wait mark is anchored.
    fn text_end_position(&self) -> (u32, u32) {
        self.lines
            .iter()
            .rev()
            .find(|line| line.char_count > 0)
            .map(|line| (line.char_x[line.char_count], line.y))
            .unwrap_or((0, 0))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AdvTextPresentedFrame {
    /// `(line index, pixel x limit)` of the smooth reveal wipe. `None` means
    /// the whole body text is visible.
    reveal_limit: Option<(usize, u32)>,
    animation_progress: Option<u32>,
    /// Wait-mark animation frame currently blitted; `None` means no wait mark.
    wait_mark_frame: Option<u32>,
}

#[derive(Clone, Debug, Default)]
struct SaveSubsystemState {
    title: i32,
    title_bytes: Vec<u8>,
    thumbnail_size: [i32; 2],
    thumbnail_pixels: Vec<u8>,
    mosaic_enabled: bool,
    capture_from_screen: bool,
    text_rect: [i32; 4],
    font_size: i32,
    font_type: i32,
    font_effect: i32,
    font_color: i32,
    locked: bool,
    /// Set by `savepoint` and cleared by `save_point_clear`. Native `save`
    /// returns success without writing when this latch is clear.
    armed: bool,
    locks: BTreeMap<i32, i32>,
    /// Slot stored when `save` is called with a non-zero remember flag.
    remembered_slot: i32,
    last_slot: i32,
    last_result: i32,
    /// VM image captured at the last `savepoint`, written by a later `save`.
    checkpoint: Option<RuntimeSaveSnapshot>,
    /// Most recent ADV wait before the save menu changes the scene.
    resume_checkpoint: Option<RuntimeSaveSnapshot>,
    /// Point id registered through `set_load_after_process` (category 10
    /// index 24). The native engine jumps the script VM to this point after a
    /// successful `load`.
    load_after_point: Option<i32>,
    snapshots: BTreeMap<i32, RuntimeSaveSnapshot>,
    text_sprites: BTreeMap<(i32, i32, i32), SpriteHandle>,
    thumbnail_sprites: BTreeMap<(i32, i32, i32), SpriteHandle>,
}

/// `savepoint(1000)` clears the captured image. Koikake `sub_422E20`.
const SAVEPOINT_CLEAR_SLOT: i32 = 1000;
const SAVE_SPRITE_CAP: usize = 128;
const SAVE_SPRITE_BYTES_CAP: usize = 8 * 1024 * 1024;
const SAVE_MEMDAT_CAP: usize = 65_536;
const SAVE_NAME_CAP: usize = 256;
const PORTABLE_SAVE_VERSION: u32 = 16;
const CHECKPOINT_SPRITE_BYTES_CAP: usize = 256 * 1024 * 1024;
/// BGM slots are a handful in practice; cap the serialized list regardless.
const SAVE_BGM_TRACK_CAP: usize = 64;

#[derive(Clone, Debug)]
struct SavedSprite {
    slot: i32,
    transition_source: bool,
    x: i32,
    y: i32,
    z: i32,
    offset_x: i32,
    offset_y: i32,
    priority: i32,
    scale_bits: u32,
    scale_y_bits: Option<u32>,
    color: u32,
    visible: bool,
    rect: [i32; 4],
    width: u32,
    height: u32,
    native_projection: Option<(f32, f32)>,
    center_scale: bool,
    rgba: Arc<Vec<u8>>,
    option_type: u32,
    option_backup: Vec<u8>,
}

#[derive(Clone, Debug)]
struct SavedButton {
    base_position: [i32; 2],
    slider_position: [i32; 2],
    registration_order: u64,
    native_flags: u32,
    hover_callbacks: [Option<u32>; 2],
    base_row: i32,
    group: i32,
    index: i32,
    visible: bool,
    enabled: bool,
    alpha: u8,
    gosub_point: i32,
    x: i32,
    y: i32,
    z: i32,
    priority: i32,
    width: u32,
    height: u32,
    cell_width: u32,
    cell_height: u32,
    rect: [i32; 4],
    color: u32,
    name: String,
    rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
struct SavedButtonGroup {
    group: i32,
    normal_image: i32,
    hover_image: i32,
    onmouse_index: i32,
    offmouse_index: i32,
    push_index: i32,
    slider_ended: bool,
    raw_toggle: i32,
    locked: bool,
    lock_started_ms: u32,
    lock_duration_ms: u32,
}

/// BGM slot state persisted in portable saves (snapshot version 5).  The
/// original engine serializes its sound wrapper; the portable snapshot keeps
/// the resolved resource name so a load can reopen and replay the track.
#[derive(Clone, Debug)]
struct SavedBgmTrack {
    slot: i32,
    name: String,
    looping: bool,
    loop_start: i64,
    loop_end: i64,
}

/// Live BGM slot bookkeeping mirrored into save snapshots.
#[derive(Clone, Debug, Default)]
struct BgmSlotState {
    name: String,
    native_flags: i32,
    looping: bool,
    playing: bool,
    loop_samples: Option<(i64, i64)>,
}

/// ADV click wait parked while a modal menu gosub (SAVE/LOAD/SYSTEM) runs.
/// Native Game.exe runs these menus as separate processes on top of the
/// parked text wait; the portable VM resolves the wait to run the menu, then
/// re-parks it when the menu's gosub returns so the story does not advance.
#[derive(Clone, Copy, Debug)]
struct ModalWaitSuspension {
    return_pc: u32,
    call_depth: usize,
    request: WaitRequest,
    text_visible: bool,
    show_wait_mark: bool,
}

#[derive(Clone, Debug, Default)]
struct RuntimeSaveSnapshot {
    ui: Option<ui_snapshot::SavedUi>,
    host_checkpoint: Option<host_checkpoint::HostCheckpoint>,
    dynamic_strings: Option<(Vec<String>, usize)>,
    version: u32,
    pc: u32,
    call_stack: Vec<u32>,
    user_mem: Vec<i32>,
    system_mem: Vec<i32>,
    temp_mem: Vec<i32>,
    mem_dat_words: Vec<i32>,
    history_records: Vec<[i32; 9]>,
    history_texts: Vec<Option<FrozenHistoryText>>,
    text_args: [i32; 4],
    text_base: i32,
    text_mode: i32,
    /// ADV window alpha (category 2 index 11). The script re-establishes it
    /// whenever a scene or background changes, so a save restored mid-scene
    /// depends on it; `None` in saves written before version 7.
    text_alpha: Option<i32>,
    text_visible: bool,
    vars: Vec<i32>,
    stack: Vec<i32>,
    argument_stack: Vec<i32>,
    argument_base: i32,
    text_initialized: bool,
    text_init_args: [i32; 8],
    text_color: u32,
    text_effect_color: u32,
    show_wait_mark: bool,
    resume_wait_click: bool,
    title_bytes: Vec<u8>,
    thumb_width: i32,
    thumb_height: i32,
    thumb_pixels: Vec<u8>,
    sprites: Vec<SavedSprite>,
    buttons: Vec<SavedButton>,
    button_groups: Vec<SavedButtonGroup>,
    bgm_tracks: Vec<SavedBgmTrack>,
    bgv_tracks: Vec<SavedBgvTrack>,
    tips: SavedTipsState,
    actions: actions::SavedActions,
    children: children::SavedChildren,
}

#[derive(Clone, Debug, Default)]
struct HistorySubsystemState {
    icons: history_icons::HistoryIcons,
    initialized: bool,
    rect: [i32; 4],
    colors: [i32; 2],
    layout: [i32; 7],
    current_text_value: i32,
    /// Native ctx+0x5D68: total pixel height of the laid-out backlog text.
    height: i32,
    /// Native ctx+0x5D70: scroll position in RECORD units; 0 shows the newest
    /// records at the bottom of the view and `max` shows the oldest.
    scroll_y: i32,
    active: bool,
    records: Vec<[i32; 9]>,
    texts: Vec<Option<FrozenHistoryText>>,
    /// Bumped on every records mutation; the wrapped layout cache keys on it.
    records_generation: u64,
    layout_cache: Option<HistoryLayoutCache>,
    /// Record index under the mouse, maintained by history_update (idx6).
    hovered_record: Option<usize>,
    skipped: bool,
    sprite: Option<SpriteHandle>,
}

/// Wrapped per-record layout for the backlog view.  Rebuilt lazily whenever
/// the record list, view rect, or wrap-relevant font metrics change.
///
/// Field semantics recovered from koikake.exe: a record is a name block
/// (history_init arg4 = 40px tall when present) stacked above a body block
/// (wrapped at the ADV body width, ADV font size, pitch = size + arg0), with
/// history_init arg7 = 16px between records.  apply_scroll (0x41F800) fills
/// the view from the BOTTOM up, placing the newest visible record's body at
/// the view bottom; a record whose top would cross the view top is dropped
/// entirely rather than clipped.
#[derive(Clone, Debug)]
struct HistoryLayoutCache {
    generation: u64,
    rect: [i32; 4],
    /// Rasterized glyph height at the backlog font size (no leading).
    line_height: i32,
    /// Line pitch inside a body block: font size + text_init's first arg.
    pitch: i32,
    /// Name-block height (history_init arg4).
    name_block: i32,
    /// Inter-record gap (history_init arg7).
    record_gap: i32,
    /// Body text x offset inside the view (history_init arg3).
    body_x: i32,
    /// Name text x offset inside the view (history_init arg1).
    name_x: i32,
    records: Vec<HistoryRecordLayout>,
    /// Total laid-out pixel height (native ctx+0x5D68, history_get_height).
    total_height: i32,
    /// Records visible when the view is filled from the newest record up.
    visible_from_bottom: usize,
    /// Native ctx+0x5D6C: maximum scroll position in record units.
    max_pos: i32,
}

#[derive(Clone, Debug)]
struct HistoryRecordLayout {
    name: Option<String>,
    body_lines: Vec<String>,
    /// Body block pixel height (wrapped lines * pitch).
    body_height: i32,
    /// Full record slot height: name block + body + trailing record gap.
    slot_height: i32,
    /// The record carries a voice clip (eligible for hover/replay).
    has_voice: bool,
}

#[derive(Clone, Debug, Default)]
struct ThreadSubsystemState {
    next_id: i32,
    active: BTreeMap<i32, bool>,
    last_id: i32,
    running: i32,
    target_point: i32,
    target_pc: Option<u32>,
    vars: Vec<i32>,
    stack: Vec<i32>,
    argument_stack: Vec<i32>,
    argument_base: i32,
    call_stack: Vec<u32>,
    wait: Option<ThreadWaitState>,
    ticking: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ThreadWaitState {
    request: WaitRequest,
    started_ms: u32,
    remaining_frames: i32,
}

impl ThreadWaitState {
    fn new(request: WaitRequest, now_ms: u32) -> Self {
        let remaining_frames = match request {
            WaitRequest::Frame(frames) => frames,
            _ => 0,
        };
        Self {
            request,
            started_ms: now_ms,
            remaining_frames,
        }
    }

    fn is_complete(&mut self, now_ms: u32, input: Option<&PalInputState>, sprites: Option<&SpriteSystem>) -> bool {
        match self.request {
            WaitRequest::Frame(frames) => {
                if frames < 0 {
                    return false;
                }
                if self.remaining_frames <= 0 {
                    return true;
                }
                self.remaining_frames -= 1;
                self.remaining_frames <= 0
            }
            WaitRequest::Time(ms) | WaitRequest::TextReveal(ms) => {
                now_ms.wrapping_sub(self.started_ms) >= ms
            }
            WaitRequest::SpriteFx(handle) => input.is_some_and(PalInputState::click_push)
                || sprites.is_some_and(|sprites| sprites.sprite_fx_effect_state(handle) == 0),
            WaitRequest::Click => input.is_some_and(PalInputState::click_push),
            WaitRequest::ClickOrTime(ms) | WaitRequest::AutoClickOrTime(ms) => {
                input.is_some_and(PalInputState::click_push)
                    || now_ms.wrapping_sub(self.started_ms) >= ms
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct MessageSubsystemState {
    next_id: i32,
    slots: [GameMessage; 8],
    queue: Vec<GameMessage>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct GameMessage {
    active: bool,
    id: i32,
    value: i32,
    param: i32,
}

fn ext_args_source_order<const N: usize>(args: &[i32]) -> [i32; N] {
    let mut ordered = [0; N];
    for (dst, value) in args.iter().rev().take(N).enumerate() {
        ordered[dst] = *value;
    }
    ordered
}

fn checked_script_mem_index(kind: &str, signed_idx: i32, len: usize) -> Option<usize> {
    if signed_idx < 0 {
        log::debug!(
            "[trace-vm] {kind} signed index {signed_idx} out of range (len={len}); returning zero / ignoring write"
        );
        return None;
    }
    let idx = signed_idx as usize;
    if idx >= len {
        log::debug!(
            "[trace-vm] {kind} index {idx} out of range (len={len}); returning zero / ignoring write"
        );
        return None;
    }
    Some(idx)
}

impl ScriptRuntime {
    pub fn boot(entry_pc: u32, config: ScriptRuntimeConfig) -> Self {
        Self {
            pc: entry_pc,
            entry_pc,
            vars: vec![0; DEFAULT_VAR_COUNT],
            stack: Vec::new(),
            argument_stack: Vec::new(),
            call_stack: Vec::new(),
            status: RuntimeStatus::Running { pc: entry_pc },
            trace: config.trace,
            wait_task_handle: None,
            wait_task_kind: None,
            adv_wait_checkpoint_pending: false,
            pal_time_ms: 0,
            host_active: true,
            host_gpu_renderer: false,
            registered_text_animation: None,
            se_prefix_info: std::array::from_fn(|_| SePrefixInfo::default()),
            registered_se_flags: [0; 16],
            registered_se_controls: [registered_audio::SeChannelControl::default(); 16],
            registered_se_pan: 0,
            pending_registered_se_wait: None,
            host_touch_available: cfg!(any(target_os = "ios", target_os = "android")),
            observed_fullscreen: false,
            last_window_mode_change_ms: 0,
            pending_global_fx_release: false,
            wait_sync_begin_ms: 0,
            wait_sync_release: None,
            wait_time_stack: Vec::new(),
            run_pipeline: RunPipelineState::default(),
            work_process_attached: false,
            access_updates: BTreeMap::new(),
            action_state: ActionSubsystemState::default(),
            action_audio_requests: Vec::new(),
            effect_system: PalEffectSystem::new(),
            msprite_system: MSpriteSystem::new(),
            full_movie: None,
            movie_volume_type: 0,
            movie_volume_raw: 10000,
            movie_muted: false,
            movie_cancel_fade_enabled: true,
            argument_base: 0,
            user_mem: vec![0; DEFAULT_MEM_SIZE],
            system_mem: vec![0; DEFAULT_MEM_SIZE],
            temp_mem: vec![0; DEFAULT_MEM_SIZE],
            mem_dat_words: Vec::new(),
            memory_state_stack: Vec::new(),
            list_point_stack: Vec::new(),
            extcall_dst_raw: 0,
            extcall_instruction_size: 12,
            input_control: InputControlState::default(),
            file_handles: Vec::new(),
            game_sprites: BTreeMap::new(),
            backbuffer_sprite: None,
            game_sprite_transitions: BTreeMap::new(),
            game_sprite_transition_sources: BTreeMap::new(),
            game_sprite_animations: BTreeMap::new(),
            game_sprite_placements: BTreeMap::new(),
            game_sprite_native_scales: BTreeMap::new(),
            game_sprite_wrapper_visuals: BTreeMap::new(),
            game_sprite_child_lanes: BTreeMap::new(),
            game_sprite_child_offsets: BTreeMap::new(),
            game_sprite_render_lanes: BTreeMap::new(),
            game_sprite_anim_params: BTreeMap::new(),
            game_sprite_aspect_position_types: BTreeMap::new(),
            game_sprite_vis_clip_slots: BTreeSet::new(),
            game_face_slots: BTreeMap::new(),
            sprite_priority_cursor: 0,
            game_sprite_pending_named_animations: BTreeMap::new(),
            game_sprite_pending_alpha: BTreeMap::new(),
            game_sprite_base_alpha: BTreeMap::new(),
            game_sprite_final_alpha_delta: BTreeMap::new(),
            game_sprite_active_alpha: Vec::new(),
            game_sprite_pending_position: Vec::new(),
            retired_sprites: Vec::new(),
            game_msprites: BTreeMap::new(),
            pending_msp_wait_slot: None,
            game_buttons: BTreeMap::new(),
            next_button_registration: 1,
            button_groups: BTreeMap::new(),
            button_push_queue: BTreeMap::new(),
            pressed_button: None,
            button_release_mode: false,
            button_release_capture: None,
            button_group_release_capture: BTreeMap::new(),
            adv_menu_expanded: false,
            system_buttons: BTreeMap::new(),
            system_button_blocked_at: None,
            button_input_block_duration: 50,
            button_run_skip: 0,
            button_children: BTreeMap::new(),
            history_navigation: Default::default(),
            button_sounds: BTreeMap::new(),
            retired_button_sounds: Vec::new(),
            pending_button_reactions: BTreeMap::new(),
            button_sound_hover: BTreeMap::new(),
            button_callback_hover: BTreeMap::new(),
            input_registered_profile: false,
            input_release_shortcuts: false,
            game_audio: BTreeMap::new(),
            bgv_state: BgvState::default(),
            master_volume_percent: 100,
            master_muted: false,
            bgm_volume_percent: 100,
            bgm_muted: false,
            bgm_auto_volume_percent: 100,
            bgm_auto_muted: false,
            bgm_loops: None,
            bgm_slots: BTreeMap::new(),
            bgm_replay_pending: false,
            se_volume_percent: BTreeMap::new(),
            se_enabled: BTreeMap::new(),
            voice_ex_volume_percent: BTreeMap::new(),
            voice_ex_enabled: BTreeMap::new(),
            voice_unit_headers: None,
            se_muted: BTreeMap::new(),
            pending_voice_wait_slot: None,
            font_state: PalFontSystem::new(),
            text_state: TextSubsystemState::default(),
            tips_state: TipsState::default(),
            text_skip_enabled: false,
            text_auto_enabled: false,
            scene_skip_active: false,
            last_skip_state: 0,
            read_text_ids: BTreeSet::new(),
            current_line_read: false,
            select_state: SelectSubsystemState::default(),
            save_state: SaveSubsystemState {
                // Native sub_44D320 initializes the most recently saved slot
                // (task data + 24) to -1, before any save has been remembered.
                remembered_slot: -1,
                ..SaveSubsystemState::default()
            },
            history_state: HistorySubsystemState::default(),
            thread_state: ThreadSubsystemState::default(),
            message_state: MessageSubsystemState::default(),
            dynamic_strings: Vec::new(),
            dynamic_string_cursor: 0,
            system_ini: None,
            text_lang: TextLang::default(),
            font_face_candidates: Vec::new(),
            loaded_face_path: None,
            style_faces_enabled: false,
            system_state: PalSystemState::new(),
            random_state: PalRandomState::with_seed(2048, pal_boot_random_seed()),
            frame_events: Vec::new(),
            pending_gosub_points: VecDeque::new(),
            modal_wait_suspensions: Vec::new(),
            pending_jump_point: None,
            menu_transition_mode: 0,
            system_scratch_value: 0,
            debug_window_state: 0,
        }
    }

    /// Select which field of a multi-language text record is displayed.
    ///
    /// Only affects the FHD text layout; original single-language pools have
    /// no separator and render identically for every value.
    pub fn set_text_lang(&mut self, lang: TextLang) {
        self.text_lang = lang;
    }

    /// Switch the displayed text field of a multi-language release and refresh
    /// everything already rendered from the previous field.
    ///
    /// The FHD releases join every language in one text record, so a language
    /// change is visible immediately: the current ADV body re-typesets and the
    /// backlog is measured again from the new field. Original single-language
    /// pools carry no separator, so the switch leaves their text alone.
    pub fn apply_text_language(&mut self, lang: TextLang) {
        if self.text_lang == lang {
            return;
        }
        self.set_text_lang(lang);
        self.invalidate_text_layout();
    }

    /// Re-typeset everything already rendered from the current text.
    ///
    /// The body panel is rasterized once per text change; dropping the cache
    /// re-resolves and rebuilds it even if the text is hidden at the moment of
    /// the change (a hidden window only clears `dirty`). The backlog is
    /// measured again for the same reason.
    fn invalidate_text_layout(&mut self) {
        self.text_state.dirty = true;
        self.text_state.render_cache = None;
        self.history_state.layout_cache = None;
    }

    pub fn set_font_profile(&mut self, registered_softpal: bool, text_dat_multilingual: bool) {
        self.font_state.set_profile(if registered_softpal {
            PalFontProfile::Totsulover
        } else if text_dat_multilingual {
            PalFontProfile::LocalizedReference
        } else {
            PalFontProfile::Koikake
        });
    }

    pub fn set_system_ini(&mut self, ini: IniFile) {
        let (width, height) = ini_graphics_size(
            Some(&ini),
            FrameScene::PAL_DEFAULT_WIDTH,
            FrameScene::PAL_DEFAULT_HEIGHT,
        );
        self.system_state
            .set_logical_size(width as i32, height as i32);
        if let Some(effect) = ini_first_int(&ini, "def_font_effect") {
            self.font_state.set_effect(effect.max(0) as u16);
        }
        if let Some(font_type) = ini_first_int(&ini, "def_font_type") {
            self.font_state.set_type_word(font_type as i32);
        }
        self.system_ini = Some(ini);
    }

    pub fn load_configured_font(
        &mut self,
        resource_manager: &mut ResourceManager,
        nls: Nls,
        root: Option<&Path>,
        font_file: Option<&Path>,
        lang: TextLang,
        text_dat_multilingual: bool,
    ) {
        if text_dat_multilingual && self.font_state.profile() != PalFontProfile::Totsulover {
            self.font_state.set_profile(PalFontProfile::LocalizedReference);
        }
        let name = self
            .system_ini
            .as_ref()
            .and_then(|ini| ini_first_str(ini, "add_fontname"))
            .unwrap_or_else(|| "default_font".to_owned());
        match open_resource_variant(resource_manager, &name, FONT_DATA_EXTENSIONS) {
            Ok(asset) => match self.font_state.load_named_bitmap_font(&asset.name, asset.bytes, nls) {
                Ok(()) => {
                    if let Some(font_type) = self
                        .system_ini
                        .as_ref()
                        .and_then(|ini| ini_first_int(ini, "def_font_type"))
                    {
                        self.font_state.set_type_word(font_type as i32);
                    }
                    log::debug!("[trace-text] loaded PAL bitmap font {:?}", asset.name);
                }
                Err(err) => log::warn!(
                    "[trace-text] PAL bitmap font {:?} is invalid: {err}",
                    asset.name
                ),
            },
            Err(err) => log::debug!("[trace-text] PAL bitmap font {name:?} unavailable: {err}"),
        }
        self.font_face_candidates = game_font_faces(root, font_file);
        let face = pick_face_for(&self.font_face_candidates, lang, None);
        let external = self.load_face(face);
        // A localized release ships the original `DEFAULT_FONT.DAT` and draws
        // text with its own faces, so only leave the bitmap font in play when
        // such a release supplied no face at all. Such a release also asks the
        // player to choose the face on its SYSTEM screen, which reaches the
        // engine as a PAL font type — only where it ships more than one face.
        if text_dat_multilingual && external {
            self.font_state.set_bitmap_text_enabled(false);
            self.style_faces_enabled = self.font_face_candidates.len() > 1;
        }
        self.ensure_wait_mark_sheet(resource_manager);
    }

    /// Load `path` as the text face, remembering it as the active one.
    ///
    /// Returns whether a face was loaded.
    fn load_face(&mut self, path: Option<PathBuf>) -> bool {
        let Some(path) = path else {
            return false;
        };
        match pal_asset::vfs::read_all(&path).map_err(|e|std::io::Error::other(e.to_string())) {
            Ok(bytes) => {
                if self.font_state.load_external_font(bytes) {
                    log::debug!("[trace-text] loaded external font {}", path.display());
                    self.loaded_face_path = Some(path);
                    true
                } else {
                    log::warn!(
                        "[trace-text] external font {} is not a usable TrueType face",
                        path.display()
                    );
                    false
                }
            }
            Err(err) => {
                log::warn!(
                    "[trace-text] external font {} is unreadable: {err}",
                    path.display()
                );
                false
            }
        }
    }

    /// Re-select the text face that a PAL font type names.
    ///
    /// The FHD releases offer 思源宋体 and 思源黑体 on their SYSTEM screen and
    /// record the picked row as the PAL font type the boot path passes to
    /// `set_language` (category 9 index 17), so the type doubles as the face
    /// style there. The row writes no other state and the change is applied
    /// here rather than by a live redraw, matching those releases' engine: the
    /// stored type is picked up on the next boot. The original releases ship no
    /// such faces and keep the face boot selected.
    fn select_style_face(&mut self, style: FaceStyle) {
        if !self.style_faces_enabled {
            return;
        }
        let lang = self.text_lang;
        let Some(path) = pick_face_for(&self.font_face_candidates, lang, Some(style)) else {
            return;
        };
        if self.loaded_face_path.as_deref() == Some(path.as_path()) {
            return;
        }
        if self.load_face(Some(path)) {
            self.invalidate_text_layout();
        }
    }

    pub fn load_portable_system_data(&mut self, root: &Path) {
        let path = portable_system_data_path(root);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return;
        };
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim().parse::<i32>().unwrap_or(0);
            match key.trim() {
                "master_volume_percent" => self.master_volume_percent = clamp_percent(value),
                "master_muted" => self.master_muted = value != 0,
                "bgm_volume_percent" => self.bgm_volume_percent = clamp_percent(value),
                "bgm_muted" => self.bgm_muted = value != 0,
                "voice_volume_percent" => self.text_state.voice_volume = clamp_percent(value),
                "voice_muted" => self.text_state.voice_muted = value != 0,
                "text_skip_enabled" => self.text_skip_enabled = value != 0,
                "text_auto_enabled" => self.text_auto_enabled = value != 0,
                "text_speed_percent" => {
                    self.system_state.set_text_speed_percent(value);
                }
                "auto_speed_percent" => {
                    self.system_state.set_auto_speed_percent(value);
                }
                "skip_gate" => {
                    self.system_state.set_skip_gate(value);
                }
                key => {
                    if let Some(slot) = key.strip_prefix("se_volume_percent_") {
                        if let Ok(slot) = slot.parse::<i32>() {
                            self.se_volume_percent.insert(slot, clamp_percent(value));
                        }
                    } else if let Some(slot) = key.strip_prefix("voice_ex_volume_percent_") {
                        if let Ok(slot) = slot.parse::<i32>() {
                            self.voice_ex_volume_percent.insert(slot, clamp_percent(value));
                        }
                    } else if let Some(slot) = key.strip_prefix("voice_ex_enabled_") {
                        if let Ok(slot) = slot.parse::<i32>() {
                            self.voice_ex_enabled.insert(slot, value != 0);
                        }
                    }
                }
            }
        }
        log::debug!(
            "[trace-save] loaded portable system data {}",
            path.display()
        );
        self.load_portable_system_mem(root);
        self.load_portable_read_lines(root);
    }

    /// Load the persisted system_mem bank (global script settings, e.g. the
    /// SYSTEM screen toggles). The native engine persists its global system
    /// state into system.dat; sena-rs stores its own portable companion file.
    fn load_portable_system_mem(&mut self, root: &Path) {
        let path = portable_system_mem_path(root);
        let Ok(bytes) = pal_asset::vfs::read_all(&path).map_err(|e|std::io::Error::other(e.to_string())) else {
            return;
        };
        match decode_portable_system_mem(&bytes) {
            Ok(words) => {
                install_i32_words(&mut self.system_mem, &words, DEFAULT_MEM_SIZE);
                log::debug!(
                    "[trace-save] loaded portable system mem {} ({} words)",
                    path.display(),
                    words.len()
                );
            }
            Err(err) => log::warn!(
                "[trace-save] ignoring invalid portable system mem {}: {err}",
                path.display()
            ),
        }
    }

    fn write_portable_system_mem(&self, root: &Path) -> std::io::Result<PathBuf> {
        let path = portable_system_mem_path(root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = encode_portable_system_mem(&self.system_mem);
        std::fs::write(&path, bytes)?;
        log::debug!(
            "[trace-save] wrote portable system mem {} ({} words)",
            path.display(),
            self.system_mem.len()
        );
        Ok(path)
    }

    fn write_portable_read_lines(&self, root: &Path) -> std::io::Result<PathBuf> {
        let path = portable_read_lines_path(root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = encode_portable_read_lines(&self.read_text_ids);
        std::fs::write(&path, bytes)?;
        log::debug!(
            "[trace-save] wrote portable read lines {} ({} ids)",
            path.display(),
            self.read_text_ids.len()
        );
        Ok(path)
    }

    /// Load the persisted read-text bitmap (native persists it in
    /// system.dat; sena-rs stores a portable companion file).
    fn load_portable_read_lines(&mut self, root: &Path) {
        let path = portable_read_lines_path(root);
        let Ok(bytes) = pal_asset::vfs::read_all(&path).map_err(|e|std::io::Error::other(e.to_string())) else {
            return;
        };
        match decode_portable_read_lines(&bytes) {
            Ok(ids) => {
                log::debug!(
                    "[trace-save] loaded portable read lines {} ({} ids)",
                    path.display(),
                    ids.len()
                );
                self.read_text_ids = ids;
            }
            Err(err) => log::warn!(
                "[trace-save] ignoring invalid portable read lines {}: {err}",
                path.display()
            ),
        }
    }

    fn write_portable_system_data(&self, root: &Path) -> std::io::Result<PathBuf> {
        let path = portable_system_data_path(root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut text = format!(
            "master_volume_percent={}\nmaster_muted={}\nbgm_volume_percent={}\nbgm_muted={}\nvoice_volume_percent={}\nvoice_muted={}\ntext_skip_enabled={}\ntext_auto_enabled={}\ntext_speed_percent={}\nauto_speed_percent={}\nskip_gate={}\n",
            self.master_volume_percent,
            i32::from(self.master_muted),
            self.bgm_volume_percent,
            i32::from(self.bgm_muted),
            self.text_state.voice_volume,
            i32::from(self.text_state.voice_muted),
            i32::from(self.text_skip_enabled),
            i32::from(self.text_auto_enabled),
            self.system_state.text_speed_percent(),
            self.system_state.auto_speed_percent(),
            self.system_state.skip_gate(),
        );
        for (slot, percent) in &self.se_volume_percent {
            text.push_str(&format!("se_volume_percent_{slot}={percent}\n"));
        }
        for (slot, percent) in &self.voice_ex_volume_percent {
            text.push_str(&format!("voice_ex_volume_percent_{slot}={percent}\n"));
        }
        for (slot, enabled) in &self.voice_ex_enabled {
            text.push_str(&format!("voice_ex_enabled_{slot}={}\n", i32::from(*enabled)));
        }
        std::fs::write(&path, text)?;
        log::debug!("[trace-save] wrote portable system data {}", path.display());
        Ok(path)
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.system_state
            .set_window_size(width.max(1) as i32, height.max(1) as i32);
    }

    pub fn observe_window_geometry(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        fullscreen: bool,
    ) {
        if self.observed_fullscreen != fullscreen {
            self.last_window_mode_change_ms = self.pal_time_ms;
        }
        self.observed_fullscreen = fullscreen;
        self.system_state
            .observe_window_geometry(x, y, width, height, fullscreen);
    }

    pub fn set_host_active(&mut self, active: bool) {
        self.host_active = active;
    }

    pub fn set_host_gpu_renderer(&mut self, enabled: bool) {
        self.host_gpu_renderer = enabled;
    }

    pub fn set_host_touch_available(&mut self, available: bool) {
        self.host_touch_available = available;
    }

    /// Game.exe's effect worker releases a stopped global FX attachment only
    /// after PalGetFxEffectState reaches zero (Koikake sub_413740).
    pub fn finish_global_fx_stop(&mut self, sprites: &mut SpriteSystem) {
        if self.pending_global_fx_release && sprites.global_raster_state() == 0 {
            sprites.release_global_raster();
            self.pending_global_fx_release = false;
        }
    }

    pub fn take_window_requests(&mut self) -> Vec<crate::system::PalWindowRequest> {
        self.system_state.take_window_requests()
    }

    fn vm_trace_enabled(&self) -> bool {
        if !self.trace && !debug_vm_enabled() {
            return false;
        }
        let pc = self.pc;
        if let Some(start) = debug_vm_pc_bound("DEBUG_VM_PC_START") {
            if pc < start {
                return false;
            }
        }
        if let Some(end) = debug_vm_pc_bound("DEBUG_VM_PC_END") {
            if pc > end {
                return false;
            }
        }
        true
    }

    fn vm_trace(&self, args: fmt::Arguments<'_>) {
        if !self.vm_trace_enabled() {
            return;
        }
        if debug_vm_enabled() {
            eprintln!("[DEBUG_VM] {args}");
        } else if self.trace {
            log::debug!("{args}");
        }
    }

    /// Initialise the writable Mem.dat shadow from the raw asset bytes.
    /// Call once after boot, before running frames.
    pub fn load_mem_dat(&mut self, bytes: &[u8]) {
        let word_count = bytes.len() / 4;
        let mut words = Vec::with_capacity(word_count);
        for i in 0..word_count {
            let off = i * 4;
            words.push(i32::from_le_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3],
            ]));
        }
        self.mem_dat_words = words;
    }

    /// Mark the runtime as faulted with a descriptive message.
    /// The runtime will stop executing but the window stays open.
    pub fn set_faulted(&mut self, message: String) {
        self.status = RuntimeStatus::Faulted {
            pc: self.pc,
            message,
        };
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn entry_pc(&self) -> u32 {
        self.entry_pc
    }

    pub fn status(&self) -> &RuntimeStatus {
        &self.status
    }

    pub fn parsed_system_ini(&self) -> Option<&IniFile> {
        self.system_ini.as_ref()
    }

    pub fn vars(&self) -> &[i32] {
        &self.vars
    }

    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    pub fn argument_stack_depth(&self) -> usize {
        self.argument_stack.len()
    }

    pub fn call_stack_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Returns the handle of the active blocking wait task, if any.
    pub fn pending_wait_handle(&self) -> Option<TaskHandle> {
        self.wait_task_handle
    }

    pub fn pending_wait_is_text_reveal(&self) -> bool {
        matches!(self.wait_task_kind, Some(WaitRequest::TextReveal(_)))
    }

    /// Called by Engine when the wait task has completed.
    /// Sets status to Running and clears the handle.
    pub fn resolve_pending_wait(&mut self) {
        let old_handle = self.wait_task_handle;
        let old_status = self.status.clone();
        self.wait_task_handle = None;
        self.wait_task_kind = None;
        if let Some(pc) = self.status.pc() {
            self.status = RuntimeStatus::Running { pc };
        }
        if matches!(old_status, RuntimeStatus::WaitClick { .. }) {
            if self.modal_wait_suspensions.is_empty() {
                self.finish_tips_text();
                self.action_state.flush_requested = true;
            }
            self.text_state.show_wait_mark = false;
            self.text_state.dirty = true;
        }
        if debug_vm_enabled() || matches!(old_status, RuntimeStatus::WaitClick { .. }) {
            log::debug!(
                "[trace-wait] resolve_pending_wait handle={old_handle:?} old_status={old_status} new_status={}",
                self.status
            );
        }
    }

    /// Associate the runtime with a newly created wait task handle.
    pub fn set_wait_handle(&mut self, handle: TaskHandle, kind: WaitRequest) {
        self.wait_task_handle = Some(handle);
        self.wait_task_kind = Some(kind);
        if debug_vm_enabled() || matches!(self.status, RuntimeStatus::WaitClick { .. }) {
            log::debug!(
                "[trace-wait] set_wait_handle handle={handle:?} kind={kind:?} status={}",
                self.status
            );
        }
    }

    /// True when a clicked button queued a gosub while the script is parked at
    /// an ADV click wait.  Native Game.exe runs such menus as separate
    /// processes on top of the parked wait; resolving the wait to run the menu
    /// would advance the story, so the engine must route the click through
    /// `suspend_wait_for_modal` instead of `resolve_pending_wait`.
    pub fn should_suspend_wait_for_modal(&self) -> bool {
        !self.pending_gosub_points.is_empty()
            && matches!(self.status, RuntimeStatus::WaitClick { .. })
    }

    /// Parks the current ADV click wait and lets the queued modal gosub run.
    /// When the gosub returns, `take_modal_wait_repark` re-parks the wait at
    /// the same PC so the line and its click mark survive the menu round trip.
    pub fn suspend_wait_for_modal(&mut self) {
        let RuntimeStatus::WaitClick { pc } = self.status else {
            return;
        };
        let request = self.wait_task_kind.unwrap_or(WaitRequest::Click);
        self.modal_wait_suspensions.push(ModalWaitSuspension {
            return_pc: pc,
            call_depth: self.call_stack.len(),
            request,
            text_visible: self.text_state.visible,
            show_wait_mark: self.text_state.show_wait_mark,
        });
        self.wait_task_handle = None;
        self.wait_task_kind = None;
        self.status = RuntimeStatus::Running { pc };
        log::debug!("[trace-wait] suspend_wait_for_modal pc=0x{pc:08X} request={request:?}");
    }

    /// Called after each executed instruction. When the innermost modal gosub
    /// has returned to the PC that followed the suspended wait, re-park the
    /// wait and hand its request back so the engine recreates the wait task.
    /// True when the innermost modal gosub has unwound to the PC that followed
    /// the suspended ADV click wait, i.e. `take_modal_wait_repark` would fire.
    fn modal_repark_due(&self) -> bool {
        let Some(susp) = self.modal_wait_suspensions.last() else {
            return false;
        };
        self.call_stack.len() == susp.call_depth && self.pc == susp.return_pc
    }

    fn take_modal_wait_repark(&mut self) -> Option<WaitRequest> {
        let susp = self.modal_wait_suspensions.last()?;
        if self.call_stack.len() > susp.call_depth {
            return None;
        }
        // A modal handler may have queued a follow-up gosub instead of opening
        // its menu directly: koikake's F-key shortcut points are pure
        // re-dispatchers that call btn_set_hit (PalButtonSetReaction) onto the
        // on-screen ADV buttons, and the clicked button's own menu gosub is
        // what actually opens the screen.  Re-parking here would strand that
        // queued gosub forever, so keep the wait suspended until the queue
        // drains; the step loop injects the next gosub on top of the
        // suspension before any story instruction can run.
        if self.pc == susp.return_pc
            && self.call_stack.len() == susp.call_depth
            && !self.pending_gosub_points.is_empty()
        {
            return None;
        }
        let susp = self.modal_wait_suspensions.pop()?;
        if self.pc != susp.return_pc || self.call_stack.len() != susp.call_depth {
            if self.modal_wait_suspensions.is_empty() {
                self.discard_tips_wait_clock();
            }
            // The modal left through a jump or returned somewhere else; do not
            // re-park a wait the script no longer sits behind.
            log::debug!(
                "[trace-wait] modal repark abandoned pc=0x{:08X} return_pc=0x{:08X}",
                self.pc,
                susp.return_pc
            );
            return None;
        }
        self.text_state.visible = susp.text_visible;
        self.text_state.show_wait_mark = susp.show_wait_mark;
        self.text_state.dirty = true;
        // The modal gosub may have flipped auto/skip mid-line (the ADV
        // SKIP/AUTO buttons run as modal gosubs on top of this wait).
        // Native wait code polls the latches live every frame: with auto
        // newly enabled the parked click wait gains the post-reveal hold
        // timer; with skip enabled the plain click wait is fine because the
        // task layer completes it through the live skip check.
        let request = if self.text_auto_enabled && matches!(susp.request, WaitRequest::Click) {
            WaitRequest::AutoClickOrTime(self.text_state.current_auto_hold_ms.max(1))
        } else {
            susp.request
        };
        if self.modal_wait_suspensions.is_empty() {
            self.repark_tips_clock();
        }
        self.wait_task_kind = Some(request);
        self.status = match request {
            WaitRequest::Click | WaitRequest::ClickOrTime(_) | WaitRequest::AutoClickOrTime(_) => {
                RuntimeStatus::WaitClick { pc: self.pc }
            }
            _ => RuntimeStatus::WaitFrame { pc: self.pc },
        };
        log::debug!(
            "[trace-wait] modal repark pc=0x{:08X} request={:?}",
            self.pc,
            request
        );
        Some(request)
    }

    /// Inject the PAL cached frame time used by Game.exe wait-sync wrappers.
    pub fn set_pal_time(&mut self, ms: u32) {
        self.pal_time_ms = ms;
        self.effect_system.tick(ms);
    }

    pub fn effect_overlay(&self, logical_width: u32, logical_height: u32) -> Option<SolidQuad> {
        self.effect_system
            .overlay_quad(logical_width, logical_height, self.pal_time_ms)
    }

    pub fn effect_shake_offset(&self) -> [i32; 2] {
        self.effect_system.shake_offset(self.pal_time_ms)
    }

    pub fn effect_state(&self) -> Option<crate::effect::PalEffectState> {
        self.effect_system.state()
    }

    pub(crate) fn effect_render_state(&self) -> Option<crate::effect::PalEffectState> {
        self.effect_system.render_state()
    }

    pub(crate) fn effect_frame_presented(&mut self) {
        self.effect_system.frame_presented();
    }

    pub(crate) fn finish_scene_effect(&mut self) {
        self.effect_system.finish();
        self.run_pipeline.effect_active = false;
    }

    pub(crate) fn cancel_scene_effect_on_click(&mut self, clicked: bool) {
        self.effect_system.cancel_on_click(clicked);
    }

    pub fn advance_msprites(&mut self, sprites: &mut SpriteSystem, delta_ms: u32) {
        for update in self.msprite_system.advance(delta_ms) {
            let sprite = self.full_movie.as_ref().filter(|movie| movie.handle == update.handle).map(|movie| movie.sprite)
                .or_else(|| self.game_msprites.iter().find(|(_, state)| state.handle == Some(update.handle))
                    .and_then(|(slot, _)| self.game_sprites.get(slot).copied()));
            if let Some(sprite) = sprite {
                sprites.replace_msprite_frame(sprite, update.handle, update.width, update.height, update.rgba, update.source_name);
            }
        }
    }

    pub fn release_retired_sprites(&mut self, sprites: &mut SpriteSystem) {
        // PAL marks completion without deleting its worker. Image retirement
        // belongs to the Game wrapper, which cancels and releases the old image.
        for (&slot, &transition) in &self.game_sprite_transitions {
            if sprites.transition_state(transition) == 2 {
                sprites.cancel_transition(transition);
                self.game_sprite_transition_sources.remove(&slot);
            }
        }
        let now = self.pal_time_ms;
        let mut pending = Vec::with_capacity(self.retired_sprites.len());
        for retired in self.retired_sprites.drain(..) {
            if now.wrapping_sub(retired.release_at_ms) < 0x8000_0000 {
                sprites.release(retired.handle);
            } else {
                pending.push(retired);
            }
        }
        self.retired_sprites = pending;
    }

    pub fn advance_sprite_action_lanes(&mut self, sprites: &mut SpriteSystem) {
        let now = self.pal_time_ms;
        let mut alpha_slots = Vec::new();
        let mut active_alpha = Vec::with_capacity(self.game_sprite_active_alpha.len());
        let alpha_actions = std::mem::take(&mut self.game_sprite_active_alpha);
        for action in alpha_actions {
            let elapsed = now.wrapping_sub(action.started_ms);
            if elapsed >= action.duration_ms {
                if self.game_sprites.get(&action.slot).copied() == Some(action.handle) {
                    *self
                        .game_sprite_final_alpha_delta
                        .entry(action.slot)
                        .or_insert(0) += action.alpha_delta;
                    alpha_slots.push((action.slot, action.handle));
                }
            } else {
                alpha_slots.push((action.slot, action.handle));
                active_alpha.push(action);
            }
        }
        self.game_sprite_active_alpha = active_alpha;
        alpha_slots.sort_by_key(|(slot, handle)| (*slot, handle.0));
        alpha_slots.dedup();
        for (slot, handle) in alpha_slots {
            if self.game_sprites.get(&slot).copied() == Some(handle) {
                self.submit_game_sprite_alpha(sprites, slot, handle);
            }
        }

        let mut pending = Vec::with_capacity(self.game_sprite_pending_position.len());
        let mut active_position_slots = Vec::new();
        let actions = std::mem::take(&mut self.game_sprite_pending_position);
        for action in actions {
            let elapsed = now.wrapping_sub(action.started_ms);
            if elapsed < action.duration_ms {
                if self.game_sprites.get(&action.slot).copied() == Some(action.handle) {
                    active_position_slots.push((action.slot, action.handle));
                }
                pending.push(action);
                continue;
            }
            if self.game_sprites.get(&action.slot).copied() != Some(action.handle) {
                continue;
            }
            let mut committed = false;
            if let Some(visual) = self.game_sprite_wrapper_visuals.get_mut(&action.slot) {
                visual.native_x += action.native_dx;
                visual.native_y += action.native_dy;
                visual.native_z += action.native_dz;
                committed = true;
            }
            if committed {
                let _ = self.commit_game_sprite_wrapper_visual(sprites, action.slot, action.handle);
                log::debug!(
                    "[trace-action] action_position_commit slot={} handle={:?} native_delta=({:.1},{:.1},{:.1}) duration_ms={}",
                    action.slot,
                    action.handle,
                    action.native_dx,
                    action.native_dy,
                    action.native_dz,
                    action.duration_ms
                );
            }
        }
        self.game_sprite_pending_position = pending;
        active_position_slots.sort_by_key(|(slot, handle)| (*slot, handle.0));
        active_position_slots.dedup();
        for (slot, handle) in active_position_slots {
            if self.game_sprites.get(&slot).copied() == Some(handle) {
                let _ = self.commit_game_sprite_wrapper_visual(sprites, slot, handle);
            }
        }
        self.apply_game_sprite_child_lanes(sprites);
        self.apply_button_children(sprites);
    }

    fn release_retired_sprites_for_slot(&mut self, slot: i32, sprites: &mut SpriteSystem) {
        let mut pending = Vec::with_capacity(self.retired_sprites.len());
        for retired in self.retired_sprites.drain(..) {
            if slot == -1 || retired.slot == slot {
                sprites.release(retired.handle);
            } else {
                pending.push(retired);
            }
        }
        self.retired_sprites = pending;
    }

    pub fn sync_text_sprite(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        mut resource_manager: Option<&mut ResourceManager>,
        sprites: &mut SpriteSystem,
    ) {
        if !self.text_state.initialized {
            if let Some(handle) = self.text_state.sprite.take() {
                let _ = sprites.release(handle);
            }
            if let Some(handle) = self.text_state.name_sprite.take() {
                let _ = sprites.release(handle);
            }
            self.text_state.dirty = false;
            self.text_state.presented_frame = None;
            self.sync_adv_button_chrome_visibility(sprites);
            return;
        }
        if !self.text_state.visible {
            if let Some(handle) = self.text_state.sprite {
                let _ = sprites.view_ctrl(handle, false);
            }
            if let Some(handle) = self.text_state.name_sprite {
                let _ = sprites.view_ctrl(handle, false);
            }
            self.text_state.dirty = false;
            self.text_state.presented_frame = None;
            self.sync_adv_button_chrome_visibility(sprites);
            return;
        }
        if !self.text_state.dirty
            && !self.text_state.reveal_enabled
            && !self.text_state.show_wait_mark
        {
            self.sync_adv_button_chrome_visibility(sprites);
            return;
        }
        let log_sync = self.text_state.dirty;
        let (body, name) = self.adv_text_parts_for_render(assets, nls);
        if log_sync {
            log::debug!(
                "[trace-text] sync visible={} args={:?} name={name:?} body={body:?}",
                self.text_state.visible,
                self.text_state.last_text_args
            );
        }
        if body.is_empty() {
            if let Some(handle) = self.text_state.sprite.take() {
                let _ = sprites.release(handle);
            }
            if let Some(handle) = self.text_state.name_sprite.take() {
                let _ = sprites.release(handle);
            }
            self.text_state.dirty = false;
            self.text_state.presented_frame = None;
            self.sync_adv_button_chrome_visibility(sprites);
            return;
        }
        let mut rebuilt = false;
        if self.text_state.dirty || self.text_state.render_cache.is_none() {
            self.rebuild_adv_text_render_cache(
                assets,
                nls,
                resource_manager.as_mut().map(|manager| &mut **manager),
                &body,
            );
            self.text_state.dirty = false;
            self.text_state.presented_frame = None;
            rebuilt = true;
            self.sync_adv_name_sprite(&name, sprites);
        }
        self.apply_pending_text_alpha_actions(sprites);
        let reveal_limit = self.adv_text_reveal_limit();
        let animation_progress = self.text_state.render_cache.as_ref()
            .and_then(|cache| cache.thresholds.as_ref())
            .filter(|_| self.text_state.reveal_enabled)
            .map(|map| {
                let elapsed = self.pal_time_ms.wrapping_sub(self.text_state.reveal_start_ms);
                (u64::from(elapsed) * u64::from(map.extent) / u64::from(self.text_state.reveal_duration_ms.max(1))) as u32
            });
        let reveal_limit = if animation_progress.is_some() { None } else { reveal_limit };
        self.activate_tips_records(false);
        let wait_frame = if self.text_state.show_wait_mark && reveal_limit.is_none() && animation_progress.is_none() {
            Some(self.pal_time_ms / 180)
        } else {
            None
        };
        let frame_key = AdvTextPresentedFrame {
            reveal_limit,
            animation_progress,
            wait_mark_frame: wait_frame,
        };
        if rebuilt
            || self.text_state.presented_frame != Some(frame_key)
            || self.text_state.sprite.is_none()
        {
            let composed = self.text_state.render_cache.as_ref().map(|cache| {
                let wait_mark = wait_frame.map(|frame| {
                    let color = argb_to_rgba_bytes(self.text_state.text_color);
                    let (mark_w, mark_h, mark_rgba) = self
                        .text_state
                        .wait_mark_sheet
                        .as_ref()
                        .map(|sheet| wait_mark_frame(sheet, frame, color))
                        .unwrap_or_else(|| fallback_wait_mark(color));
                    let (mark_x, mark_y) = cache.text_end_position();
                    (mark_w, mark_h, mark_rgba, mark_x, mark_y)
                });
                compose_adv_text_frame(cache, reveal_limit, animation_progress, wait_mark)
            });
            if let Some((width, height, rgba, x, y)) = composed {
                // Native PAL draws ADV text windows above scene sprites but
                // below the Game.exe button layer; the synthetic sprite must
                // join the native depth band or it buries the ADV chrome.
                let z = adv_text_render_priority(self.sprite_priority_cursor);
                let position_z = 0;
                if let Some(handle) = self.text_state.sprite {
                    let _ = sprites.replace_sprite_surface(
                        handle,
                        width,
                        height,
                        rgba,
                        "adv:text".to_owned(),
                    );
                    // Reveal and wait-mark frames only replace pixels. Moving
                    // the wrapper again would erase a committed text action.
                    if rebuilt {
                        let _ = sprites.set_pos(handle, x, y, position_z);
                    }
                    let _ = sprites.set_priority(handle, z);
                    let _ = sprites.view_ctrl(handle, true);
                } else if let Some(handle) = sprites.create_rgba_sprite(
                    width,
                    height,
                    rgba,
                    PalVec3::new(x, y, position_z),
                    z,
                    "adv:text".to_owned(),
                ) {
                    self.text_state.sprite = Some(handle);
                }
                self.text_state.presented_frame = Some(frame_key);
            }
        }
        self.sync_adv_button_chrome_visibility(sprites);
    }

    /// Rasterize the whole ADV body text once and split the result into a
    /// base panel plus per-line glyph geometry. Reveal and wait-mark frames
    /// then only re-clip these cached pixels.
    fn rebuild_adv_text_render_cache(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        mut resource_manager: Option<&mut ResourceManager>,
        body: &str,
    ) {
        let saved_size = self.font_state.font_size();
        let saved_color = self.font_state.color();
        let native_text_size = self.text_state.init_font_size().max(1) as u16;
        self.font_state.set_font_size(native_text_size.max(22));
        self.font_state.set_color(
            self.text_state.text_color,
            self.text_state.text_effect_color,
        );
        let (full_text, temporary_size) = parse_pal_text_directives(body);
        if let Some(size) = temporary_size {
            self.font_state.set_font_size(size);
        }
        let wrap_width = self.text_state.init_text_width().max(1) as u32;
        let (panel_text_width, panel_text_height, full_lines) =
            measure_wrapped_text(&self.font_state, &full_text, wrap_width);
        let (text_width, text_height, mut text_rgba, lines) = rasterize_text_block_with_layout(
            &self.font_state,
            &full_lines,
            panel_text_width,
            panel_text_height,
        );
        self.capture_tips_layout(body, &lines);
        self.apply_tips_colors(body, &full_lines, &lines, text_width, text_height, &mut text_rgba);
        let full_char_count = lines.iter().map(|line| line.char_count).sum();
        if let Some(manager) = resource_manager.as_mut() {
            self.ensure_wait_mark_sheet(manager);
        }
        let base_image =
            self.load_text_base_image(assets, nls, resource_manager, self.text_state.base);
        let base_width = base_image.as_ref().map(|image| image.width).unwrap_or(0);
        let has_base_image = base_image.is_some();
        let (logical_width, _) = self.logical_size();
        let x = if has_base_image && base_width >= logical_width {
            // TextInit's mid-field x values describe the body/name text area.
            // Full-width ADV bases such as MAIN_BASE00.PGD are PAL window
            // surfaces and must stay anchored at the logical screen origin.
            0
        } else if self.text_state.init_body_x() != 0 {
            self.text_state.init_body_x()
        } else if self.text_state.rect[0] != 0 {
            self.text_state.rect[0]
        } else {
            72
        };
        let text_draw_x = if self.text_state.init_body_x() != 0 {
            self.text_state.init_body_x()
        } else {
            x + 24
        };
        let text_draw_y = if self.text_state.init_body_y() != 0 {
            self.text_state.init_body_y()
        } else {
            self.text_state.rect[1].saturating_add(18)
        };
        let y = if has_base_image {
            // MAIN_BASE00 is a full-width PAL window base.  Game.exe keeps the
            // window base anchored at TextInit's name-y lane and draws body
            // glyphs at the separate body-y lane inside that surface.
            self.text_state.init_name_y()
        } else if text_draw_y != 0 {
            text_draw_y.saturating_sub(18)
        } else if self.text_state.rect[1] != 0 {
            self.text_state.rect[1]
        } else {
            558
        };
        let text_origin_x = text_draw_x.saturating_sub(x).max(0) as u32;
        // PAL leaves one line-gap above the ADV glyph cell. The bitmap font's
        // tallest glyphs start at row zero, so omitting this leading makes the
        // line sit visibly higher than the original renderer.
        let text_leading = (u32::from(native_text_size) / 4).max(1);
        let text_origin_y =
            (text_draw_y.saturating_sub(y).max(0) as u32).saturating_add(text_leading);
        let min_width = self.text_state.init_text_width().max(760) as u32;
        let (panel_width, panel_height, panel_rgba, fallback_origin_x, fallback_origin_y) =
            adv_text_base_panel(
                panel_text_width,
                panel_text_height,
                min_width,
                88,
                base_image,
                self.text_state.alpha,
            );
        let text_origin_x = if text_origin_x == 0 {
            fallback_origin_x
        } else {
            text_origin_x
        };
        let text_origin_y = if text_origin_y == 0 {
            fallback_origin_y
        } else {
            text_origin_y
        };
        self.text_state.render_cache = Some(AdvTextPanelCache {
            panel_width,
            panel_height,
            panel_rgba,
            text_width,
            thresholds: self.registered_text_animation.map(|mode| TextThresholdMap::build(mode, text_width, text_height, &lines, self.font_state.effect())),
            text_rgba,
            text_origin_x,
            text_origin_y,
            lines,
            full_char_count,
            sprite_x: x,
            sprite_y: y,
        });
        self.font_state.set_font_size(saved_size);
        self.font_state.set_color(saved_color.0, saved_color.1);
    }

    fn sync_adv_name_sprite(&mut self, name: &str, sprites: &mut SpriteSystem) {
        if name.is_empty() {
            if let Some(handle) = self.text_state.name_sprite.take() {
                let _ = sprites.release(handle);
            }
            return;
        }
        let saved_size = self.font_state.font_size();
        let saved_color = self.font_state.color();
        self.font_state.set_font_size(22);
        let (name, _) = parse_pal_text_directives(name);
        let (name_width, name_height, name_rgba) = self.font_state.rasterize(&name);
        self.font_state.set_font_size(saved_size);
        self.font_state.set_color(saved_color.0, saved_color.1);
        let name_surface_width = name_width.max(1);
        let name_surface_height = name_height.max(1);
        let (x, y) = match self.text_state.render_cache.as_ref() {
            Some(cache) => (cache.sprite_x, cache.sprite_y),
            None => return,
        };
        let native_text_size = self.text_state.init_font_size().max(1) as u16;
        let text_leading = (u32::from(native_text_size) / 4).max(1);
        let name_x = if self.text_state.init_name_x() != 0 {
            self.text_state.init_name_x()
        } else {
            x + 18
        };
        let name_y = if self.text_state.init_name_y() != 0 {
            self.text_state.init_name_y()
        } else {
            y.saturating_sub(name_surface_height as i32 + 8)
        }
        .saturating_add(text_leading as i32);
        // The native name text is painted over the nameplate/VOICE button
        // surface. Anchor the synthetic sprite just above whichever visible
        // ADV chrome button sits under the name origin, so popup screens and
        // their lanes still cover the name when the text window is hidden.
        let name_z = self
            .game_buttons
            .iter()
            .filter(|((group, _), entry)| *group == 0 && entry.visible)
            .filter_map(|(_, entry)| {
                let sprite = sprites.get(entry.handle)?;
                if !sprite.visible {
                    return None;
                }
                let pos = sprite.effective_position();
                let right = pos.x.saturating_add(sprite.source_rect.width());
                let bottom = pos.y.saturating_add(sprite.source_rect.height());
                let center_x = name_x.saturating_add(name_surface_width as i32 / 2);
                let center_y = name_y.saturating_add(name_surface_height as i32 / 2);
                (center_x >= pos.x && center_x < right && center_y >= pos.y && center_y < bottom)
                    .then(|| sprite.effective_priority())
            })
            .max()
            .map(|priority| priority.saturating_add(1))
            .unwrap_or_else(|| {
                adv_text_render_priority(self.sprite_priority_cursor).saturating_add(1)
            });
        let position_z = 0;
        if let Some(handle) = self.text_state.name_sprite {
            let _ = sprites.replace_sprite_surface(
                handle,
                name_surface_width,
                name_surface_height,
                name_rgba,
                "adv:name".to_owned(),
            );
            let _ = sprites.set_pos(handle, name_x, name_y, position_z);
            let _ = sprites.set_priority(handle, name_z);
            let _ = sprites.view_ctrl(handle, true);
        } else if let Some(handle) = sprites.create_rgba_sprite(
            name_surface_width,
            name_surface_height,
            name_rgba,
            PalVec3::new(name_x, name_y, position_z),
            name_z,
            "adv:name".to_owned(),
        ) {
            self.text_state.name_sprite = Some(handle);
        }
    }

    /// Current reveal clip as `(line index, pixel x limit)`, or `None` when
    /// the whole body text is visible. The limit interpolates inside the
    /// current glyph so the wipe moves smoothly instead of per character.
    fn adv_text_reveal_limit(&mut self) -> Option<(usize, u32)> {
        if !self.text_state.reveal_enabled {
            return None;
        }
        let elapsed = self
            .pal_time_ms
            .wrapping_sub(self.text_state.reveal_start_ms);
        let duration = self.text_state.reveal_duration_ms.max(1);
        if elapsed >= duration {
            self.text_state.reveal_enabled = false;
            return None;
        }
        let cache = self.text_state.render_cache.as_ref()?;
        let total = cache.full_char_count.max(1) as f64;
        let visible = (f64::from(elapsed) / f64::from(duration)) * total;
        for (index, line) in cache.lines.iter().enumerate() {
            if line.char_count == 0 {
                continue;
            }
            let start = line.char_start as f64;
            let end = start + line.char_count as f64;
            if visible >= end {
                continue;
            }
            if visible <= start {
                return Some((index, 0));
            }
            let local = visible - start;
            let char_index = (local.floor() as usize).min(line.char_count - 1);
            let frac = (local - char_index as f64) as f32;
            let x0 = line.char_x[char_index] as f32;
            let x1 = line.char_x[char_index + 1] as f32;
            return Some((index, (x0 + (x1 - x0) * frac).max(0.0) as u32));
        }
        None
    }

    fn sync_adv_button_chrome_visibility(&mut self, sprites: &mut SpriteSystem) {
        let chrome_visible = self.text_state.initialized
            && self.text_state.visible
            && !self.history_state.active
            && self
                .text_state
                .sprite
                .is_some_and(|handle| sprites.get(handle).is_some_and(|sprite| sprite.visible));
        let menu_bases = self.adv_menu_bases(sprites);
        // Some scripts leave quick-save/load at alpha zero while fading the
        // rest of the ADV toolbar through the native button table. Mirror the
        // live alpha of its visible peers, including when the toolbar fades
        // away, instead of restoring those two controls to 255 every frame.
        let inline_quick_peer_alpha = self
            .game_buttons
            .iter()
            .filter(|((group, _), entry)| {
                *group == 0
                    && entry.visible
                    && entry.enabled
                    && !entry.name.eq_ignore_ascii_case("MAIN_BTN_QSAVE")
                    && !entry.name.eq_ignore_ascii_case("MAIN_BTN_QLOAD")
            })
            .filter_map(|(_, entry)| sprites.get(entry.handle).map(|sprite| sprite.color.alpha()))
            .max()
            .unwrap_or(0);
        for ((group, _), entry) in self.game_buttons.iter() {
            if *group != 0 {
                continue;
            }
            // A paired compact/expanded base defines an ADV menu. Keep its
            // compact button visible while the script's separate panel sprites
            // animate, and reveal panel controls only after a menu click.
            let inline_quick_button = menu_bases.is_none()
                && (entry.name.eq_ignore_ascii_case("MAIN_BTN_QSAVE")
                    || entry.name.eq_ignore_ascii_case("MAIN_BTN_QLOAD"));
            let visible = if let Some((compact, expanded, panel_left)) = menu_bases {
                if entry.handle == compact {
                    chrome_visible && entry.visible && !self.adv_menu_expanded
                } else if entry.handle == expanded {
                    chrome_visible && entry.visible && self.adv_menu_expanded
                } else {
                    let on_panel = sprites
                        .get(entry.handle)
                        .is_some_and(|sprite| sprite.effective_position().x >= panel_left);
                    chrome_visible
                        && entry.visible
                        && entry.enabled
                        && (!on_panel || self.adv_menu_expanded)
                }
            } else {
                chrome_visible && entry.visible && entry.enabled
            };
            let compatible_quick_alpha = inline_quick_button && entry.alpha == 0;
            let visible = visible && (!compatible_quick_alpha || inline_quick_peer_alpha > 0);
            let _ = sprites.view_ctrl(entry.handle, visible);
            if compatible_quick_alpha {
                sprites.set_alpha(entry.handle, inline_quick_peer_alpha);
            }
        }
    }

    fn adv_menu_bases(&self, sprites: &SpriteSystem) -> Option<(SpriteHandle, SpriteHandle, i32)> {
        let compact = self.game_buttons.iter().find(|((group, _), entry)| {
            *group == 0 && entry.name.eq_ignore_ascii_case("MAIN_BTN_BASE0")
        })?.1;
        let expanded = self.game_buttons.iter().find(|((group, _), entry)| {
            *group == 0 && entry.name.eq_ignore_ascii_case("MAIN_BTN_BASE1")
        })?.1;
        let panel_left = sprites.get(expanded.handle)?.effective_position().x;
        Some((compact.handle, expanded.handle, panel_left))
    }

    pub fn consume_text_reveal_push(&mut self, input: &PalInputState) -> bool {
        if !(input.click_push() || input.fast_forward_held()) || self.text_reveal_remaining_ms() == 0
        {
            return false;
        }
        let advance_wait = input.click_push();
        self.text_state.reveal_enabled = false;
        self.text_state.dirty = true;
        log::debug!(
            "[trace-text] reveal completed by input/fast-forward click_push={} fast_forward={} advance_wait={advance_wait}",
            input.click_push(),
            input.fast_forward_held()
        );
        advance_wait
    }

    pub fn text_reveal_remaining_ms(&self) -> u32 {
        if !self.text_state.visible || !self.text_state.reveal_enabled {
            return 0;
        }
        let elapsed = self
            .pal_time_ms
            .wrapping_sub(self.text_state.reveal_start_ms);
        self.text_state.reveal_duration_ms.saturating_sub(elapsed)
    }

    /// Portable mirror of koikake.exe GetSkipState (0x437800), the single hub
    /// every native wait/reveal/effect consumer polls each frame:
    ///   scene-skip latch -> -1; skip byte -> bit 0; wheel-down or
    ///   Space/Return/left-click push -> bit 3; held Ctrl -> bit 2.
    pub fn skip_state(&self, input: &PalInputState) -> i32 {
        if self.history_navigation.target.is_some() { return -17; }
        if self.scene_skip_active {
            return if self.input_registered_profile { -33 } else { -1 };
        }
        let mut state = 0i32;
        if self.text_skip_enabled {
            state |= 1;
        }
        if input.wheel_delta() > 0.0 {
            state |= 8;
        }
        let advance = if self.button_release_mode { input.raw_key_pull() } else { input.raw_key_push() };
        if input.raw_key_on() & (1 << 5) == 0 && advance & 0x800006 != 0 {
            state |= 8;
        }
        if input.fast_forward_held() {
            state |= 4;
        }
        state
    }

    /// The held/latched part of `skip_state`: true while skip mode, scene
    /// skip, or Ctrl fast-forward should keep completing waits every frame
    /// (as opposed to push edges, which the wait tasks observe directly).
    pub fn skip_active(&self, input: &PalInputState) -> bool {
        self.skip_engaged() || input.fast_forward_held()
    }

    /// Latched skip (skip byte or scene skip) after the read-only gate.
    ///
    /// koikake.exe consults the read bitmap in two places: 0x42C510 lets
    /// scene-skip bypass a line's display only when the line is already
    /// read, and 0x42CBD0 clears the text object's skip flag (text+0x85c)
    /// when an unread line appears while skip is latched.  With the gate
    /// open (task data +0xC != 0) every line skips; with it closed only
    /// read lines skip.  Ctrl fast-forward bypasses the gate and is
    /// composed by the caller.
    fn skip_engaged(&self) -> bool {
        if self.history_navigation.target.is_some() { return true; }
        if !(self.text_skip_enabled || self.scene_skip_active) {
            return false;
        }
        self.system_state.skip_gate() != 0 || self.current_line_read
    }

    /// koikake.exe 0x42CBD0: every displayed text-pool line updates the
    /// read bitmap and the current-line read flag, and posts the slot-1
    /// cancel message when an unread line appears under latched skip in
    /// read-only mode.
    fn note_displayed_line(&mut self, text_value: i32, assets: &CoreAssets) {
        let Some(id) = text_pool_record_id(text_value, assets) else {
            // Runtime-generated strings have no pool record; native only
            // gates pool lines, so treat them as read.
            self.current_line_read = true;
            return;
        };
        if self.read_text_ids.contains(&id) {
            self.current_line_read = true;
            return;
        }
        self.current_line_read = false;
        self.read_text_ids.insert(id);
        if self.system_state.skip_gate() == 0 && self.text_skip_enabled {
            log::debug!("[trace-text] skip gate: unread line {id}, posting slot1 cancel");
            self.post_engine_message(1, 0);
        }
    }

    /// Engine-side post to the category-23 message board (koikake.exe
    /// 0x421590): sets the slot flag and value; scripts poll the slot with
    /// get_message and read the value with get_message_param.
    fn post_engine_message(&mut self, slot: usize, value: i32) {
        let slot = slot.min(7);
        self.message_state.next_id = self.message_state.next_id.saturating_add(1).max(1);
        let message = GameMessage {
            active: true,
            id: self.message_state.next_id,
            value,
            param: value,
        };
        self.message_state.slots[slot] = message;
        self.message_state.queue.push(message);
    }

    /// Skip state for extcall handlers without input access: the frame-cached
    /// input bits re-combined with the live script latches, matching how
    /// native consumers poll GetSkipState at use time.  The latched lanes are
    /// read-gated like skip_engaged so transitions do not skip through
    /// unread text in read-only mode.
    fn dispatch_skip_state(&self) -> i32 {
        if self.history_navigation.target.is_some() { return -17; }
        if self.scene_skip_active && (self.system_state.skip_gate() != 0 || self.current_line_read)
        {
            return if self.input_registered_profile { -33 } else { -1 };
        }
        let mut state = self.last_skip_state;
        if self.text_skip_enabled
            && (self.system_state.skip_gate() != 0 || self.current_line_read)
        {
            state |= 1;
        }
        state
    }

    /// Finish the typewriter reveal immediately.  Native completes the reveal
    /// in the same frame its text tick sees a non-zero skip state
    /// (0x42CE40), without advancing the parked wait by itself.
    pub fn complete_text_reveal_for_skip(&mut self) {
        if self.text_reveal_remaining_ms() == 0 {
            return;
        }
        self.text_state.reveal_enabled = false;
        self.text_state.dirty = true;
        log::debug!("[trace-text] reveal completed by skip state");
    }

    /// True while any registered voice channel (category 13) is still
    /// playing.  Native auto-mode advance polls PalSoundPlayIs across the
    /// voice channels before letting the hold timer finish the wait
    /// (0x431180, consulted by 0x42F9D0).
    pub fn any_voice_playing(&self, audio: &AudioSystem) -> bool {
        self.game_audio
            .iter()
            .filter(|((category, _), _)| *category == 13)
            .any(|(_, handle)| audio.is_playing(*handle).unwrap_or(false))
    }

    pub fn sync_history_sprite(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        sprites: &mut SpriteSystem,
    ) {
        if !self.history_state.active {
            if let Some(handle) = self.history_state.sprite {
                let _ = sprites.view_ctrl(handle, false);
            }
            return;
        }
        self.sync_adv_button_chrome_visibility(sprites);

        let [left, top, right, bottom] = self.history_state.rect;
        let width = (right - left).max(1) as u32;
        let height = (bottom - top).max(1) as u32;
        let mut rgba = vec![0_u8; (width * height * 4) as usize];

        let saved_size = self.font_state.font_size();
        let saved_color = self.font_state.color();
        let (font_size, wrap_width, pitch, name_block, _, body_x, name_x) =
            self.history_typeset_params();
        self.font_state.set_font_size(font_size);

        // Native scroll positions are record indices: 0 anchors the newest
        // record at the bottom of the view, and each step toward `max` moves
        // one record further into the past.  Draw the window the shared
        // layout cache computed so wrapping, scroll math, and hit-testing all
        // agree.
        let hovered = self.history_state.hovered_record;
        let (start, _end, tops) = self.history_visible_window(assets, nls);
        let records = self
            .history_state
            .layout_cache
            .as_ref()
            .expect("history_visible_window installs the cache")
            .records
            .clone();
        let hover_text_color = self.history_state.colors[1] as u32;
        for (offset, record_top) in tops.iter().enumerate() {
            let record_index = start + offset;
            let record = &records[record_index];
            let record_top = *record_top;
            let body_top = record_top + if record.name.is_some() { name_block } else { 0 };
            let hovered_record = Some(record_index) == hovered && record.has_voice;
            if hovered_record && record.body_height > 0 {
                // Native hover feedback (history_update 0x41EF20): an opaque
                // dark plate (hardcoded 0xFF4E4E4E) behind the body block,
                // with the text redrawn in the second history color.
                let plate_top = body_top.max(0) as u32;
                let plate_height = record.body_height as u32;
                let plate_left = (body_x - 8).max(0) as u32;
                let plate_right = (body_x + wrap_width as i32 + font_size as i32)
                    .clamp(0, width as i32) as u32;
                for by in plate_top..(plate_top + plate_height).min(height) {
                    for bx in plate_left..plate_right {
                        let di = ((by * width + bx) * 4) as usize;
                        alpha_blend_rgba(&mut rgba[di..di + 4], &[0x4E, 0x4E, 0x4E, 0xFF]);
                    }
                }
            }
            let text_color = if hovered_record {
                hover_text_color
            } else {
                self.history_state.colors[0] as u32
            };
            self.font_state.set_color(text_color, 0x0000_0000);
            if let Some(name) = &record.name {
                let (line_width, line_raster_height, line_rgba) = self.font_state.rasterize(name);
                blit_rgba(
                    &mut rgba,
                    width,
                    height,
                    &line_rgba,
                    line_width,
                    line_raster_height,
                    name_x.max(0) as u32,
                    record_top.max(0) as u32,
                );
            }
            let mut y = body_top;
            for line in &record.body_lines {
                if y >= height as i32 {
                    break;
                }
                if !line.is_empty() {
                    let (line_width, line_raster_height, line_rgba) =
                        self.font_state.rasterize(line);
                    blit_rgba(
                        &mut rgba,
                        width,
                        height,
                        &line_rgba,
                        line_width,
                        line_raster_height,
                        body_x.max(0) as u32,
                        y.max(0) as u32,
                    );
                }
                y = y.saturating_add(pitch);
            }
        }

        self.font_state.set_font_size(saved_size);
        self.font_state.set_color(saved_color.0, saved_color.1);

        // The backlog overlays the ADV text window but stays under the native
        // button layer, following the popup lane the log screen opened on.
        let z = history_text_render_priority(self.sprite_priority_cursor);
        if let Some(handle) = self.history_state.sprite {
            let _ = sprites.replace_sprite_surface(handle, width, height, rgba, "history:text");
            let _ = sprites.set_pos(handle, left, top, 0);
            let _ = sprites.set_priority(handle, z);
            let _ = sprites.view_ctrl(handle, true);
        } else if let Some(handle) = sprites.create_rgba_sprite(
            width,
            height,
            rgba,
            PalVec3::new(left, top, 0),
            z,
            "history:text",
        ) {
            self.history_state.sprite = Some(handle);
        }
    }

    fn adv_text_parts_for_render(&self, assets: &CoreAssets, nls: Nls) -> (String, String) {
        let body = self
            .resolved_dialog_text_arg(self.text_body_value(), assets, nls)
            .unwrap_or_default();
        let name = self
            .resolved_dialog_text_arg(self.text_name_value(), assets, nls)
            .unwrap_or_default();
        (body, name)
    }

    fn text_body_value(&self) -> i32 {
        // Game.exe `AdvCommandText` (`sub_440390`) stores the third popped
        // value at text_ctx+4236 and passes it to `sub_43D770`, the ADV body
        // resolver. `ext_args_source_order` maps that native value to slot 1.
        self.text_state.last_text_args[1]
    }

    fn text_name_value(&self) -> i32 {
        // Native stores the second popped value at text_ctx+4232 and feeds it
        // to `sub_43D9D0`, the name lane resolver. In source order this is slot 2.
        self.text_state.last_text_args[2]
    }

    fn text_voice_value(&self) -> i32 {
        // Native stores the first popped value at text_ctx+4228 and passes it
        // to `sub_43C2C0`, which resolves the ADV voice/face lane. In source
        // order this is slot 3.
        self.text_state.last_text_args[3]
    }

    /// Game.exe `sub_43C2C0` resolves the ADV voice lane via `sub_44B1E0`,
    /// releases the current text voice group, then calls the same sound loader
    /// used by `voice_play`. IDB evidence: `sub_43C2C0 -> sub_442C10`, and
    /// `sub_4024D0(Str1, 1, 0)` loads voices as PAL sound type/group 1.
    /// Dynamic strings can also
    /// carry URLs such as `http://ustrack...`; native tries to resolve them as
    /// plain strings, but they are not voice resources, so the portable bridge
    /// refuses URL-like names instead of spamming failed audio loads.
    fn try_play_text_voice(
        &mut self,
        voice_value: i32,
        assets: &CoreAssets,
        nls: Nls,
        mut resource_manager: Option<&mut ResourceManager>,
        mut audio: Option<&mut AudioSystem>,
    ) {
        if voice_value == 0x0FFF_FFFF || !self.text_state.voice_enabled {
            log::debug!(
                "[trace-audio] text_voice skip value=0x{voice_value:08X} enabled={}",
                self.text_state.voice_enabled
            );
            return;
        }
        let Some(name) = self.resolve_resource_string(voice_value, assets, nls) else {
            log::debug!("[trace-audio] text_voice unresolved value=0x{voice_value:08X}");
            return;
        };
        let lower = name.to_ascii_lowercase();
        if name.is_empty()
            || is_resource_clear_sentinel(&name)
            || lower.starts_with("http:")
            || lower.starts_with("https:")
        {
            log::debug!("[trace-audio] text_voice skip name={name:?}");
            return;
        }
        // Game.exe 0x43C2C0 releases all eight ADV voice lanes before a
        // new line, including every member of the previous MIX.
        if let Some(audio) = audio.as_deref_mut() {
            self.release_adv_voice_slots(audio);
        }
        let unit_percent = resource_manager
            .as_deref_mut()
            .and_then(|rm| self.voice_unit_slot_for_name(rm, &name))
            .map(|slot| self.voice_unit_channel_percent(slot))
            .unwrap_or(100);
        let outcome = self.audio_load_and_play(
            13,
            0,
            PalSoundGroup::GROUP1,
            voice_value,
            0,
            unit_percent,
            true,
            None,
            assets,
            nls,
            resource_manager,
            audio,
            0,
        );
        log::debug!("[trace-audio] text_voice name={name:?} outcome={outcome:?}");
    }

    fn ensure_wait_mark_sheet(&mut self, resource_manager: &mut ResourceManager) {
        if self.text_state.wait_mark_sheet.is_some() || self.text_state.wait_mark_missing {
            return;
        }
        let name = self
            .system_ini
            .as_ref()
            .and_then(|ini| ini_first_str(ini, "ex_fontname"))
            .unwrap_or_else(|| "font_ex".to_owned());
        match open_resource_variant(resource_manager, &name, FONT_SHEET_EXTENSIONS) {
            Ok(asset) => match decode_image(&asset.bytes) {
                Ok(image) => {
                    if let Err(error) = self.font_state.load_extended_font(&asset.bytes) {
                        log::warn!("[trace-text] native extended font {:?} rejected: {error}", asset.name);
                    }
                    log::debug!(
                        "[trace-text] wait mark sheet {:?} {}x{}",
                        asset.name,
                        image.width,
                        image.height
                    );
                    self.text_state.wait_mark_sheet = Some(image);
                }
                Err(err) => {
                    log::warn!(
                        "[trace-text] wait mark sheet {:?} decode failed: {err}",
                        asset.name
                    );
                    self.text_state.wait_mark_missing = true;
                }
            },
            Err(err) => {
                log::debug!("[trace-text] wait mark sheet {name:?} open failed: {err}");
                self.text_state.wait_mark_missing = true;
            }
        }
    }

    fn load_text_base_image(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        base_value: i32,
    ) -> Option<DecodedImage> {
        if base_value == 0 || base_value == 0x0FFF_FFFF {
            self.text_state.base_image_value = base_value;
            self.text_state.base_image = None;
            return None;
        }
        if self.text_state.base_image_value == base_value {
            if let Some(image) = self.text_state.base_image.as_ref() {
                return Some(image.clone());
            }
        }
        self.text_state.base_image_value = base_value;
        self.text_state.base_image = None;
        let Some(name) = self.resolve_resource_string(base_value, assets, nls) else {
            log::debug!("[trace-text] text_set_base unresolved value={base_value}");
            return None;
        };
        if name.is_empty() {
            return None;
        }
        let Some(resource_manager) = resource_manager else {
            return None;
        };
        let asset = match open_resource_variant(resource_manager, &name, IMAGE_EXTENSIONS) {
            Ok(asset) => asset,
            Err(err) => {
                log::debug!("[trace-text] text_set_base name={name:?} image open failed: {err}");
                return None;
            }
        };
        match decode_asset_image(resource_manager, &asset) {
            Ok(decoded) => {
                log::debug!(
                    "[trace-text] text_set_base name={name:?} asset={:?} size={}x{}",
                    asset.name,
                    decoded.width,
                    decoded.height
                );
                self.text_state.base_image = Some(decoded.clone());
                Some(decoded)
            }
            Err(err) => {
                log::debug!(
                    "[trace-text] text_set_base name={name:?} asset={:?} decode failed: {err}",
                    asset.name
                );
                None
            }
        }
    }

    fn resolved_dialog_text_arg(
        &self,
        value: i32,
        assets: &CoreAssets,
        nls: Nls,
    ) -> Option<String> {
        if value == 0 || value == 0x0FFF_FFFF {
            return None;
        }
        let text = self.resolve_script_string(value, assets, nls)?;
        let text = parse_pal_text_directives(&text).0.trim().to_owned();
        if text.is_empty() || looks_like_media_or_resource_name(&text) {
            None
        } else {
            Some(text)
        }
    }

    fn text_wait_duration_ms(&self) -> u32 {
        let chars = self
            .text_state
            .last_text_args
            .iter()
            .copied()
            .filter(|value| *value != 0x0FFF_FFFF)
            .count()
            .max(1) as u32;
        500_u32.saturating_add(chars.saturating_mul(350)).min(3500)
    }

    /// Backlog typesetting parameters, recovered from koikake.exe
    /// historybegin (0x41F2C0): the records are typeset with the ADV text
    /// configuration from text_init — font size (arg8), body wrap width
    /// (arg5), pitch = font size + arg1 (or +2 when arg1 is 0).  history_init
    /// contributes the name/body x offsets (arg1/arg3), the name block height
    /// (arg4), and the inter-record gap (arg7).
    fn history_typeset_params(&self) -> (u16, u32, i32, i32, i32, i32, i32) {
        let layout = self.history_state.layout;
        let (font_size, wrap_width, pitch_extra) = if self.text_state.initialized {
            (
                self.text_state.init_args[7].max(1) as u16,
                self.text_state.init_args[4].max(1) as u32,
                self.text_state.init_args[0],
            )
        } else {
            let view = (self.history_state.rect[2] - self.history_state.rect[0]).max(1);
            (24, (view - layout[0].max(0) - layout[2].max(0)).max(1) as u32, 0)
        };
        let pitch = font_size as i32 + if pitch_extra != 0 { pitch_extra } else { 2 };
        (
            font_size,
            wrap_width,
            pitch.max(1),
            layout[3].max(0),
            layout[6].max(0),
            layout[2].max(0),
            layout[0].max(0),
        )
    }

    /// Rebuild the wrapped backlog layout when the record list or the view
    /// rect changed.  All wrapped-line consumers (height, capacity, max
    /// scroll pos, renderer) read this one cache so scroll math and drawing
    /// can never disagree.
    fn rebuild_history_layout(&mut self, assets: &CoreAssets, nls: Nls) {
        let generation = self.history_state.records_generation;
        let rect = self.history_state.rect;
        if self
            .history_state
            .layout_cache
            .as_ref()
            .is_some_and(|cache| cache.generation == generation && cache.rect == rect)
        {
            return;
        }

        let (font_size, wrap_width, pitch, name_block, record_gap, body_x, name_x) =
            self.history_typeset_params();

        let saved_size = self.font_state.font_size();
        let saved_color = self.font_state.color();
        self.font_state.set_font_size(font_size);
        self.font_state.set_color(0xFF20_2020, 0x0000_0000);

        let line_height = {
            let (_, probe_height, _) = self.font_state.rasterize("あ");
            (probe_height as i32).max(1)
        };
        let view_height = (rect[3] - rect[1]).max(1);

        let mut records = Vec::with_capacity(self.history_state.records.len());
        let mut total_height = 0i32;
        for (index, _record) in self.history_state.records.iter().enumerate() {
            let text = self.history_record_text(index, assets, nls);
            let body = parse_pal_text_directives(&text.body).0.trim().to_owned();
            let name = Some(parse_pal_text_directives(&text.name).0.trim().to_owned())
                .filter(|name| !name.is_empty());
            let body_lines = if body.is_empty() {
                Vec::new()
            } else {
                wrap_text_lines(&self.font_state, &body, wrap_width)
            };
            let body_height = body_lines.len() as i32 * pitch;
            let slot_height = if body_height == 0 && name.is_none() {
                0
            } else {
                name.as_ref().map_or(0, |_| name_block) + body_height + record_gap
            };
            total_height = total_height.saturating_add(slot_height);
            records.push(HistoryRecordLayout {
                name,
                body_lines,
                body_height,
                slot_height,
                has_voice: !text.voice.is_empty(),
            });
        }
        total_height = total_height.saturating_sub(record_gap).max(0);

        // apply_scroll fills from the newest record upward and drops a record
        // whose top would cross the view top, so the capacity is exactly the
        // tail-fill count.
        let mut y = view_height;
        let mut visible_from_bottom = 0usize;
        for record in records.iter().rev() {
            y -= record.slot_height;
            if y < 0 {
                break;
            }
            visible_from_bottom += 1;
        }
        let max_pos = records.len().saturating_sub(visible_from_bottom) as i32;

        self.font_state.set_font_size(saved_size);
        self.font_state.set_color(saved_color.0, saved_color.1);

        self.history_state.height = total_height;
        self.history_state.layout_cache = Some(HistoryLayoutCache {
            generation,
            rect,
            line_height,
            pitch,
            name_block,
            record_gap,
            body_x,
            name_x,
            records,
            total_height,
            visible_from_bottom,
            max_pos,
        });
    }

    /// Records visible at the current scroll position: `(start, end, tops)`
    /// where `tops[i]` is the view-relative top y of record `start + i`'s
    /// name/body stack.  The window ends `pos` records before the newest and
    /// fills bottom-up; like native apply_scroll (0x41F800), a record whose
    /// top would cross the view top is dropped rather than clipped.
    fn history_visible_window(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
    ) -> (usize, usize, Vec<i32>) {
        self.rebuild_history_layout(assets, nls);
        let (rect, records, max_pos) = {
            let cache = self
                .history_state
                .layout_cache
                .as_ref()
                .expect("rebuild_history_layout always installs the cache");
            (cache.rect, cache.records.clone(), cache.max_pos)
        };
        let count = records.len();
        let view_height = (rect[3] - rect[1]).max(1);
        let pos = (self.history_state.scroll_y.max(0) as usize).min(max_pos.max(0) as usize);
        let end = count.saturating_sub(pos);
        // Stack from the bottom of the view upward.
        let mut y = view_height;
        let mut start = end;
        let mut tops_rev = Vec::new();
        for index in (0..end).rev() {
            let slot = records[index].slot_height;
            let top = y - slot;
            if top < 0 {
                break;
            }
            tops_rev.push(top);
            y = top;
            start = index;
        }
        tops_rev.reverse();
        (start, end, tops_rev)
    }

    /// Wrapped layout, rebuilding when stale.
    fn history_layout(&mut self, assets: &CoreAssets, nls: Nls) -> &HistoryLayoutCache {
        self.rebuild_history_layout(assets, nls);
        self.history_state
            .layout_cache
            .as_ref()
            .expect("rebuild_history_layout always installs the cache")
    }

    /// How many record lines fit into the history view rect.

    fn push_history_text_record(&mut self, text_args: [i32; 4], assets: &CoreAssets, nls: Nls) {
        if !self.text_state.history_enabled {
            return;
        }
        let text_value = text_args[1];
        if text_value == 0 || text_value == 0x0FFF_FFFF {
            return;
        }
        let frozen = self.resolve_history_text(text_args, assets, nls);
        self.history_state.texts.resize(self.history_state.records.len(), None);
        self.history_state.icons.records.resize(self.history_state.records.len(), Default::default());
        // Native history records carry text/name/voice/face fields.  Store the
        // script string handles in source order so the history UI and save-data
        // layer can recover the same resources later.  The native record array
        // holds 128 entries (0x80 * 0x114C bytes); when full the oldest record
        // is shifted out.
        if self.history_state.records.len() >= 128 {
            self.history_state.records.remove(0);
            self.history_state.texts.remove(0);
            self.history_state.icons.records.remove(0);
        }
        self.history_state.texts.push(Some(frozen));
        self.history_state.icons.records.push(history_icons::RecordIcons {
            face: self.history_state.icons.config.face_capture,
            checkpoint: if self.history_state.icons.config.secondary_enabled {
                self.history_state.icons.current_checkpoint
            } else { -1 },
        });
        self.history_state.records.push([
            text_args[0],
            text_args[1],
            text_args[2],
            text_args[3],
            // Native history+4416 stores the PC captured before executing
            // this 12-byte text extcall (Game.exe 0x44A2F0 / 0x43F900).
            self.pc.saturating_sub(12) as i32,
            self.text_state.icon,
            self.text_state.base,
            self.text_state.mode,
            self.pal_time_ms as i32,
        ]);
        self.history_state.records_generation += 1;
        // The wrapped layout cache is rebuilt lazily by history_begin or the
        // next layout query; no eager metric work is needed per line.
    }

    fn text_reveal_duration_ms(
        &self,
        text_value: i32,
        explicit_duration_ms: i32,
        assets: &CoreAssets,
        nls: Nls,
    ) -> u32 {
        if explicit_duration_ms > 0 {
            return explicit_duration_ms as u32;
        }
        let char_count = self
            .resolved_dialog_text_arg(text_value, assets, nls)
            .map(|text| parse_pal_text_directives(&text).0.chars().count())
            .or_else(|| {
                dynamic_string_index(text_value)
                    .and_then(|idx| self.dynamic_strings.get(idx))
                    .map(|text| parse_pal_text_directives(text).0.chars().count())
            })
            .unwrap_or(0)
            .max(1) as u32;
        // Game.exe `text_w` (`sub_43FFC0`) stores the explicit duration in
        // text_ctx+4196.  When that duration is zero, native computes the
        // reveal span from the configured message speed (task data +0x20)
        // and the line length in font-height units, the same shape as the
        // auto-mode hold (`speed * (len*24/font_height)`).  A speed of zero
        // (slider at MAX) reveals instantly.
        let speed = self.system_state.text_speed_percent().max(0) as u32;
        if speed == 0 {
            return 0;
        }
        let font_height = self.text_state.init_font_size().max(1) as u32;
        let line_units = char_count.saturating_mul(24) / font_height;
        speed.saturating_mul(line_units.max(1)).clamp(60, 8000)
    }

    fn text_auto_hold_duration_ms(&self, text_value: i32, assets: &CoreAssets, nls: Nls) -> u32 {
        let char_count = self
            .resolved_dialog_text_arg(text_value, assets, nls)
            .map(|text| parse_pal_text_directives(&text).0.chars().count())
            .or_else(|| {
                dynamic_string_index(text_value)
                    .and_then(|idx| self.dynamic_strings.get(idx))
                    .map(|text| parse_pal_text_directives(text).0.chars().count())
            })
            .unwrap_or(1)
            .max(1) as u32;
        let font_height = self.text_state.init_font_size().max(1) as u32;
        let line_units = char_count.saturating_mul(24) / font_height;
        // Native `sub_43A860` switches from reveal to the post-text wait flag
        // and computes `TaskData+28 * (text_len / font_height) + 500`.
        (self.system_state.auto_speed_percent().max(0) as u32)
            .saturating_mul(line_units.max(1))
            .saturating_add(500)
            .max(500)
    }

    fn text_wait_request_after_submit(
        &mut self,
        text_value: i32,
        reveal_ms: u32,
        assets: &CoreAssets,
        nls: Nls,
    ) -> WaitRequest {
        self.begin_tips_text();
        self.note_displayed_line(text_value, assets);
        let auto_ms = self.text_auto_hold_duration_ms(text_value, assets, nls);
        self.text_state.current_reveal_ms = reveal_ms;
        self.text_state.current_auto_hold_ms = auto_ms;
        if self.skip_engaged() {
            // Native skip completes the reveal in the same frame its text
            // tick sees the skip state and advances without a click; the
            // engine completes the reveal and the task layer completes the
            // wait through the live skip check.
            return WaitRequest::TextReveal(1);
        }
        if self.text_auto_enabled {
            // Native arms the auto hold when the post-reveal wait begins
            // (text_len/font_unit * auto_speed% + 500ms); the engine adds
            // the remaining reveal span when it creates the wait task, so
            // only the hold belongs in the request.
            return WaitRequest::AutoClickOrTime(auto_ms.max(1));
        }
        WaitRequest::Click
    }

    pub fn update_button_input_state(
        &mut self,
        sprites: &mut SpriteSystem,
        input: &PalInputState,
    ) -> ButtonInputOutcome {
        self.update_button_input_state_with_audio(sprites, input, None)
    }

    pub(crate) fn update_button_input_state_with_audio(
        &mut self, sprites: &mut SpriteSystem, input: &PalInputState,
        audio: Option<&mut AudioSystem>,
    ) -> ButtonInputOutcome {
        if self.system_button_blocked_at.is_some_and(|start|
            self.pal_time_ms.wrapping_sub(start) >= self.button_input_block_duration) {
            self.system_button_blocked_at = None;
            self.button_input_block_duration = 0;
        }
        if self.button_run_skip != 0 { return ButtonInputOutcome::default(); }
        let outcome = self.poll_button_input_state(sprites, input, audio);
        // Native locked groups only clear their elapsed latch on this visit;
        // reaction polling resumes on the following visit.
        self.expire_button_locks();
        outcome
    }

    fn poll_button_input_state(
        &mut self, sprites: &mut SpriteSystem, input: &PalInputState,
        mut audio: Option<&mut AudioSystem>,
    ) -> ButtonInputOutcome {
        if !input.mouse_on(PalMouseButton::Left) { self.pressed_button = None; }
        if self.system_button_blocked_at.is_some_and(|start|
            self.pal_time_ms.wrapping_sub(start) < self.button_input_block_duration) {
            return ButtonInputOutcome::default();
        }
        let completed_reaction = self.poll_button_reactions(audio.as_deref_mut(), input);
        let pending_reaction = self.button_reaction_pending();
        let (mouse_x, mouse_y) = input.mouse_position();
        if input.mouse_push(PalMouseButton::Left)
            && !self.adv_menu_expanded
            && self.adv_menu_bases(sprites).is_some_and(|(compact, _, _)| {
                sprites.get(compact).is_some_and(|sprite| {
                    let pos = sprite.effective_position();
                    sprite.visible
                        && sprite.color.alpha() != 0
                        && mouse_x >= pos.x
                        && mouse_x < pos.x.saturating_add(sprite.source_rect.width())
                        && mouse_y >= pos.y
                        && mouse_y < pos.y.saturating_add(sprite.source_rect.height())
                })
            })
        {
            self.adv_menu_expanded = true;
            self.sync_adv_button_chrome_visibility(sprites);
            return ButtonInputOutcome {
                consumed_mouse_push: true,
                ..Default::default()
            };
        }
        if input.mouse_push(PalMouseButton::Left)
            && self.adv_menu_expanded
            && self.adv_menu_bases(sprites).is_some_and(|(_, expanded, _)| {
                sprites.get(expanded).is_some_and(|sprite| {
                    let pos = sprite.effective_position();
                    sprite.visible
                        && mouse_x >= pos.x
                        && mouse_x < pos.x.saturating_add(sprite.source_rect.width())
                        && mouse_y >= pos.y
                        && mouse_y < pos.y.saturating_add(40)
                })
            })
        {
            self.adv_menu_expanded = false;
            self.sync_adv_button_chrome_visibility(sprites);
            return ButtonInputOutcome {
                consumed_mouse_push: true,
                ..Default::default()
            };
        }
        // The EXE polls groups in ascending order. Overlapping groups each
        // retain hover state; the first pending physical reaction clears the
        // input snapshot before subsequent groups can react to that edge.
        let groups: BTreeSet<_> = self.button_groups.keys().copied()
            .chain(self.game_buttons.keys().map(|&(group, _)| group)).collect();
        let hovered_by_group: Vec<_> = groups.into_iter().filter(|group|
            !self.button_groups.get(group).is_some_and(|g| g.locked)
                && !self.pending_button_reactions.contains_key(group))
            .map(|group| (group, self.button_hit_at(sprites, mouse_x, mouse_y, group).map(|(_, i)| i)))
            .collect();
        let hovered = hovered_by_group.iter().find_map(|&(group, index)| index.map(|i| (group, i)));
        let held = input.mouse_on(PalMouseButton::Left);
        if !held { self.pressed_button = None; }
        let mut outcome = ButtonInputOutcome {
            consumed_mouse_capture: pending_reaction,
            consumed_push_edge: completed_reaction,
            ..Default::default()
        };
        let left_push_hit_button = input.mouse_push(PalMouseButton::Left) && hovered.is_some();
        if left_push_hit_button {
            self.pressed_button = hovered;
            if self.button_release_mode { outcome.consumed_mouse_capture = true; }
        }
        let reaction = if pending_reaction { None } else if self.button_release_mode {
            if held {
                for &(group, index) in &hovered_by_group {
                    if let Some(index) = index { self.button_group_release_capture.insert(group, index); }
                    else { self.button_group_release_capture.remove(&group); }
                }
                self.button_release_capture = hovered;
                None
            } else {
                let result = hovered_by_group.iter().find_map(|&(group, index)|
                    index.filter(|_| self.button_group_release_capture.contains_key(&group)).map(|i| (group, i)));
                self.button_group_release_capture.clear();
                self.button_release_capture = None;
                result
            }
        } else {
            hovered_by_group.iter().find_map(|&(group, index)| index.and_then(|index| {
                let entry = self.game_buttons.get(&(group, index))?;
                let eligible = if entry.native_flags & 1 != 0 {
                    input.mouse_push(PalMouseButton::Left)
                } else {
                    entry.native_flags & 2 != 0
                        && (input.raw_key_push() | input.raw_key_on_ex()) & (1 << 23) != 0
                };
                eligible.then_some((group, index))
            }))
        };
        if let Some((group, index)) = reaction {
            self.button_sound_hover.insert(group, index);
            self.begin_button_reaction(group, index, false, audio.as_deref_mut());
            outcome.consumed_mouse_capture = true;
            log::debug!("[trace-button] pending group={group} index={index} release={} pos=({mouse_x},{mouse_y})", self.button_release_mode);
        }
        let hover_callbacks: Vec<_> = hovered_by_group.iter().copied()
            .filter(|(group, _)| reaction.is_none_or(|(pending, _)| pending != *group)).collect();
        for &(group, index) in &hover_callbacks {
            self.update_button_hover_sound(group, index, audio.as_deref_mut());
        }
        outcome.callback_queued = self.update_button_hover_callbacks(&hover_callbacks, input);
        // Native shortcut polling walks slots in ascending order and dispatches
        // only the first match. Older EXEs reject every stable held key; the
        // registered build permits held arrows. Repeat is a PAL output mask.
        let held_gate = if self.input_registered_profile { !0x780 } else { u32::MAX };
        let blocked = self.system_button_blocked_at.is_some_and(|start|
            self.pal_time_ms.wrapping_sub(start) < 50);
        if !self.button_reaction_pending() && !blocked && input.raw_key_on() & held_gate == 0 {
            let release = self.input_release_shortcuts && self.button_release_mode;
            for slot in 0..32 {
                let Some(entry) = self.system_buttons.get(&slot) else { continue; };
                if !entry.enabled { continue; }
                let mouse_slot = slot == 0 || slot == 3;
                let edges = if mouse_slot && release { input.raw_key_pull() } else { input.raw_key_push() };
                let candidate = edges | (entry.state as u32 & input.raw_key_on_ex());
                let mask = match slot {
                    0 => 1 << 24,
                    3 if self.input_registered_profile => 0x4080_0000,
                    3 => 1 << 23,
                    4..=15 => 1 << (slot + 7),
                    16 => 1,
                    17..=20 => 1 << (slot - 10),
                    _ => 0,
                };
                let matches = match slot {
                    1 => input.wheel_delta() < 0.0,
                    2 => input.wheel_delta() > 0.0,
                    21..=31 => true,
                    _ => candidate & mask != 0,
                };
                if !matches || (slot == 3 && (left_push_hit_button || reaction.is_some())) {
                    continue;
                }
                // A matching slot with a zero point ends native polling too.
                if self.queue_system_button_shortcut(slot, input) {
                    if mouse_slot && release {
                        outcome.consumed_mouse_pull = true;
                    } else if mouse_slot {
                        outcome.consumed_mouse_push = true;
                    } else {
                        outcome.consumed_push_edge = true;
                    }
                }
                break;
            }
        }

        let keys = self.game_buttons.keys().copied().collect::<Vec<_>>();
        for key @ (group, index) in keys {
            let Some(entry) = self.game_buttons.get(&key).cloned() else {
                continue;
            };
            let flags = self.effective_button_flags(entry.native_flags);
            if !entry.visible || !entry.enabled || entry.locked || flags & 4 != 0 {
                continue;
            }
            let Some(sprite) = sprites.get(entry.handle) else {
                continue;
            };
            if !sprite.visible || sprite.color.alpha() == 0 {
                continue;
            }
            // Lock is input-only in native (sub_40E0C0 stores a group lock
            // flag); it must not change the rendered cell.  Koikake locks
            // groups 4/5 while its shortcut dropdown is open, and treating
            // locked as the disabled cell made every option render its
            // "selected" frame at once.
            let state = if reaction == Some((group, index)) {
                2
            } else if hovered_by_group.iter().any(|&(g, i)| g == group && i == Some(index)) {
                1
            } else {
                0
            };
            sprites.rect_set_pos(entry.handle, 0, entry.toggle.wrapping_add(state));
        }
        outcome
    }

    /// Queue the first eligible shortcut and arm the shared 50 ms idle gate.
    fn queue_system_button_shortcut(&mut self, slot: i32, input: &PalInputState) -> bool {
        let Some(entry) = self.system_buttons.get(&slot) else { return false; };
        if !entry.enabled || entry.image == 0 { return false; }
        let point_id = entry.image as u32;
        if self.skip_state(input) == 0 {
            self.system_button_blocked_at = Some(self.pal_time_ms);
            self.button_input_block_duration = 50;
        }
        self.pending_gosub_points.push_back(point_id);
        let (mouse_x, mouse_y) = input.mouse_position();
        log::debug!(
            "[trace-system-button] shortcut slot={slot} -> queued gosub point[{point_id}] pos=({mouse_x},{mouse_y})"
        );
        true
    }

    pub fn run_frame(
        &mut self,
        assets: &CoreAssets,
        config: &ScriptRuntimeConfig,
    ) -> Result<RuntimeTick, RuntimeError> {
        self.run_frame_with_resources(assets, None, None, None, None, None, config)
    }

    pub fn run_frame_with_resources(
        &mut self,
        assets: &CoreAssets,
        mut resource_manager: Option<&mut ResourceManager>,
        mut sprites: Option<&mut SpriteSystem>,
        mut task_system: Option<&mut TaskSystem>,
        mut audio: Option<&mut AudioSystem>,
        input: Option<&PalInputState>,
        config: &ScriptRuntimeConfig,
    ) -> Result<RuntimeTick, RuntimeError> {
        if self.movie_active() {
            return Ok(RuntimeTick { executed: 0, status: self.status.clone(), wait_request: None, frame_events: Vec::new() });
        }
        self.begin_script_input_frame();
        // A restored save re-arms its BGM before any wait-state early return,
        // because a snapshot parked at an ADV click wait would otherwise never
        // reach the script loop that can see the audio backend.
        self.replay_restored_bgm(resource_manager.as_deref_mut(), audio.as_deref_mut());
        self.replay_restored_bgv(resource_manager.as_deref_mut(), audio.as_deref_mut());
        self.update_voice_autopan(sprites.as_deref(), audio.as_deref_mut());
        self.update_bgv(audio.as_deref_mut());
        // Cache this frame's GetSkipState value so extcall handlers that run
        // without input access mirror the native per-frame skip polling.
        if let Some(input) = input {
            self.last_skip_state = self.skip_state(&self.filter_script_input(input));
        }
        match self.status {
            RuntimeStatus::Halted { .. }
            | RuntimeStatus::UnsupportedCommand { .. }
            | RuntimeStatus::UnsupportedExtCall { .. }
            | RuntimeStatus::Faulted { .. } => {
                return Ok(RuntimeTick {
                    executed: 0,
                    status: self.status.clone(),
                    wait_request: None,
                    frame_events: Vec::new(),
                });
            }
            // Still waiting on a task; don't re-emit wait_request.
            RuntimeStatus::WaitFrame { .. } | RuntimeStatus::WaitClick { .. } => {
                return Ok(RuntimeTick {
                    executed: 0,
                    status: self.status.clone(),
                    wait_request: None,
                    frame_events: Vec::new(),
                });
            }
            RuntimeStatus::NotBooted => {
                self.status = RuntimeStatus::Running { pc: self.pc };
            }
            RuntimeStatus::Running { .. } => {}
        }

        // Inject pending script continuations before the script resumes.
        if let Some(point_id) = self.pending_jump_point.take() {
            match assets.point_table.resolve_target_pc(point_id) {
                Ok(Some(target_pc)) => {
                    self.pc = target_pc;
                    log::debug!(
                        "[trace-system] injected jump point[{point_id}] -> 0x{target_pc:08X}"
                    );
                }
                Ok(None) => {
                    log::warn!(
                        "[trace-system] jump point[{point_id}] resolved to no-op target, skipped"
                    );
                }
                Err(err) => {
                    log::warn!(
                        "[trace-system] jump point[{point_id}] resolve failed: {err}, skipped"
                    );
                }
            }
        }

        self.inject_pending_button_callbacks(&assets.point_table);

        self.frame_events.clear();

        let script = assets
            .script_image()
            .map_err(|source| RuntimeError::ScriptParse {
                source: source.to_string(),
            })?;
        self.run_thread_slice(
            &script,
            &assets.point_table,
            &assets.mem_dat.bytes,
            assets,
            resource_manager.as_deref_mut(),
            sprites.as_deref_mut(),
            task_system.as_deref_mut(),
            audio.as_deref_mut(),
            input,
            config.instructions_per_frame.max(1).min(64),
        )?;
        let mut executed = 0usize;
        let budget = config.instructions_per_frame.max(1);

        while executed < budget && !self.movie_active() {
            let result = self.step(
                &script,
                &assets.point_table,
                &assets.mem_dat.bytes,
                assets,
                resource_manager.as_deref_mut(),
                sprites.as_deref_mut(),
                task_system.as_deref_mut(),
                audio.as_deref_mut(),
                input,
            );
            if let Some(audio) = audio.as_deref_mut() { self.msprite_system.sync_audio(audio); }
            match result {
                Ok(StepResult::Continue) => {
                    executed += 1;
                    if self.modal_repark_due() && !self.pending_gosub_points.is_empty() {
                        // The modal handler returned but queued a follow-up
                        // gosub (btn_set_hit re-dispatch).  Inject it on top of
                        // the still-suspended wait instead of letting the script
                        // run past the parked line.
                        self.inject_pending_button_callbacks(&assets.point_table);
                        continue;
                    }
                    if let Some(request) = self.take_modal_wait_repark() {
                        // The modal menu gosub returned to the PC right after
                        // the suspended ADV click wait; re-park there instead
                        // of letting the script run on to the next line.
                        let events = std::mem::take(&mut self.frame_events);
                        return Ok(RuntimeTick {
                            executed,
                            status: self.status.clone(),
                            wait_request: Some(request),
                            frame_events: events,
                        });
                    }
                }
                Ok(StepResult::Blocked) => {
                    executed += 1;
                    let events = std::mem::take(&mut self.frame_events);
                    return Ok(RuntimeTick {
                        executed,
                        status: self.status.clone(),
                        wait_request: None,
                        frame_events: events,
                    });
                }
                Ok(StepResult::BlockedWithWait(req)) => {
                    executed += 1;
                    let is_adv_wait = std::mem::take(&mut self.adv_wait_checkpoint_pending);
                    if self.save_state.armed && is_adv_wait && matches!(req, WaitRequest::Click) {
                        let mut snapshot = self.capture_resumable_scene(sprites.as_deref());
                        snapshot.resume_wait_click = true;
                        // The parked line is fully revealed on restore, so its
                        // click mark should show even when the checkpoint was
                        // captured while the typewriter reveal was running.
                        snapshot.show_wait_mark = true;
                        let (body, name) = self.adv_text_parts_for_render(
                            assets,
                            resource_manager
                                .as_deref()
                                .map_or(Nls::ShiftJis, ResourceManager::nls),
                        );
                        snapshot.tips = self.capture_tips_snapshot_for_body(&body);
                        let title = if name.is_empty() {
                            body
                        } else if body.is_empty() {
                            name
                        } else {
                            format!("{name} {body}")
                        };
                        let (title, _) = parse_pal_text_directives(&title);
                        if !title.is_empty() {
                            snapshot.title_bytes = title.into_bytes();
                        }
                        self.save_state.resume_checkpoint = Some(snapshot);
                    }
                    let events = std::mem::take(&mut self.frame_events);
                    return Ok(RuntimeTick {
                        executed,
                        status: self.status.clone(),
                        wait_request: Some(req),
                        frame_events: events,
                    });
                }
                Err(err) => {
                    self.status = RuntimeStatus::Faulted {
                        pc: self.pc,
                        message: err.to_string(),
                    };
                    return Err(err);
                }
            }
        }

        self.status = RuntimeStatus::Running { pc: self.pc };
        let events = std::mem::take(&mut self.frame_events);
        Ok(RuntimeTick {
            executed,
            status: self.status.clone(),
            wait_request: None,
            frame_events: events,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_thread_slice(
        &mut self,
        script: &ScriptImage<'_>,
        point_table: &PointTable,
        mem_dat: &[u8],
        assets: &CoreAssets,
        mut resource_manager: Option<&mut ResourceManager>,
        mut sprites: Option<&mut SpriteSystem>,
        mut task_system: Option<&mut TaskSystem>,
        mut audio: Option<&mut AudioSystem>,
        input: Option<&PalInputState>,
        budget: usize,
    ) -> Result<(), RuntimeError> {
        if self.thread_state.ticking || self.thread_state.running == 0 {
            return Ok(());
        }
        let Some(thread_pc) = self.thread_state.target_pc else {
            return Ok(());
        };
        if let Some(mut wait) = self.thread_state.wait {
            if !wait.is_complete(self.pal_time_ms, input, sprites.as_deref()) {
                self.thread_state.wait = Some(wait);
                return Ok(());
            }
            log::debug!(
                "[trace-thread] thread point={} wait complete {:?}",
                self.thread_state.last_id,
                wait.request
            );
            self.thread_state.wait = None;
        }

        let main_pc = self.pc;
        let main_vars = std::mem::take(&mut self.vars);
        let main_stack = std::mem::take(&mut self.stack);
        let main_argument_stack = std::mem::take(&mut self.argument_stack);
        let main_argument_base = self.argument_base;
        let main_call_stack = std::mem::take(&mut self.call_stack);
        let main_status = self.status.clone();
        self.pc = thread_pc;
        self.vars = if self.thread_state.vars.is_empty() {
            vec![0; DEFAULT_VAR_COUNT]
        } else {
            std::mem::take(&mut self.thread_state.vars)
        };
        self.stack = std::mem::take(&mut self.thread_state.stack);
        self.argument_stack = std::mem::take(&mut self.thread_state.argument_stack);
        self.argument_base = self.thread_state.argument_base;
        self.call_stack = std::mem::take(&mut self.thread_state.call_stack);
        self.thread_state.ticking = true;

        let mut stop_thread = false;
        for _ in 0..budget.max(1) {
            match self.step(
                script,
                point_table,
                mem_dat,
                assets,
                resource_manager.as_deref_mut(),
                sprites.as_deref_mut(),
                task_system.as_deref_mut(),
                audio.as_deref_mut(),
                input,
            ) {
                Ok(StepResult::Continue) => {
                    if self.thread_state.running == 0 {
                        break;
                    }
                }
                Ok(StepResult::Blocked) => {
                    if matches!(self.status, RuntimeStatus::Halted { .. }) {
                        stop_thread = true;
                    }
                    break;
                }
                Ok(StepResult::BlockedWithWait(request)) => {
                    // Game work-process callbacks run with their own VM context.
                    // A wait emitted by that worker must pause only the worker;
                    // routing it through the main RuntimeStatus lets the worker
                    // resume too early and can leave transition quads/text waits
                    // over the normal script path.
                    self.thread_state.wait = Some(ThreadWaitState::new(request, self.pal_time_ms));
                    log::debug!(
                        "[trace-thread] thread point={} wait {:?} at pc=0x{:08X}",
                        self.thread_state.last_id,
                        request,
                        self.pc
                    );
                    break;
                }
                Err(RuntimeError::ReturnStackUnderflow { pc }) => {
                    log::debug!(
                        "[trace-thread] thread point={} returned at pc=0x{pc:08X}",
                        self.thread_state.last_id
                    );
                    stop_thread = true;
                    break;
                }
                Err(err) => {
                    self.thread_state.ticking = false;
                    self.thread_state.target_pc = Some(self.pc);
                    self.thread_state.vars = std::mem::take(&mut self.vars);
                    self.thread_state.stack = std::mem::take(&mut self.stack);
                    self.thread_state.argument_stack = std::mem::take(&mut self.argument_stack);
                    self.thread_state.argument_base = self.argument_base;
                    self.thread_state.call_stack = std::mem::take(&mut self.call_stack);
                    self.pc = main_pc;
                    self.vars = main_vars;
                    self.stack = main_stack;
                    self.argument_stack = main_argument_stack;
                    self.argument_base = main_argument_base;
                    self.call_stack = main_call_stack;
                    self.status = main_status;
                    return Err(err);
                }
            }
        }

        if stop_thread {
            self.thread_state.running = 0;
            self.thread_state.target_pc = None;
            self.thread_state.vars.clear();
            self.thread_state.stack.clear();
            self.thread_state.argument_stack.clear();
            self.thread_state.argument_base = 0;
            self.thread_state.call_stack.clear();
            self.thread_state.wait = None;
        } else {
            self.thread_state.target_pc = Some(self.pc);
            self.thread_state.vars = std::mem::take(&mut self.vars);
            self.thread_state.stack = std::mem::take(&mut self.stack);
            self.thread_state.argument_stack = std::mem::take(&mut self.argument_stack);
            self.thread_state.argument_base = self.argument_base;
            self.thread_state.call_stack = std::mem::take(&mut self.call_stack);
        }
        self.thread_state.ticking = false;
        self.pc = main_pc;
        self.vars = main_vars;
        self.stack = main_stack;
        self.argument_stack = main_argument_stack;
        self.argument_base = main_argument_base;
        self.call_stack = main_call_stack;
        self.status = main_status;
        Ok(())
    }

    fn step(
        &mut self,
        script: &ScriptImage<'_>,
        point_table: &PointTable,
        mem_dat: &[u8],
        assets: &CoreAssets,
        resource_manager: Option<&mut ResourceManager>,
        mut sprites: Option<&mut SpriteSystem>,
        task_system: Option<&mut TaskSystem>,
        audio: Option<&mut AudioSystem>,
        input: Option<&PalInputState>,
    ) -> Result<StepResult, RuntimeError> {
        if self.finish_history_replay(assets, resource_manager.as_deref().map_or(Nls::ShiftJis, ResourceManager::nls), sprites.as_deref()) {
            return Ok(StepResult::Blocked);
        }
        let filtered_input = input.map(|input| self.filter_script_input(input));
        let input = filtered_input.as_ref();
        let insn_pc = self.pc;
        let word = self.fetch_u32(script)?;
        let hi = ((word >> 16) & 0xFFFF) as u16;
        let opcode = (word & 0xFFFF) as u16;

        if hi != 1 {
            return Err(RuntimeError::InvalidInstructionWord { pc: insn_pc, word });
        }

        if self.vm_trace_enabled() {
            let name = primary_opcode(opcode)
                .map_or_else(|| format!("op_{opcode:04X}"), |meta| meta.name.to_owned());
            self.vm_trace(format_args!(
                "opcode pc=0x{insn_pc:08X} word=0x{word:08X} opcode={}({}) stack_len={} arg_len={} call_depth={}",
                opcode,
                name,
                self.stack.len(),
                self.argument_stack.len(),
                self.call_stack.len()
            ));
        }

        match opcode {
            // run_no_wait(): Game.exe sub_421F80 performs no VM pop and only
            // toggles/queues transition state.  It must not consume the
            // previous run(effect,arg1,arg2) arguments.
            1 => {
                let dst = self.fetch_operand(script)?;
                let src = self.fetch_operand(script)?;
                let value = self.eval_operand(src, mem_dat)?;
                self.vm_trace(format_args!("  op mov dst={dst} src={src} value={value}"));
                self.write_operand(dst, value)?;
            }
            2..=8 | 12..=19 | 26..=28 => {
                let dst = self.fetch_operand(script)?;
                let src = self.fetch_operand(script)?;
                let lhs = self.eval_operand(dst, mem_dat)?;
                let rhs = self.eval_operand(src, mem_dat)?;
                let value = self.eval_binary(opcode, lhs, rhs, insn_pc)?;
                self.vm_trace(format_args!(
                    "  op binary opcode={opcode} dst={dst} src={src} lhs={lhs} rhs={rhs} result={value}"
                ));
                self.write_operand(dst, value)?;
            }
            9 => {
                // jmp_operand: one operand whose VALUE is the point_id to jump to.
                let point_op = self.fetch_operand(script)?;
                let point_id = self.eval_operand(point_op, mem_dat)? as u32;
                self.vm_trace(format_args!(
                    "  op jmp point_operand={point_op} point={point_id}"
                ));
                self.jump_point(point_table, point_id)?;
            }
            10 => {
                let point_id = self.fetch_u32(script)?;
                let cond = self.fetch_operand(script)?;
                let value = self.eval_operand(cond, mem_dat)?;
                self.vm_trace(format_args!(
                    "  op jf point={point_id} cond={cond} value={value} taken={}",
                    value == 0
                ));
                if value == 0 {
                    self.jump_point(point_table, point_id)?;
                }
            }
            11 => {
                // Newer sub_458C10 resolves the operand through sub_458740,
                // including variable/argument references used by SE callbacks.
                let point_id = if assets.registered_softpal {
                    let point = self.fetch_operand(script)?;
                    self.eval_operand(point, mem_dat)? as u32
                } else {
                    self.fetch_u32(script)?
                };
                self.vm_trace(format_args!(
                    "  op gosub point={point_id} return_pc=0x{:08X}",
                    self.pc
                ));
                self.call_stack.push(self.pc);
                self.jump_point(point_table, point_id)?;
            }
            20 => {
                let raw = self.fetch_u32(script)?;
                let slot = (raw & 0xFFFF) as usize;
                let value = self.read_var(slot)?;
                self.vm_trace(format_args!(
                    "  op not slot={slot} old={value} new={}",
                    if value == 0 { 1 } else { 0 }
                ));
                self.write_var(slot, if value == 0 { 1 } else { 0 })?;
            }
            21 => {
                self.vm_trace(format_args!("  op halt next_pc=0x{:08X}", self.pc));
                self.status = RuntimeStatus::Halted { pc: self.pc };
                return Ok(StepResult::Blocked);
            }
            22 => {
                self.vm_trace(format_args!("  op nop"));
            }
            op if op == 23 || ((assets.registered_softpal || (op != 252 && op != 253))
                && primary_extcall_for_profile(op, assets.registered_softpal).is_some()) => {
                let (raw, dst_slot_raw) = if op == 23 {
                    (self.fetch_u32(script)?, self.fetch_u32(script)?)
                } else {
                    let (category, index) = primary_extcall_for_profile(op, assets.registered_softpal).unwrap();
                    (((category as u32) << 16) | index as u32, 0)
                };
                self.extcall_instruction_size = if op == 23 { 12 } else { 4 };
                let category = ((raw >> 16) & 0xFFFF) as u16;
                let index = (raw & 0xFFFF) as u16;
                let registered_sig = assets.registered_softpal.then(|| lookup_registered_sig(category, index)).flatten();
                let name = registered_sig.map(|sig| sig.name)
                    .or_else(|| ext_opcode(category, index).and_then(|meta| meta.name));
                self.vm_trace(format_args!(
                    "  op extcall raw=0x{raw:08X} ext_{category:04X}_{index:04X}.{} dst_raw=0x{dst_slot_raw:08X} stack_before={:?}",
                    name.unwrap_or("?"),
                    self.stack
                ));
                let writes_destination = op == 23 && !(category == 11 && index == 3) && registered_sig.and_then(|sig| sig.writes_destination) != Some(false);
                // Store for extcall handlers that need it
                self.extcall_dst_raw = dst_slot_raw;
                if category == 2 && index == 2 {
                    // Native AdvCommandText mirrors the current text command
                    // position into the save image header on every line.
                    self.text_state.last_text_pc = insn_pc;
                    // koikake's ADV body is a linear run of text commands; each
                    // one submits a line and parks in a click wait without an
                    // intervening wait_click. Mark the wait so the blocked path
                    // refreshes the resumable checkpoint per line, otherwise
                    // saves keep the stale savepoint pc and loads replay the
                    // section from its start.
                    self.adv_wait_checkpoint_pending = true;
                }
                // Check if there is a Rust handler; if not, null-handler semantics: write 0 to
                // dst and continue (matches original behavior for null-category dispatches).
                let result = self.dispatch_extcall(
                    category,
                    index,
                    mem_dat,
                    assets,
                    point_table,
                    resource_manager,
                    sprites,
                    task_system,
                    audio,
                    input,
                );
                self.vm_trace(format_args!(
                    "  extcall result ext_{category:04X}_{index:04X}.{} => {:?} stack_after={:?}",
                    name.unwrap_or("?"),
                    result,
                    self.stack
                ));
                match result {
                    ExtCallOutcome::Continue => {}
                    ExtCallOutcome::Yield { value } => {
                        if writes_destination {
                            if let Some(value) = value { let _ = self.write_extcall_dst(dst_slot_raw, value); }
                        }
                        return Ok(StepResult::Blocked);
                    }
                    ExtCallOutcome::Value(v) => {
                        self.vm_trace(format_args!(
                            "  extcall write dst_slot[{dst_slot_raw}] value={v}"
                        ));
                        if writes_destination { let _ = self.write_extcall_dst(dst_slot_raw, v); }
                    }
                    ExtCallOutcome::Wait { value, request } => {
                        self.vm_trace(format_args!(
                            "  extcall wait dst_slot[{dst_slot_raw}] value={value} request={request:?}"
                        ));
                        if writes_destination { let _ = self.write_extcall_dst(dst_slot_raw, value); }
                        self.status = match request {
                            WaitRequest::Click
                            | WaitRequest::ClickOrTime(_)
                            | WaitRequest::AutoClickOrTime(_) => {
                                RuntimeStatus::WaitClick { pc: self.pc }
                            }
                            WaitRequest::Frame(_)
                            | WaitRequest::Time(_)
                            | WaitRequest::TextReveal(_)
                            | WaitRequest::SpriteFx(_) => {
                                RuntimeStatus::WaitFrame { pc: self.pc }
                            }
                        };
                        if self.frame_events.len() < MAX_FRAME_EVENTS {
                            self.frame_events.push(FrameEvent::WaitEmitted {
                                pc: insn_pc,
                                kind: request,
                            });
                        }
                        return Ok(StepResult::BlockedWithWait(request));
                    }
                    ExtCallOutcome::Skip => {
                        let name_str = name.unwrap_or("?");
                        let cleanup_count = registered_sig.map(|sig| sig.pop_count).or_else(|| lookup_sig(category, index)
                            .map(|sig| sig.pop_count)
                            .or_else(|| observed_pop_count(category, index)));
                        if let Some(count) = cleanup_count {
                            let dropped = self.pop_ext_args(count);
                            self.vm_trace(format_args!(
                                "  extcall skip cleanup argc={count} args={dropped:?}"
                            ));
                        }
                        log::warn!(
                            "unimplemented extcall ext_{:04X}_{:04X}.{} pc=0x{:08X} -> 0",
                            category,
                            index,
                            name_str,
                            insn_pc
                        );
                        if self.frame_events.len() < MAX_FRAME_EVENTS {
                            self.frame_events.push(FrameEvent::ExtCallSkipped {
                                pc: insn_pc,
                                category,
                                index,
                                name: name.map(str::to_owned),
                            });
                        }
                        self.vm_trace(format_args!(
                            "  extcall skip write dst_slot[{dst_slot_raw}] value=0"
                        ));
                        if writes_destination { let _ = self.write_extcall_dst(dst_slot_raw, 0); }
                    }
                    ExtCallOutcome::Block => {
                        let name_owned = name.map(str::to_owned);
                        if self.frame_events.len() < MAX_FRAME_EVENTS {
                            self.frame_events.push(FrameEvent::UnsupportedExt {
                                pc: insn_pc,
                                category,
                                index,
                                name: name_owned.clone(),
                            });
                        }
                        self.status = RuntimeStatus::UnsupportedExtCall {
                            pc: insn_pc,
                            category,
                            index,
                            name: name_owned,
                            dst_slot: dst_slot_raw,
                        };
                        return Ok(StepResult::Blocked);
                    }
                }
            }
            25 => {
                // reset_adv: argc 0. Clears the ADV text window so the next line
                // starts without the previous wait mark or reveal.
                self.reset_adv();
                self.vm_trace(format_args!("  op reset_adv"));
            }
            24 => {
                let Some(return_pc) = self.call_stack.pop() else {
                    return Err(RuntimeError::ReturnStackUnderflow { pc: insn_pc });
                };
                self.vm_trace(format_args!("  op ret target=0x{return_pc:08X}"));
                self.pc = return_pc;
            }
            29 => {
                let raw = self.fetch_u32(script)?;
                let slot = (raw & 0xFFFF) as usize;
                let value = self.read_var(slot)?;
                self.vm_trace(format_args!(
                    "  op neg_slot slot={slot} old={value} new={}",
                    value.wrapping_neg()
                ));
                self.write_var(slot, value.wrapping_neg())?;
            }
            30 => {
                let dst = self.fetch_operand(script)?;
                let value = self
                    .stack
                    .pop()
                    .ok_or(RuntimeError::StackUnderflow { pc: insn_pc })?;
                self.vm_trace(format_args!(
                    "  op pop dst={dst} value={value} stack_len={}",
                    self.stack.len()
                ));
                self.write_operand(dst, value)?;
            }
            31 => {
                let src = self.fetch_operand(script)?;
                let value = self.eval_operand(src, mem_dat)?;
                self.vm_trace(format_args!("  op push src={src} value={value}"));
                self.push_stack(value, insn_pc)?;
            }
            32 => {
                let count_operand = self.fetch_operand(script)?;
                let count = self.eval_operand(count_operand, mem_dat)?;
                self.vm_trace(format_args!(
                    "  op pack_args count_operand={count_operand} count={count}"
                ));
                self.pack_args(count, insn_pc)?;
            }
            33 => {
                let count_operand = self.fetch_operand(script)?;
                let count = self.eval_operand(count_operand, mem_dat)?;
                self.vm_trace(format_args!(
                    "  op drop_args count_operand={count_operand} count={count}"
                ));
                self.drop_args(count, insn_pc)?;
            }
            252 | 253 if !assets.registered_softpal => {
                // Native primary entries share the category-7 handlers, including
                // operand consumption and the ADV -1 sentinel. Direct opcodes
                // resume at the next 4-byte instruction, not an extcall's +12.
                let arity = if opcode == 252 { 2 } else { 1 };
                if self.stack.len() < arity {
                    return Err(RuntimeError::StackUnderflow { pc: insn_pc });
                }
                if let ExtCallOutcome::Wait { request, .. } = self.dispatch_wait_ext(opcode - 252) {
                    self.status = match request {
                        WaitRequest::Click | WaitRequest::ClickOrTime(_) | WaitRequest::AutoClickOrTime(_) => RuntimeStatus::WaitClick { pc: self.pc },
                        _ => RuntimeStatus::WaitFrame { pc: self.pc },
                    };
                    if self.frame_events.len() < MAX_FRAME_EVENTS {
                        self.frame_events.push(FrameEvent::WaitEmitted { pc: insn_pc, kind: request });
                    }
                    return Ok(StepResult::BlockedWithWait(request));
                }
            }
            _ => {
                let name = primary_opcode(opcode).map(|meta| meta.name.to_owned());
                self.vm_trace(format_args!(
                    "  op unsupported opcode={opcode} name={}",
                    name.as_deref().unwrap_or("?")
                ));
                if self.frame_events.len() < MAX_FRAME_EVENTS {
                    self.frame_events.push(FrameEvent::UnsupportedCmd {
                        pc: insn_pc,
                        opcode,
                        name: name.clone(),
                    });
                }
                self.status = RuntimeStatus::UnsupportedCommand {
                    pc: insn_pc,
                    opcode,
                    name,
                };
                return Ok(StepResult::Blocked);
            }
        }

        self.status = RuntimeStatus::Running { pc: self.pc };
        Ok(StepResult::Continue)
    }

    fn fetch_u32(&mut self, script: &ScriptImage<'_>) -> Result<u32, RuntimeError> {
        let pc = self.pc as usize;
        let fetch_pc = self.pc;
        let end = pc
            .checked_add(4)
            .ok_or(RuntimeError::ArithmeticOverflow { pc: self.pc })?;
        if end > script.len() {
            return Err(RuntimeError::ReadOutOfBounds {
                pc: self.pc,
                len: script.len(),
            });
        }
        let bytes = script.bytes();
        let value = u32::from_le_bytes([bytes[pc], bytes[pc + 1], bytes[pc + 2], bytes[pc + 3]]);
        self.pc = self
            .pc
            .checked_add(4)
            .ok_or(RuntimeError::ArithmeticOverflow { pc: self.pc })?;
        self.vm_trace(format_args!(
            "  fetch_u32 pc=0x{fetch_pc:08X} value=0x{value:08X} ({})",
            value as i32
        ));
        Ok(value)
    }

    fn fetch_operand(&mut self, script: &ScriptImage<'_>) -> Result<Operand, RuntimeError> {
        let raw = self.fetch_u32(script)?;
        let operand = Operand::decode(raw);
        self.vm_trace(format_args!(
            "  fetch_operand raw=0x{raw:08X} decoded={operand}"
        ));
        Ok(operand)
    }

    fn jump_point(&mut self, point_table: &PointTable, point_id: u32) -> Result<(), RuntimeError> {
        match point_table.resolve_target_pc(point_id) {
            Ok(Some(target)) => {
                self.vm_trace(format_args!(
                    "  jump_point point={point_id} target=0x{target:08X}"
                ));
                self.pc = target;
                Ok(())
            }
            Ok(None) => {
                self.vm_trace(format_args!("  jump_point point={point_id} target=<none>"));
                Ok(())
            }
            Err(source) => Err(RuntimeError::PointResolve {
                point_id,
                source: source.to_string(),
            }),
        }
    }

    fn eval_binary(&self, opcode: u16, lhs: i32, rhs: i32, pc: u32) -> Result<i32, RuntimeError> {
        let value = match opcode {
            2 => lhs.wrapping_add(rhs),
            3 => lhs.wrapping_sub(rhs),
            4 => lhs.wrapping_mul(rhs),
            5 => {
                if rhs == 0 {
                    return Err(RuntimeError::DivideByZero { pc });
                }
                lhs.wrapping_div(rhs)
            }
            6 => lhs & rhs,
            7 => lhs | rhs,
            8 => lhs ^ rhs,
            12 => bool_to_i32(lhs == rhs),
            13 => bool_to_i32(lhs != rhs),
            14 => bool_to_i32(lhs <= rhs),
            15 => bool_to_i32(lhs >= rhs),
            16 => bool_to_i32(lhs < rhs),
            17 => bool_to_i32(lhs > rhs),
            18 => bool_to_i32(lhs != 0 || rhs != 0),
            19 => bool_to_i32(lhs != 0 && rhs != 0),
            26 => {
                if rhs == 0 {
                    return Err(RuntimeError::DivideByZero { pc });
                }
                lhs.wrapping_rem(rhs)
            }
            27 => lhs.wrapping_shl((rhs & 31) as u32),
            28 => ((lhs as u32).wrapping_shr((rhs & 31) as u32)) as i32,
            _ => return Err(RuntimeError::UnsupportedInternalOpcode { pc, opcode }),
        };
        Ok(value)
    }

    fn eval_operand(&self, operand: Operand, _mem_dat: &[u8]) -> Result<i32, RuntimeError> {
        if self.vm_trace_enabled() {
            self.vm_trace(format_args!("  eval_operand {operand}"));
        }
        let result = match operand.kind {
            OperandKind::Immediate => Ok(operand.raw as i32),
            // kind >= 0xA: treat as a raw literal (same encoding as Immediate).
            OperandKind::LiteralSlot => Ok(operand.raw as i32),
            OperandKind::VariableSlot => self.read_var(operand.lo as usize),
            OperandKind::StackSlot => self.read_stack_slot(operand.lo as usize),
            OperandKind::ArgumentStack => self.read_argument_stack(operand.lo as usize),
            // kind 0x9: argument_base scalar field.
            OperandKind::ArgumentBase => Ok(self.argument_base),
            // kind 0x1: user_mem[vars[lo]]
            OperandKind::UserMemoryViaVar => {
                let signed_idx = self.read_var(operand.lo as usize)?;
                let Some(idx) =
                    checked_script_mem_index("user_mem", signed_idx, self.user_mem.len())
                else {
                    return Ok(0);
                };
                Ok(self.user_mem[idx])
            }
            // kind 0x2: system_mem[vars[lo]]
            OperandKind::SystemMemoryViaVar => {
                let signed_idx = self.read_var(operand.lo as usize)?;
                let Some(idx) =
                    checked_script_mem_index("system_mem", signed_idx, self.system_mem.len())
                else {
                    return Ok(0);
                };
                Ok(self.system_mem[idx])
            }
            // kind 0x5: temp_mem[(bank != 0 ? bank + argument_base : 0) + vars[lo]]
            // The index may alias other areas in the original flat "this" object.
            // Gracefully return 0 on out-of-range (the full aliasing semantics
            // require a unified flat memory model not yet implemented).
            OperandKind::TempMemoryViaVar => {
                let bank = operand.bank as i32;
                let base = if bank != 0 {
                    bank + self.argument_base
                } else {
                    0
                };
                let var_val = self.read_var(operand.lo as usize)?;
                let signed_idx = base.wrapping_add(var_val);
                if signed_idx < 0 || signed_idx as usize >= self.temp_mem.len() {
                    log::debug!(
                        "TempMemoryViaVar read out of range: idx={} bank={} var={}",
                        signed_idx,
                        bank,
                        var_val
                    );
                    return Ok(0);
                }
                Ok(self.temp_mem[signed_idx as usize])
            }
            // kind 0x6: MemDatDirect — read from writable Mem.dat shadow.
            OperandKind::MemDatDirect => self.read_mem_dat_i32(operand),
            // kind 0x7: MemDatIndirect — Game.exe 0x42116b loads mem[bank+4]
            // and resolves that word as a direct operand, keeping this lo.
            OperandKind::MemDatIndirect => self.read_mem_dat_indirect(operand),
        };
        if self.vm_trace_enabled() {
            if let Ok(v) = result {
                self.vm_trace(format_args!("  eval_operand_result {operand} => {v}"));
            }
        }
        result
    }

    fn write_operand(&mut self, operand: Operand, value: i32) -> Result<(), RuntimeError> {
        if self.vm_trace_enabled() {
            self.vm_trace(format_args!("  write_operand {operand} = {value}"));
        }
        match operand.kind {
            OperandKind::VariableSlot => self.write_var(operand.lo as usize, value),
            OperandKind::StackSlot => self.write_stack_slot(operand.lo as usize, value),
            // kind 0x9: argument_base scalar.
            OperandKind::ArgumentBase => {
                self.argument_base = value;
                Ok(())
            }
            // kind 0x1: user_mem[vars[lo]] = value
            OperandKind::UserMemoryViaVar => {
                let signed_idx = self.read_var(operand.lo as usize)?;
                let Some(idx) =
                    checked_script_mem_index("user_mem", signed_idx, self.user_mem.len())
                else {
                    return Ok(());
                };
                self.user_mem[idx] = value;
                Ok(())
            }
            // kind 0x2: system_mem[vars[lo]] = value
            OperandKind::SystemMemoryViaVar => {
                let signed_idx = self.read_var(operand.lo as usize)?;
                let Some(idx) =
                    checked_script_mem_index("system_mem", signed_idx, self.system_mem.len())
                else {
                    return Ok(());
                };
                self.system_mem[idx] = value;
                Ok(())
            }
            // kind 0x5: temp_mem[(bank != 0 ? bank + argument_base : 0) + vars[lo]] = value
            OperandKind::TempMemoryViaVar => {
                let bank = operand.bank as i32;
                let base = if bank != 0 {
                    bank + self.argument_base
                } else {
                    0
                };
                let var_val = self.read_var(operand.lo as usize)?;
                let signed_idx = base.wrapping_add(var_val);
                if signed_idx < 0 {
                    log::debug!(
                        "[trace-vm] temp_mem signed index {signed_idx} out of range; ignoring write"
                    );
                    return Ok(());
                }
                let idx = signed_idx as usize;
                // The native work bank is one flat region; large argument_base
                // frames legitimately address past the default size.  Grow like
                // write_temp_mem_absolute instead of dropping the write.
                if idx >= self.temp_mem.len() {
                    self.temp_mem.resize(idx + 1, 0);
                }
                self.temp_mem[idx] = value;
                Ok(())
            }
            // kind 0x6: MemDatDirect — write to the writable shadow copy.
            OperandKind::MemDatDirect => {
                let word_index = self.mem_dat_word_index(operand)?;
                self.write_mem_dat_word(word_index, value);
                Ok(())
            }
            OperandKind::MemDatIndirect => {
                let word_index = self.mem_dat_indirect_word_index(operand)?;
                self.write_mem_dat_word(word_index, value);
                Ok(())
            }
            // kind 0x8: ArgumentStack — write to arg_area[arg_top - lo].
            OperandKind::ArgumentStack => {
                let lo = operand.lo as usize;
                let depth = self.argument_stack.len();
                if lo == 0 || lo > depth {
                    return Err(RuntimeError::ArgumentSlotOutOfRange { slot: lo, depth });
                }
                self.argument_stack[depth - lo] = value;
                Ok(())
            }
            // Immediate/LiteralSlot — write to the temp result slot (harmless, result is
            // discarded). Matches original where the pointer to a temp scratch slot is returned.
            OperandKind::Immediate | OperandKind::LiteralSlot => Ok(()),
            _ => Err(RuntimeError::UnsupportedOperandWrite {
                raw: operand.raw,
                kind: format!("{:?}", operand.kind),
            }),
        }
    }

    fn read_var(&self, slot: usize) -> Result<i32, RuntimeError> {
        self.vars
            .get(slot)
            .copied()
            .ok_or(RuntimeError::VariableOutOfRange { slot })
    }

    fn write_var(&mut self, slot: usize, value: i32) -> Result<(), RuntimeError> {
        let dst = self
            .vars
            .get_mut(slot)
            .ok_or(RuntimeError::VariableOutOfRange { slot })?;
        *dst = value;
        Ok(())
    }

    fn write_extcall_dst(&mut self, dst_slot_raw: u32, value: i32) -> Result<(), RuntimeError> {
        self.write_var(dst_slot_raw as usize, value)
    }

    fn read_stack_slot(&self, slot: usize) -> Result<i32, RuntimeError> {
        self.stack
            .get(slot)
            .copied()
            .ok_or(RuntimeError::StackSlotOutOfRange {
                slot,
                depth: self.stack.len(),
            })
    }

    fn write_stack_slot(&mut self, slot: usize, value: i32) -> Result<(), RuntimeError> {
        let depth = self.stack.len();
        let dst = self
            .stack
            .get_mut(slot)
            .ok_or(RuntimeError::StackSlotOutOfRange { slot, depth })?;
        *dst = value;
        Ok(())
    }

    /// Read from argument_stack using 1-based `arg_stack[-N]` indexing.
    ///
    /// Game.exe `VmOpc_PackArgs` (0x42CAA0) pops the VM value stack top-first
    /// into the argument array at increasing indices.  Therefore `arg[-1]`
    /// addresses the newest pushed source argument for the current call group,
    /// i.e. the last appended argument-array cell.
    fn read_argument_stack(&self, lo: usize) -> Result<i32, RuntimeError> {
        let depth = self.argument_stack.len();
        let index = depth
            .checked_sub(lo)
            .ok_or(RuntimeError::ArgumentSlotOutOfRange { slot: lo, depth })?;
        Ok(self.argument_stack[index])
    }

    fn read_mem_dat_i32(&self, operand: Operand) -> Result<i32, RuntimeError> {
        let word_index = self.mem_dat_word_index(operand)?;
        Ok(self.mem_dat_words.get(word_index).copied().unwrap_or(0))
    }

    fn read_mem_dat_indirect(&self, operand: Operand) -> Result<i32, RuntimeError> {
        let word_index = self.mem_dat_indirect_word_index(operand)?;
        Ok(self.mem_dat_words.get(word_index).copied().unwrap_or(0))
    }

    fn write_mem_dat_word(&mut self, word_index: usize, value: i32) {
        if word_index >= self.mem_dat_words.len() {
            self.mem_dat_words.resize(word_index + 1, 0);
        }
        let old = self.mem_dat_words[word_index];
        if old != value && debug_memdat_write_enabled() {
            log::debug!(
                "[trace-memdat] write word[{}] (memdat[{}]) {:08X} -> {:08X} pc=0x{:08X}",
                word_index,
                word_index.wrapping_sub(4),
                old as u32,
                value as u32,
                self.pc
            );
        }
        self.mem_dat_words[word_index] = value;
    }

    fn mem_dat_word_index(&self, operand: Operand) -> Result<usize, RuntimeError> {
        self.mem_dat_word_index_parts(operand.bank as i32, operand.lo)
    }

    /// Game.exe `0x42116b` handles tag `0x70000000` by loading `mem[bank + 4]`
    /// and falling into the direct resolver. The loaded word supplies the
    /// direct bank; the original operand keeps its variable slot.
    fn mem_dat_indirect_word_index(&self, operand: Operand) -> Result<usize, RuntimeError> {
        let pointer_index = self.mem_dat_header_index(operand.bank as i32)?;
        let loaded = self.mem_dat_words.get(pointer_index).copied().unwrap_or(0) as u32;
        let inner_bank = ((loaded >> 16) & 0x0FFF) as i32;
        self.mem_dat_word_index_parts(inner_bank, operand.lo)
    }

    fn mem_dat_header_index(&self, bank: i32) -> Result<usize, RuntimeError> {
        let signed = bank.wrapping_add(4);
        if signed < 0 {
            return Err(RuntimeError::MemDatOutOfRange {
                offset: 0,
                len: self.mem_dat_words.len() * 4,
            });
        }
        Ok(signed as usize)
    }

    fn mem_dat_word_index_parts(&self, bank: i32, var_slot: u16) -> Result<usize, RuntimeError> {
        // Original formula: mem_dat_ptr + 4*(bank + vars[lo]) + 16.
        // The "+16" skips the 16-byte header present in Mem.dat. Scripts also
        // use this area as mutable work storage, so writes can extend the shadow.
        let var_val = self.read_var(var_slot as usize)?;
        let signed_word_index = bank.wrapping_add(var_val).wrapping_add(4);
        if signed_word_index < 0 {
            return Err(RuntimeError::MemDatOutOfRange {
                offset: 0,
                len: self.mem_dat_words.len() * 4,
            });
        }
        Ok(signed_word_index as usize)
    }

    fn push_stack(&mut self, value: i32, pc: u32) -> Result<(), RuntimeError> {
        if self.stack.len() >= DEFAULT_STACK_LIMIT {
            return Err(RuntimeError::StackOverflow {
                pc,
                limit: DEFAULT_STACK_LIMIT,
            });
        }
        self.stack.push(value);
        self.vm_trace(format_args!(
            "  stack_push value={value} stack_len={}",
            self.stack.len()
        ));
        Ok(())
    }

    fn pack_args(&mut self, count: i32, pc: u32) -> Result<(), RuntimeError> {
        if count < 0 {
            return Err(RuntimeError::NegativeCount { pc, count });
        }
        let count = count as usize;
        if count > self.stack.len() {
            return Err(RuntimeError::StackUnderflow { pc });
        }
        // Game.exe `VmOpc_PackArgs` (sub_42CAA0) repeatedly pops the VM value
        // stack top-first into the argument array at increasing indices, then
        // pushes the new argument top back onto the value stack.  With caller
        // pushes [arg1, arg2, arg3], the native argument array receives
        // [arg3, arg2, arg1], so arg_stack[-1] resolves to the first source
        // argument after the `top - lo` addressing in sub_42C910.
        let mut packed = Vec::with_capacity(count);
        for _ in 0..count {
            let Some(value) = self.stack.pop() else {
                return Err(RuntimeError::StackUnderflow { pc });
            };
            packed.push(value);
        }
        let new_arg_top = self.argument_stack.len() + packed.len();
        self.vm_trace(format_args!(
            "  pack_args count={count} native_values={packed:?} stack_len={} arg_len_before={}",
            self.stack.len(),
            self.argument_stack.len()
        ));
        self.argument_stack.append(&mut packed);
        self.push_stack(new_arg_top as i32, pc)?;
        self.vm_trace(format_args!(
            "  pack_args_done arg_len={} pushed_arg_top={new_arg_top}",
            self.argument_stack.len(),
        ));
        Ok(())
    }

    fn drop_args(&mut self, count: i32, pc: u32) -> Result<(), RuntimeError> {
        if count < 0 {
            return Err(RuntimeError::NegativeCount { pc, count });
        }
        let count = count as usize;
        if count > self.argument_stack.len() {
            return Err(RuntimeError::ArgumentStackUnderflow { pc });
        }
        let new_len = self.argument_stack.len() - count;
        let dropped = self.argument_stack[new_len..].to_vec();
        self.argument_stack.truncate(new_len);
        // Game.exe `VmOpc_DropArgs` (sub_42CA50) also pops one value from the
        // ordinary VM stack after subtracting the requested argument count. It
        // consumes the argument-top marker written by `VmOpc_PackArgs`; without
        // this, nested helper calls leak frame markers into extcall arguments.
        let stack_marker = self
            .stack
            .pop()
            .ok_or(RuntimeError::StackUnderflow { pc })?;
        self.vm_trace(format_args!(
            "  drop_args count={count} dropped={dropped:?} arg_len={new_len} popped_arg_top={stack_marker}"
        ));
        Ok(())
    }

    /// Read extcall argument by 1-based index.
    /// lo=1 is the most recently packed item (top of the argument group).
    pub fn extcall_arg(&self, lo: usize) -> i32 {
        let depth = self.argument_stack.len();
        if lo == 0 || lo > depth {
            return 0;
        }
        self.argument_stack[depth - lo]
    }

    /// Dispatch an extcall by category and index.
    /// Returns Value(n) if handled, Skip to continue with 0, or Block to pause execution.
    fn dispatch_extcall(
        &mut self,
        category: u16,
        index: u16,
        _mem_dat: &[u8],
        assets: &CoreAssets,
        point_table: &PointTable,
        resource_manager: Option<&mut ResourceManager>,
        mut sprites: Option<&mut SpriteSystem>,
        task_system: Option<&mut TaskSystem>,
        audio: Option<&mut AudioSystem>,
        input: Option<&PalInputState>,
    ) -> ExtCallOutcome {
        self.input_registered_profile = assets.registered_softpal;
        self.input_release_shortcuts = assets.registered_softpal || assets.text_dat_multilingual;
        if let Some(audio) = audio.as_deref() { self.msprite_system.set_audio_levels(audio); }
        self.effect_system.set_registered_profile(assets.registered_softpal, assets.extended_softpal);
        // Script metadata distinguishes the three audited releases; it is not
        // a universal PAL version detector for unknown games.
        self.set_font_profile(assets.registered_softpal, assets.text_dat_multilingual);
        let nls = resource_manager.as_ref().map(|manager| manager.nls()).unwrap_or(Nls::ShiftJis);
        if let Some(sprites) = sprites.as_deref_mut() {
            sprites.set_registered_profile(assets.registered_softpal);
        }
        if assets.registered_softpal {
            match (category, index) {
                (11, 8..=13) => return self.dispatch_movie_control(index, audio),
                (2, 47) => {
                    let args = self.pop_ext_args(3);
                    if args.len() != 3 { return ExtCallOutcome::Block; }
                    self.registered_text_animation = Some(TextAnimationMode { kind: args[0], fade: args[1] != 0, space_skip: args[2] != 0 });
                    self.text_state.dirty = true;
                    return ExtCallOutcome::Continue;
                }
                (9, 63) => return ExtCallOutcome::Value(i32::from(self.host_active)),
                (18, 127) => return ExtCallOutcome::Value(self.last_window_mode_change_ms as i32),
                (18, 96) => return ExtCallOutcome::Value(if self.host_gpu_renderer { 8 } else { 0 }),
                (18, 88) => return ExtCallOutcome::Value(host_queries::network_status().unwrap_or(0) as i32),
                (18, 89) => return ExtCallOutcome::Value(i32::from(self.host_touch_available)),
                (5, 8..=12) => return self.dispatch_registered_se_info(index, assets, nls),
                (5, 0..=7 | 13..=18) => return self.dispatch_registered_se(index, assets, nls, resource_manager, audio, input),
                (18, 97 | 98) => {
                    let args = self.pop_ext_args(2);
                    if args.len() != 2 { return ExtCallOutcome::Block; }
                    let value = if index == 97 {
                        host_queries::memory_status(args[0])
                    } else {
                        resource_manager.as_ref().and_then(|manager| host_queries::disk_status(args[0], manager.root()))
                    }.unwrap_or(0) >> 20;
                    // Native shifts the complete 64-bit value before writing
                    // its high word, even for memory-load selector zero.
                    if args[1] >= 0 {
                        if let Some(slot) = self.temp_mem.get_mut(args[1] as usize) {
                            *slot = (value >> 32) as i32;
                        }
                    }
                    return ExtCallOutcome::Value(value as i32);
                }
                (18, 99) => {
                    let args = self.pop_ext_args(1);
                    let Some(&selector) = args.first() else { return ExtCallOutcome::Block; };
                    return ExtCallOutcome::Value(host_queries::os_version(selector).unwrap_or(0) as i32);
                }
                (18, 90) => return self.dispatch_registered_debug(assets, nls),
                (3, 82..=84) => return self.dispatch_target_render(index, sprites),
                (10, 37..=39) => return self.dispatch_history_navigation(index, resource_manager.as_deref(), sprites),
                (18, 54) => {
                    self.capture_save_checkpoint(assets, nls, sprites.as_deref());
                    self.capture_history_checkpoint(sprites.as_deref());
                    return ExtCallOutcome::Continue;
                }
                (9, 57) => {
                    let args = self.pop_ext_args(1);
                    let Some(&target) = args.first() else { return ExtCallOutcome::Block; };
                    self.history_navigation.target = Some(target as u32);
                    return ExtCallOutcome::Continue;
                }
                (3, 59) => {
                    if self.pop_ext_args(2).len() != 2 { return ExtCallOutcome::Block; }
                    return ExtCallOutcome::Continue;
                }
                (18, 91) => {
                    let args = self.pop_ext_args(1);
                    let Some(&enabled) = args.first() else { return ExtCallOutcome::Block; };
                    self.system_state.set_touch_enabled(enabled != 0);
                    return ExtCallOutcome::Value(1);
                }
                _ => {}
            }
        }
        let nls = resource_manager
            .as_ref()
            .map(|manager| manager.nls())
            .unwrap_or(Nls::ShiftJis);

        if category == 9 && index == 44 {
            let args = self.pop_ext_args(1);
            let Some(&profile) = args.first() else { return ExtCallOutcome::Block; };
            self.system_state.clear_system_paths();
            let Some(manager) = resource_manager else { return ExtCallOutcome::Value(0); };
            let result = manager.select_archive_profile(profile as usize);
            if let Err(err) = &result {
                log::warn!("set_system_path({profile}) failed: {err}");
            }
            return ExtCallOutcome::Value(i32::from(result.is_ok()));
        }

        if category == 16 && index == 5 {
            let args = self.pop_ext_args(4);
            if args.len() != 4 { return ExtCallOutcome::Block; }
            if let Some(sprites) = sprites {
                if sprites.start_global_raster(game_sprite_priority(args[0], self.sprite_priority_cursor),
                    crate::fx::RasterSource { period_ms: args[3] as u32,
                        undulate_x: (args[1] as u32) as f32 * 0.01,
                        undulate_y: (args[2] as u32) as f32 * 0.01 }, args[3] as u32) {
                    self.pending_global_fx_release = false;
                }
            }
            return ExtCallOutcome::Wait { value: 1, request: WaitRequest::Frame(1) };
        }
        if category == 16 && index == 0 {
            let count = self.stack.len();
            if count >= 2 && self.stack[count - 1] & 0x10 != 0 {
                let duration = if self.dispatch_skip_state() != 0 { 0 } else { self.stack[count - 2] as u32 };
                if let Some(sprites) = sprites {
                    if sprites.global_raster_state() == 1 {
                        sprites.stop_global_raster(duration);
                        self.pending_global_fx_release = true;
                    }
                }
            }
            return self.dispatch_window_effect_stub(index);
        }

        if category == 19 && index <= 3 {
            return self.dispatch_fx_ext(index + 4, sprites);
        }
        if category == 22 && matches!(index, 4..=7) {
            return self.dispatch_fx_ext(index, sprites);
        }

        if category == 9 && matches!(index, 15 | 16 | 20) {
            return self.dispatch_font_resource(index, assets, nls, resource_manager);
        }

        if category == 9 && matches!(index, 18 | 32..=35 | 54) {
            return self.dispatch_input_control(index, input);
        }
        if category == 6 {
            return self.dispatch_select_ext(index, assets, nls, resource_manager, sprites, input);
        }

        if category == 13 {
            return self.dispatch_voice_ext(index, assets, nls, resource_manager, audio);
        }

        if category == 15 && index == 2 {
            return self.ext_set_window_text(assets, nls, sprites.as_deref());
        }

        if let Some(name) = ext_opcode(category, index).and_then(|opcode| opcode.name) {
            match name {
                "key_canncel" => return self.dispatch_input_control(18, input),
                "get_pull_key" => return self.dispatch_input_control(32, input),
                "get_on_key" => return self.dispatch_input_control(33, input),
                "get_push_key" => return self.dispatch_input_control(34, input),
                "input_clear" => return self.dispatch_input_control(35, input),
                "get_async_key" => return self.dispatch_input_control(54, input),
                "text_init" => {
                    // Scripts only re-run text_init as part of a full UI
                    // re-init (boot, return to title). Drop buttons left
                    // behind by abandoned screens before they rebuild.
                    let mut sprites = sprites;
                    self.clear_select_scene(sprites.as_deref_mut());
                    self.clear_all_buttons(sprites);
                    return self.dispatch_text_stub(0, assets, nls, resource_manager, audio);
                }
                "text_set_icon" => {
                    return self.dispatch_text_stub(1, assets, nls, resource_manager, audio)
                }
                "text" => return self.dispatch_text_stub(2, assets, nls, resource_manager, audio),
                "text_hide" => {
                    return self.dispatch_text_stub(3, assets, nls, resource_manager, audio)
                }
                "text_show" => {
                    return self.dispatch_text_stub(4, assets, nls, resource_manager, audio)
                }
                "text_set_btn" => {
                    return self.dispatch_text_stub(5, assets, nls, resource_manager, audio)
                }
                "text_uninit" => {
                    return self.dispatch_text_stub(6, assets, nls, resource_manager, audio)
                }
                "text_set_rect_invalid_param" => {
                    return self.dispatch_text_stub(7, assets, nls, resource_manager, audio)
                }
                "text_clear" => {
                    return self.dispatch_text_stub(8, assets, nls, resource_manager, audio)
                }
                "text_get_time" => {
                    return self.dispatch_text_stub(10, assets, nls, resource_manager, audio)
                }
                "text_window_set_alpha" => {
                    return self.dispatch_text_stub(11, assets, nls, resource_manager, audio)
                }
                "text_voice_play" => {
                    return self.dispatch_text_stub(12, assets, nls, resource_manager, audio)
                }
                "text_set_icon_animation_time" => {
                    return self.dispatch_text_stub(14, assets, nls, resource_manager, audio)
                }
                "text_w" => {
                    return self.dispatch_text_stub(15, assets, nls, resource_manager, audio)
                }
                "text_a" => {
                    return self.dispatch_text_stub(16, assets, nls, resource_manager, audio)
                }
                "text_wa" => {
                    return self.dispatch_text_stub(17, assets, nls, resource_manager, audio)
                }
                "text_n" => {
                    return self.dispatch_text_stub(18, assets, nls, resource_manager, audio)
                }
                "text_cat" => {
                    return self.dispatch_text_stub(19, assets, nls, resource_manager, audio)
                }
                "set_history" => {
                    return self.dispatch_text_stub(20, assets, nls, resource_manager, audio)
                }
                "is_text_visible" => {
                    return self.dispatch_text_stub(21, assets, nls, resource_manager, audio)
                }
                "text_set_base" => {
                    return self.dispatch_text_stub(22, assets, nls, resource_manager, audio)
                }
                "enable_voice_cut" => {
                    return self.dispatch_text_stub(23, assets, nls, resource_manager, audio)
                }
                "is_voice_cut" => {
                    return self.dispatch_text_stub(24, assets, nls, resource_manager, audio)
                }
                "texttimecheckset" => {
                    return self.dispatch_text_stub(25, assets, nls, resource_manager, audio)
                }
                "text_set_color" => {
                    return self.dispatch_text_stub(28, assets, nls, resource_manager, audio)
                }
                "textredraw" => {
                    return self.dispatch_text_stub(29, assets, nls, resource_manager, audio)
                }
                "set_text_mode" => {
                    return self.dispatch_text_stub(30, assets, nls, resource_manager, audio)
                }
                "text_init_visualnovelmode" => {
                    return self.dispatch_text_stub(31, assets, nls, resource_manager, audio)
                }
                "text_set_icon_mode" => {
                    return self.dispatch_text_stub(32, assets, nls, resource_manager, audio)
                }
                "text_vn_br" => {
                    return self.dispatch_text_stub(33, assets, nls, resource_manager, audio)
                }
                "voice_set_volume" => {
                    return self.dispatch_text_stub(50, assets, nls, resource_manager, audio)
                }
                "voice_get_volume" => {
                    return self.dispatch_text_stub(51, assets, nls, resource_manager, audio)
                }
                "voice_enable" => {
                    return self.dispatch_text_stub(53, assets, nls, resource_manager, audio)
                }
                "is_voice_enable" => {
                    return self.dispatch_text_stub(54, assets, nls, resource_manager, audio)
                }
                "bgv_enable" => {
                    return self.dispatch_text_stub(58, assets, nls, resource_manager, audio)
                }
                "get_voice_ex_volume" => {
                    return self.dispatch_text_stub(59, assets, nls, resource_manager, audio)
                }
                "set_voice_ex_volume" => {
                    return self.dispatch_text_stub(60, assets, nls, resource_manager, audio)
                }
                "voice_check_enable" => {
                    return self.dispatch_text_stub(61, assets, nls, resource_manager, audio)
                }
                "voice_autopan_initialize" => {
                    return self.dispatch_text_stub(62, assets, nls, resource_manager, audio)
                }
                "voice_autopan_enable" => {
                    return self.dispatch_text_stub(63, assets, nls, resource_manager, audio)
                }
                "set_voice_autopan_size_over" => {
                    return self.dispatch_text_stub(64, assets, nls, resource_manager, audio)
                }
                "is_voice_autopan_enable" => {
                    return self.dispatch_text_stub(65, assets, nls, resource_manager, audio)
                }
                "bgv_mute" => {
                    return self.dispatch_text_stub(68, assets, nls, resource_manager, audio)
                }
                "set_bgv_volume" => {
                    return self.dispatch_text_stub(69, assets, nls, resource_manager, audio)
                }
                "get_bgv_volume" => {
                    return self.dispatch_text_stub(70, assets, nls, resource_manager, audio)
                }
                "set_bgv_auto_volume" => {
                    return self.dispatch_text_stub(71, assets, nls, resource_manager, audio)
                }
                "voice_mute" => {
                    return self.dispatch_text_stub(72, assets, nls, resource_manager, audio)
                }
                "wait" => return self.dispatch_wait_ext(0),
                "wait_click" => return self.dispatch_wait_ext(1),
                "wait_sync_begin" => return self.dispatch_wait_ext(2),
                "wait_sync_release" => return self.dispatch_wait_ext(3),
                "wait_sync_end" => return self.dispatch_wait_ext(4),
                "wait_clear" => return self.dispatch_wait_ext(6),
                "wait_click_no_anim" => return self.dispatch_wait_ext(7),
                "wait_sync_get_time" => return self.dispatch_wait_ext(8),
                "wait_time_push" => return self.dispatch_wait_ext(9),
                "wait_time_pop" => return self.dispatch_wait_ext(10),
                "set_font_size" => return self.dispatch_font_system_stub(27, input),
                "get_font_size" => return self.dispatch_font_system_stub(28, input),
                "get_font_type" => return self.dispatch_font_system_stub(29, input),
                "set_font_effect" => return self.dispatch_font_system_stub(30, input),
                "get_font_effect" => return self.dispatch_font_system_stub(31, input),
                "set_font_color" => return self.dispatch_font_system_stub(19, input),
                "get_font_color" => return self.dispatch_font_system_stub(55, input),
                "input_clear" => return self.dispatch_font_system_stub(35, input),
                "change_window_size" => return self.dispatch_font_system_stub(36, input),
                "change_aspect_mode" => return self.dispatch_font_system_stub(37, input),
                "get_aspect_mode" => return self.dispatch_font_system_stub(40, input),
                "enable_window_change" => return self.dispatch_font_system_stub(46, input),
                "is_enable_window_change" => return self.dispatch_font_system_stub(47, input),
                "history_skip" => return self.dispatch_font_system_stub(57, input),
                "save" => {
                    return self.dispatch_save_stub(0, assets, nls, resource_manager, sprites)
                }
                "load" => {
                    return self.dispatch_save_stub(1, assets, nls, resource_manager, sprites)
                }
                "save_set_title" => {
                    return self.dispatch_save_stub(2, assets, nls, resource_manager, sprites);
                }
                "save_data" => {
                    return self.dispatch_save_stub(3, assets, nls, resource_manager, sprites);
                }
                "save_set_thumbnail_size" => {
                    return self.dispatch_save_stub(4, assets, nls, resource_manager, sprites);
                }
                "save_set_font_size" => {
                    return self.dispatch_save_stub(7, assets, nls, resource_manager, sprites);
                }
                "is_save" => {
                    return self.dispatch_save_stub(9, assets, nls, resource_manager, sprites);
                }
                "savepoint" => {
                    return self.dispatch_save_stub(11, assets, nls, resource_manager, sprites);
                }
                "savetimedraw" => {
                    return self.dispatch_save_stub(13, assets, nls, resource_manager, sprites);
                }
                "save_set_text_rect" => {
                    return self.dispatch_save_stub(15, assets, nls, resource_manager, sprites);
                }
                "get_new_savefile" => {
                    return self.dispatch_save_stub(17, assets, nls, resource_manager, sprites);
                }
                "save_set_font_type" => {
                    return self.dispatch_save_stub(23, assets, nls, resource_manager, sprites);
                }
                "set_load_after_process" => {
                    return self.dispatch_save_stub(24, assets, nls, resource_manager, sprites);
                }
                "savesystemdata" => {
                    return self.dispatch_save_stub(25, assets, nls, resource_manager, sprites);
                }
                "save_set_font_effect" => {
                    return self.dispatch_save_stub(26, assets, nls, resource_manager, sprites);
                }
                "save_set_font_color_0x_0x" => {
                    return self.dispatch_save_stub(27, assets, nls, resource_manager, sprites);
                }
                "save_lock_not_open_savefileno" => {
                    return self.dispatch_save_stub(32, assets, nls, resource_manager, sprites);
                }
                "is_save_lock" => {
                    return self.dispatch_save_stub(33, assets, nls, resource_manager, sprites);
                }
                "is_prev_data" => {
                    return self.dispatch_save_stub(34, assets, nls, resource_manager, sprites);
                }
                "save_point_clear" => {
                    return self.dispatch_save_stub(35, assets, nls, resource_manager, sprites);
                }
                "save_point_lock" => {
                    return self.dispatch_save_stub(36, assets, nls, resource_manager, sprites);
                }
                "system_btn_set" if category == 12 => return self.dispatch_system_button_stub(0),
                "system_btn_release" if category == 12 => {
                    return self.dispatch_system_button_stub(1)
                }
                "system_btn_enable" if category == 12 => {
                    return self.dispatch_system_button_stub(2)
                }
                "history_init_0x_0x" => {
                    return self.dispatch_history_stub(0, assets, nls, resource_manager, audio, input)
                }
                "historybegin_lpbyte_ptagdata_sztext" => {
                    return self.dispatch_history_stub(1, assets, nls, resource_manager, audio, input)
                }
                "history_end" => {
                    return self.dispatch_history_stub(2, assets, nls, resource_manager, audio, input)
                }
                "history_get_height" => {
                    return self.dispatch_history_stub(5, assets, nls, resource_manager, audio, input)
                }
                "history_set_rect" => {
                    return self.dispatch_history_stub(10, assets, nls, resource_manager, audio, input)
                }
                "history_clear" => {
                    return self.dispatch_history_stub(11, assets, nls, resource_manager, audio, input)
                }
                "history_set" => {
                    return self.dispatch_history_stub(12, assets, nls, resource_manager, audio, input)
                }
                "history_get_text" => {
                    return self.dispatch_history_stub(20, assets, nls, resource_manager, audio, input)
                }
                "movie_play" => {
                    return self.ext_movie_play(
                        assets,
                        nls,
                        resource_manager,
                        sprites,
                        audio,
                    );
                }
                "msp_set_loop_sp_ep" => {
                    return self.ext_msp_set_loop_sp_ep(
                        assets,
                        nls,
                        resource_manager,
                        sprites,
                        task_system,
                    );
                }
                "msp_cls" => return self.ext_msp_cls(sprites, task_system),
                "msp_wait" => return self.ext_msp_wait(assets.extended_softpal),
                "msp_lock" => return self.ext_msp_lock(),
                "msp_unlock" => return self.ext_msp_unlock(),
                "msp_play" => return self.ext_msp_play(),
                "msp_stop" => return self.ext_msp_stop(),
                "create_thread" => return self.dispatch_thread_stub(0, point_table),
                "exit_thread" => return self.dispatch_thread_stub(1, point_table),
                "get_thread" => return self.dispatch_thread_stub(3, point_table),
                "create_message" => return self.dispatch_message_stub(0, point_table),
                "get_message" => return self.dispatch_message_stub(1, point_table),
                "get_message_param" => return self.dispatch_message_stub(2, point_table),
                "run_no_wait" => return self.dispatch_run_ext(1),
                "run_stack" => return self.dispatch_run_ext(2),
                "random" => return self.dispatch_random_ext(0),
                _ => {}
            }
        }

        let outcome = match category {
            2 => self.dispatch_text_stub(index, assets, nls, resource_manager, audio),
            3 => {
                self.dispatch_sprite_ext(index, assets, nls, resource_manager, sprites, task_system)
            }
            4 => self.dispatch_bgm_ext(index, assets, nls, resource_manager, audio),
            5 => self.dispatch_se_ext(index, assets, nls, resource_manager, audio),
            18 => match index {
                1 => self.ext_app_exec(assets, nls),
                3 => self.ext_string_not_equal(assets, nls),
                4 => self.ext_string_append(assets, nls),
                5 => self.ext_str_get_char_or_int(assets, nls),
                6 => self.ext_string_alloc(),
                7 => self.ext_string_copy_len(assets, nls),
                8 => self.ext_file_exist(assets, nls, resource_manager),
                9 => self.ext_wsprint_compat(assets, nls),
                10 => self.ext_check_disc(assets, nls),
                12 => self.ext_string_length(assets, nls),
                13 => self.ext_process_checkpoint_set(),
                14 => self.ext_update_access(assets, nls),
                15 => self.ext_system_task_value(),
                17 => self.ext_work_process_value(),
                18 => self.ext_string_replace(assets, nls),
                21 => self.ext_strlenf(assets, nls),
                28 => self.ext_attach_work_process(),
                29 => self.ext_detach_work_process(),
                30 => self.ext_open_file(assets, nls, resource_manager),
                31 => self.ext_read_file(),
                32 => self.ext_close_file_not_handle(),
                33 => self.ext_set_file_pointer(),
                34 => self.ext_file_string(),
                35 => self.ext_set_last_process(),
                36 => self.ext_sz_buf(assets, nls, resource_manager),
                37 => self.ext_get_private_profile_int(assets, nls, resource_manager),
                38 => self.ext_write_private_profile_int(assets, nls, resource_manager),
                39 => self.ext_write_private_profile_string(assets, nls, resource_manager),
                40 => self.ext_access_clear(),
                // Later SoftPAL scripts use this zero-argument clock query in
                // their own elapsed-time loops. Return the same PAL clock that
                // drives wait_sync and renderer effects.
                119 if assets.registered_softpal => self.ext_string_substr_bytes(assets, nls),
                121 => ExtCallOutcome::Value(self.pal_time_ms as i32),
                _ => ExtCallOutcome::Skip,
            },
            7 => self.dispatch_wait_ext(index),
            8 => self.dispatch_button_ext(index, assets, nls, resource_manager, sprites, input, audio),
            9 => self.dispatch_font_system_stub(index, input),
            10 => self.dispatch_save_stub(index, assets, nls, resource_manager, sprites),
            12 => self.dispatch_system_button_stub(index),
            14 => self.dispatch_history_stub(index, assets, nls, resource_manager, audio, input),
            6 => self.dispatch_select_ext(index, assets, nls, resource_manager, sprites, input),
            15 => self.dispatch_misc_system_stub(index, assets.extended_softpal),
            16 => self.dispatch_window_effect_stub(index),
            21 => self.dispatch_thread_stub(index, point_table),
            22 => self.dispatch_run_ext(index),
            17 => {
                self.action_state.extended = assets.extended_softpal;
                self.dispatch_action(index, sprites)
            }
            20 => self.dispatch_random_ext(index),
            23 => self.dispatch_message_stub(index, point_table),
            _ => ExtCallOutcome::Skip,
        };


        outcome
    }

    fn dispatch_wait_ext(&mut self, index: u16) -> ExtCallOutcome {
        self.adv_wait_checkpoint_pending = false;
        match index {
            // wait(duration_ms,skip_cancel): Game.exe sub_444F40 pops two
            // values, records start time, and rewinds the PC until the duration
            // expires or the native cancel path completes.
            0 => {
                let args = self.pop_ext_args(2);
                let duration_ms = args.first().copied().unwrap_or(1);
                let skip_cancel = args.get(1).copied().unwrap_or(0);
                log::debug!("[trace-script] wait duration_ms={duration_ms}");
                if skip_cancel != 0 {
                    log::debug!("[trace-script] wait skip_cancel={skip_cancel}");
                }
                let request = if skip_cancel != 0 {
                    if duration_ms < 0 {
                        // No timeout: only the cancel (click) path completes
                        // the wait.  Koikake's HIDE button callback uses
                        // wait(-1, 1) to park until the player clicks the
                        // text window back; clamping -1 to 1ms restored the
                        // window on the very next frame.
                        WaitRequest::Click
                    } else {
                        WaitRequest::ClickOrTime(duration_ms.max(1) as u32)
                    }
                } else {
                    if duration_ms == -1 {
                        WaitRequest::Frame(-1)
                    } else {
                        WaitRequest::Time((duration_ms as u32).max(1))
                    }
                };
                ExtCallOutcome::Wait { value: 1, request }
            }
            // wait_click(duration_ms): Game.exe sub_444DE0 uses -1 as the
            // native text-task completion branch (ctx[1050] clear or
            // sub_43A860 + sub_44A010), while non-negative values are
            // click-or-time waits that rewind PC.  IDB also shows the text
            // producers set ctx+804084, so the native scheduler keeps pumping
            // the PAL text task instead of letting the script immediately
            // overwrite the line.  In the portable VM, map the -1 branch to an
            // ADV click wait when text is visible: first click completes the
            // reveal in Engine::consume_text_reveal_push, the next click lets
            // this wait task finish.
            1 => {
                let args = self.pop_ext_args(1);
                let duration_ms = args.first().copied().unwrap_or(-1);
                log::debug!("[trace-script] wait_click duration_ms={duration_ms}");
                if duration_ms == -1 {
                    if self.text_state.visible && self.text_state.last_text_value != 0 {
                        self.text_state.show_wait_mark = true;
                        self.text_state.dirty = true;
                        self.adv_wait_checkpoint_pending = true;
                        ExtCallOutcome::Wait {
                            value: 1,
                            request: WaitRequest::Click,
                        }
                    } else {
                        ExtCallOutcome::Value(1)
                    }
                } else {
                    self.text_state.show_wait_mark = false;
                    ExtCallOutcome::Wait {
                        value: 1,
                        request: WaitRequest::ClickOrTime(duration_ms.max(1) as u32),
                    }
                }
            }
            2 => {
                self.pop_ext_args(0);
                self.wait_sync_begin_ms = self.pal_time_ms;
                self.wait_sync_release = None;
                log::debug!("[trace-script] wait_sync_begin");
                ExtCallOutcome::Value(1)
            }
            3 => {
                if let Some(active) = self.wait_sync_release {
                    let elapsed = self.pal_time_ms.wrapping_sub(active.start_ms);
                    if elapsed >= active.duration_ms {
                        self.wait_sync_release = None;
                        log::debug!("[trace-script] wait_sync_release elapsed_ms={elapsed}");
                        ExtCallOutcome::Value(1)
                    } else {
                        let remaining = active.duration_ms - elapsed;
                        log::debug!("[trace-script] wait_sync_release remaining_ms={remaining}");
                        ExtCallOutcome::Wait {
                            value: 1,
                            request: WaitRequest::Time(remaining.max(1)),
                        }
                    }
                } else {
                    let args = self.pop_ext_args(1);
                    let duration_ms = args.first().copied().unwrap_or(1).max(1) as u32;
                    self.wait_sync_release = Some(WaitSyncRelease {
                        duration_ms,
                        start_ms: self.pal_time_ms,
                    });
                    log::debug!("[trace-script] wait_sync duration_ms={duration_ms}");
                    ExtCallOutcome::Wait {
                        value: 1,
                        request: WaitRequest::Time(duration_ms.max(1)),
                    }
                }
            }
            4 => {
                self.pop_ext_args(0);
                self.wait_sync_begin_ms = 0;
                self.wait_sync_release = None;
                log::debug!("[trace-script] wait_sync_end");
                ExtCallOutcome::Value(1)
            }
            5 => {
                self.pop_ext_args(0);
                self.wait_sync_release = None;
                log::debug!("[trace-script] wait_sync_step");
                ExtCallOutcome::Wait {
                    value: 1,
                    request: WaitRequest::Frame(1),
                }
            }
            6 => {
                self.pop_ext_args(0);
                self.wait_sync_begin_ms = 0;
                self.wait_sync_release = None;
                self.wait_time_stack.clear();
                log::debug!("[trace-script] wait_clear");
                ExtCallOutcome::Value(1)
            }
            // wait_click_no_anim(duration_ms): sub_444C90 shares wait_click
            // blocking semantics while bypassing wait-icon animation state.
            7 => {
                let args = self.pop_ext_args(1);
                let duration_ms = args.first().copied().unwrap_or(1);
                log::debug!("[trace-script] wait_click_no_anim duration_ms={duration_ms}");
                self.text_state.show_wait_mark = false;
                if duration_ms == -1 {
                    // sub_444C90 shares the -1 text-task completion shortcut
                    // with wait_click, only bypassing wait-icon animation.
                    if self.text_state.visible && self.text_state.last_text_value != 0 {
                        ExtCallOutcome::Wait {
                            value: 1,
                            request: WaitRequest::Click,
                        }
                    } else {
                        ExtCallOutcome::Value(1)
                    }
                } else {
                    ExtCallOutcome::Wait {
                        value: 1,
                        request: WaitRequest::ClickOrTime(duration_ms.max(1) as u32),
                    }
                }
            }
            8 => {
                self.pop_ext_args(0);
                let elapsed = self.pal_time_ms.wrapping_sub(self.wait_sync_begin_ms) as i32;
                log::debug!("[trace-script] wait_sync_get_time elapsed_ms={elapsed}");
                ExtCallOutcome::Value(elapsed)
            }
            9 => {
                self.pop_ext_args(0);
                self.wait_time_stack.push(self.pal_time_ms);
                log::debug!("[trace-script] wait_time_push time_ms={}", self.pal_time_ms);
                ExtCallOutcome::Value(1)
            }
            10 => {
                self.pop_ext_args(0);
                let elapsed = self
                    .wait_time_stack
                    .pop()
                    .map(|start| self.pal_time_ms.wrapping_sub(start) as i32)
                    .unwrap_or(0);
                log::debug!("[trace-script] wait_time_pop elapsed_ms={elapsed}");
                ExtCallOutcome::Value(elapsed)
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    fn dispatch_random_ext(&mut self, index: u16) -> ExtCallOutcome {
        match index {
            0 => {
                let args = self.pop_ext_args(2);
                if args.len() < 2 {
                    return ExtCallOutcome::Block;
                }
                let lo = args[0].min(args[1]);
                let hi = args[0].max(args[1]);
                let span = hi.saturating_sub(lo).saturating_add(1).max(1);
                let value = lo.saturating_add(
                    (self.random_state.random_ex().unsigned_abs() % span as u32) as i32,
                );
                ExtCallOutcome::Value(value)
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    fn dispatch_font_system_stub(
        &mut self,
        index: u16,
        input: Option<&PalInputState>,
    ) -> ExtCallOutcome {
        match index {
            12 => {
                // Game.exe `input_mouse_to_mem(dst_x, dst_y)`: copy the live PAL
                // mouse position into caller-chosen work slots (-1 skips a
                // lane).  The SOUND/SYSTEM slider track-click handlers read the
                // position back from these slots to compute the knob percent.
                let args = self.pop_ext_args(2);
                if args.len() < 2 {
                    return ExtCallOutcome::Block;
                }
                let (mouse_x, mouse_y) = input
                    .map(|input| input.mouse_position())
                    .unwrap_or((-1, -1));
                let dst_x = args[0];
                let dst_y = args[1];
                if dst_x >= 0 {
                    self.write_temp_mem_absolute(dst_x, mouse_x);
                }
                if dst_y >= 0 {
                    self.write_temp_mem_absolute(dst_y, mouse_y);
                }
                ExtCallOutcome::Value(1)
            }
            0 => {
                let args = self.pop_ext_args(1);
                let enable = args.first().copied().unwrap_or(0) != 0;
                if enable && self.system_state.skip_gate() == 0 && !self.current_line_read {
                    // Gate A (koikake.exe 0x428E10): read-only skip mode
                    // refuses to latch while the on-screen line is unread;
                    // native posts the slot-1 cancel message and leaves the
                    // skip byte untouched.
                    log::debug!("[trace-text] skip_set refused: current line unread");
                    self.post_engine_message(1, 0);
                    return ExtCallOutcome::Value(1);
                }
                self.text_skip_enabled = enable;
                log::debug!("[trace-text] skip_set enabled={}", self.text_skip_enabled);
                ExtCallOutcome::Value(1)
            }
            1 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.text_skip_enabled as i32)
            }
            2 => {
                let args = self.pop_ext_args(1);
                self.text_auto_enabled = args.first().copied().unwrap_or(0) != 0;
                log::debug!("[trace-text] auto_set enabled={}", self.text_auto_enabled);
                ExtCallOutcome::Value(1)
            }
            3 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.text_auto_enabled as i32)
            }
            9 => {
                // koikake.exe 0x428AE0 returns task data +8 (the live window
                // mode written by window_change_mode) to the destination
                // slot.  (The "effect_enable_is" naming came from a
                // different Game.exe build.)
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_state.script_window_mode())
            }
            4 => {
                // Game.exe sub_4389F0: auto_set_speed. It pops one config value,
                // clamps values above 100, and stores it at PalTaskGetTaskData
                // +28. Text reveal timing reads this portable system value.
                let args = self.pop_ext_args(1);
                let speed = args.first().copied().unwrap_or(0).clamp(0, 100);
                self.system_state.set_auto_speed_percent(speed);
                ExtCallOutcome::Value(1)
            }
            6 => {
                // Game.exe sub_438950: window_change_mode. Native calls
                // PalWindowChangeMode(mode) and stores task-data +8.
                let args = self.pop_ext_args(1);
                let mode = args.first().copied().unwrap_or(0);
                self.system_state.change_window_mode(mode);
                ExtCallOutcome::Value(1)
            }
            7 => {
                // koikake.exe 0x428B70 writes task data +0xC, whose only
                // native consumers are the two skip gates (skip_set refusal
                // on unread lines and the display-time cancel message):
                // 0 = skip read text only, nonzero = skip everything.  The
                // koikake SYSTEM screen's スキップタイプ toggle routes here.
                // Other Game.exe builds use the same slot as a pending window
                // mode cache, so keep that write as well.
                let args = self.pop_ext_args(1);
                let mode = args.first().copied().unwrap_or(0);
                self.system_state.set_skip_gate(mode);
                self.system_state.set_window_mode_cache(mode);
                ExtCallOutcome::Value(1)
            }
            8 => {
                // Game.exe sub_438890: effect_enable. Native calls
                // PalEffectEnable(flag) and stores task-data +16.
                let args = self.pop_ext_args(1);
                let enabled = args.first().copied().unwrap_or(0);
                self.system_state.set_effect_enabled(enabled);
                self.effect_system.set_enabled(enabled != 0);
                ExtCallOutcome::Value(1)
            }
            10 => {
                // koikake.exe 0x428AB0 returns task data +0xC: the value
                // written by index 7 (skip-type gate; a pending window-mode
                // cache in other Game.exe builds).  The koikake SYSTEM
                // screen syncs its スキップタイプ toggle from this getter.
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_state.skip_gate())
            }
            21 => {
                // Game.exe memory_stack_push snapshots one 0x4000-byte task work
                // bank (ctx+0x1500) onto a 32-entry stack. user_mem/system_mem
                // live outside that range (ctx+0x22F40/+0x27F40) and must NOT be
                // rolled back on pop: menu handlers store persistent settings
                // there (e.g. the SYSTEM screen's mem_system[0] initialized flag
                // and its toggle values). memdat[158] page requests also survive.
                self.pop_ext_args(0);
                if self.memory_state_stack.len() < 32 {
                    self.memory_state_stack.push(ScriptMemorySnapshot {
                        temp_mem: self.temp_mem.clone(),
                    });
                } else {
                    log::warn!("[trace-system] memory_stack_push overflow");
                }
                ExtCallOutcome::Value(1)
            }
            22 => {
                // Restores only the pushed work-bank snapshot; user_mem,
                // system_mem, and the Mem.dat shadow keep their current values.
                self.pop_ext_args(0);
                if let Some(snapshot) = self.memory_state_stack.pop() {
                    self.temp_mem = snapshot.temp_mem;
                } else {
                    log::warn!("[trace-system] memory_stack_pop underflow");
                }
                ExtCallOutcome::Value(1)
            }
            23 => {
                // Game.exe sub_437CD0: list_stack_push_point. Native resolves a
                // point to a process-relative address and PalListPushes it.
                // Store the point id and bridge the menu continuation into the
                // existing pending jump path until the full PalList scheduler is
                // byte-for-byte modeled.
                let args = self.pop_ext_args(1);
                let point_id = args.first().copied().unwrap_or(0);
                if point_id > 0 {
                    self.list_point_stack.push(point_id as u32);
                    self.pending_jump_point = Some(point_id as u32);
                    log::debug!(
                        "[trace-system] list_stack_push_point point[{point_id}] depth={} mode={}",
                        self.list_point_stack.len(),
                        self.menu_transition_mode,
                    );
                }
                ExtCallOutcome::Value(1)
            }
            24 => {
                // Game.exe sub_437C00: list_stack_pop_count. Native pops one
                // count/control value, then removes one or more PalList entries.
                let args = self.pop_ext_args(1);
                let count_mode = args.first().copied().unwrap_or(0);
                self.menu_transition_mode = count_mode;
                let pop_count = if count_mode <= 0 {
                    1
                } else {
                    count_mode as usize
                };
                for _ in 0..pop_count {
                    if self.list_point_stack.pop().is_none() {
                        break;
                    }
                }
                log::debug!(
                    "[trace-system] list_stack_pop_count count_mode={count_mode} depth={}",
                    self.list_point_stack.len()
                );
                ExtCallOutcome::Value(1)
            }
            25 => {
                // Game.exe sub_437BA0 consumes no VM arguments and clears the
                // ctx+804244 scratch latch used by the adjacent list helpers.
                self.pop_ext_args(0);
                self.system_scratch_value = 0;
                ExtCallOutcome::Value(1)
            }
            26 => {
                // Game.exe sub_437B80 consumes no VM arguments and writes the
                // ctx+804244 scratch latch to the extcall destination.
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_scratch_value)
            }
            53 => {
                // Game.exe sub_437BC0: list_stack_get_count. Native returns
                // PalListGetDataCount(ctx+804228, 2); scripts use it to unwind
                // overlay/menu continuations back to a saved depth.
                self.pop_ext_args(0);
                let depth = self.list_point_stack.len() as i32;
                log::debug!("[trace-system] list_stack_get_count depth={depth}");
                ExtCallOutcome::Value(depth)
            }
            5 => {
                // VmOpc_AutoGetTime @ 0x004389B0. Native code returns the
                // configured auto-advance interval; the current compatible
                // state stores no separate auto timer yet, so expose the
                // conservative disabled value while preserving the real zero-arg
                // stack discipline.
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_state.auto_speed_percent())
            }
            17 => {
                // Game.exe sub_438090 (`set_language`) pops one PAL font type
                // and calls PalFontSetType. It does not return the old value.
                let args = self.pop_ext_args(1);
                if let Some(font_type) = args.first().copied() {
                    self.font_state.set_type_word(font_type as i32);
                    self.system_state.set_language(font_type);
                    // On a release that ships its own faces the type also names
                    // the text face style (the SYSTEM screen's 文本字体 row).
                    if let Some(style) = FaceStyle::from_font_type(font_type.max(0) as u16) {
                        self.select_style_face(style);
                    }
                }
                ExtCallOutcome::Value(1)
            }
            60 => {
                // FHD release `set_language` (category 9 index 60, distinct
                // from the PAL font type the index-17 call sets). The language
                // row on the SYSTEM screen stores the picked text field in
                // `mem_system[102]` and passes it here, and the boot path calls
                // this once with that stored value. Only the localized fields
                // are offered on screen (a Japanese install never calls this
                // index at all), while a fresh install leaves the stored value
                // at 0 — so 0 keeps the language the host configured
                // (`--lang` / launcher) instead of resetting a localized
                // release to its Japanese field.
                let args = self.pop_ext_args(1);
                let value = args.first().copied().unwrap_or(0);
                if value != 0 {
                    if let Some(lang) = TextLang::from_field_index(value) {
                        log::debug!(
                            "[trace-text] set_language field={value} -> {}",
                            lang.as_str()
                        );
                        self.apply_text_language(lang);
                    }
                }
                ExtCallOutcome::Value(1)
            }
            18 => self.dispatch_input_control(18, None),
            11 | 15 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(1)
            }
            12 => {
                // Game category 9 index 12 (`sub_438770`) pops Mem.dat
                // destinations for mouse x/y and writes each destination when
                // it is not -1.
                let args = self.pop_ext_args(2);
                if args.len() < 2 {
                    return ExtCallOutcome::Block;
                }
                if args[0] >= 0 {
                    self.write_mem_dat_word(args[0] as usize, 0);
                }
                if args[1] >= 0 {
                    self.write_mem_dat_word(args[1] as usize, 0);
                }
                ExtCallOutcome::Value(1)
            }
            14 => {
                // Game category 9 index 14 (`sub_438460`) clears a 0x1000 byte
                // VM scratch/memory-bank range and consumes no VM stack args.
                self.pop_ext_args(0);
                self.temp_mem.fill(0);
                ExtCallOutcome::Value(1)
            }
            16 => {
                // Game.exe sub_438390 (`unload_font`) pops no args and calls
                // PalFontUnload.
                self.pop_ext_args(0);
                self.font_state.unload_bitmap_font();
                ExtCallOutcome::Value(1)
            }
            19 => {
                let args = self.pop_ext_args(2);
                if args.len() >= 2 {
                    self.font_state.set_color(
                        args[0] as u32,
                        args[1] as u32,
                    );
                }
                ExtCallOutcome::Value(1)
            }
            20 => {
                // Game.exe sub_4382D0 (`load_font_ex`) pops one File.dat string
                // id, appends .tga, unloads the previous extended font, and
                // loads the replacement. Resource IO is handled by the renderer,
                // but the script-visible loaded latch must follow the syscall.
                self.pop_ext_args(1);
                self.font_state.set_ex_font_loaded(true);
                ExtCallOutcome::Value(1)
            }
            27 => {
                let args = self.pop_ext_args(1);
                let old = i32::from(self.font_state.font_size());
                if let Some(size) = args.first().copied() {
                    self.font_state.set_font_size(size as u16);
                }
                ExtCallOutcome::Value(old)
            }
            28 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.font_state.font_size()))
            }
            29 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.font_state.font_type()))
            }
            30 => {
                let args = self.pop_ext_args(1);
                if let Some(effect) = args.first().copied() {
                    self.font_state.set_effect(effect.max(0) as u16);
                }
                ExtCallOutcome::Value(1)
            }
            31 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.font_state.effect()))
            }
            32..=35 | 54 => self.dispatch_input_control(index, None),
            36 => {
                let args = self.pop_ext_args(2);
                if args.len() < 2 {
                    return ExtCallOutcome::Block;
                }
                self.system_state.set_window_size(args[0], args[1]);
                ExtCallOutcome::Value(1)
            }
            37 => {
                let args = self.pop_ext_args(1);
                let mode = args.first().copied().unwrap_or(0);
                self.system_state.change_aspect_mode(mode);
                ExtCallOutcome::Value(1)
            }
            38 => {
                let args = self.pop_ext_args(1);
                self.system_state
                    .set_aspect_position_enabled(args.first().copied().unwrap_or(0));
                ExtCallOutcome::Value(1)
            }
            39 | 40 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_state.aspect_mode())
            }
            41 => {
                self.pop_ext_args(2);
                ExtCallOutcome::Value(1)
            }
            42 | 43 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(0)
            }
            44 => {
                self.pop_ext_args(1);
                self.system_state.clear_system_paths();
                ExtCallOutcome::Value(1)
            }
            45 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(1)
            }
            46 => {
                let args = self.pop_ext_args(1);
                self.system_state
                    .set_window_change_enabled(args.first().copied().unwrap_or(1));
                ExtCallOutcome::Value(1)
            }
            47 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_state.window_change_enabled())
            }
            48 => {
                self.pop_ext_args(1);
                self.system_state.set_cursor_null(true);
                ExtCallOutcome::Value(1)
            }
            49 => {
                let args = self.pop_ext_args(1);
                self.system_state
                    .set_hide_cursor_time(args.first().copied().unwrap_or(0));
                ExtCallOutcome::Value(1)
            }
            50 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_state.hide_cursor_time())
            }
            51 => {
                // Game category 9 index 51 (`scene_skip`, koikake.exe
                // 0x427BA0) sets the scene-skip latch at ADVctx+0x31830 and
                // stores its argument at +0x31834; while latched, native
                // GetSkipState returns -1 so every wait, reveal, and effect
                // behaves as skipped until cancel_scene_skip, a select entry,
                // or arrival at unread text clears it.
                let args = self.pop_ext_args(1);
                self.scene_skip_active = true;
                log::debug!(
                    "[trace-system] scene_skip arg={}",
                    args.first().copied().unwrap_or(0)
                );
                ExtCallOutcome::Value(1)
            }
            57 | 60 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(1)
            }
            52 => {
                // Game category 9 index 52 (`cancel_scene_skip`, koikake.exe
                // 0x427B40) clears the scene-skip latch and updates save-point
                // state only when the latch is set. It consumes no VM stack
                // arguments.
                self.pop_ext_args(0);
                if self.scene_skip_active {
                    log::debug!("[trace-system] cancel_scene_skip");
                }
                self.scene_skip_active = false;
                ExtCallOutcome::Value(1)
            }
            55 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.font_state.color().0 as i32)
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    /// Game category 2 text/message-window extcalls.
    ///
    /// Evidence: shared `ExtSig` entries plus Game.sqlite text handler traces show
    /// these calls pop their parameters from the script argument stack and write a
    /// small integer status/result to the extcall destination.  PAL.dll text
    /// rendering is not a single export here; the VM mutates `TextSubsystemState`,
    /// and the renderer consumes that state later when building text/name sprites.
    /// Glyph reveal duration and icon animation use the recovered native state
    /// dependencies while remaining portable across render backends.
    fn dispatch_text_stub(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        match index {
            0 => {
                let args = self.pop_ext_args(8);
                let ordered = ext_args_source_order::<8>(&args);
                self.text_state.initialized = true;
                self.text_state.visible = true;
                self.text_state.mode = ordered[0];
                self.text_state.init_args = ordered;
                self.font_state.set_font_size(ordered[7].max(1) as u16);
                let (text_color, effect_color) = self.font_state.color();
                self.text_state.text_color = text_color;
                self.text_state.text_effect_color = effect_color;
                self.text_state.last_event_time_ms = self.pal_time_ms;
                self.text_state.dirty = true;
                log::debug!("[trace-text] text_init args={ordered:?}");
                ExtCallOutcome::Value(1)
            }
            1 => {
                let args = self.pop_ext_args(4);
                let ordered = ext_args_source_order::<4>(&args);
                self.text_state.icon = ordered[3];
                log::debug!("[trace-text] text_set_icon args={ordered:?}");
                ExtCallOutcome::Value(1)
            }
            2 => {
                let args = self.pop_ext_args(4);
                let ordered = ext_args_source_order::<4>(&args);
                self.text_state.last_text_args = ordered;
                self.text_state.last_text_value = ordered[1];
                self.text_state.last_event_time_ms = self.pal_time_ms;
                self.text_state.reveal_start_ms = self.pal_time_ms;
                self.text_state.reveal_duration_ms =
                    self.text_reveal_duration_ms(ordered[1], ordered[0], assets, nls);
                self.text_state.reveal_enabled = self.text_state.reveal_duration_ms > 0;
                self.text_state.show_wait_mark = false;
                self.text_state.visible = true;
                self.text_state.dirty = true;
                self.push_history_text_record(ordered, assets, nls);
                self.try_play_text_voice(ordered[3], assets, nls, resource_manager, audio);
                let reveal_ms = self.text_state.reveal_duration_ms;
                log::debug!("[trace-text] text args={ordered:?} reveal_ms={}", reveal_ms);
                // Game.exe AdvCommandText updates the text context, sets the
                // VM scheduler flag at ctx+804084, and the text task advances
                // through sub_43DB40 before script execution resumes.  Waiting
                // for the recovered reveal duration prevents a later text
                // command from overwriting this line before the typewriter pass
                // is visible.
                ExtCallOutcome::Wait {
                    value: 1,
                    request: self
                        .text_wait_request_after_submit(ordered[1], reveal_ms, assets, nls),
                }
            }
            3 => {
                let args = self.pop_ext_args(1);
                self.text_state.visible = false;
                self.text_state.reveal_enabled = false;
                self.text_state.pending_alpha.clear();
                self.text_state.dirty = true;
                log::debug!("[trace-text] text_hide args={args:?}");
                ExtCallOutcome::Value(1)
            }
            4 => {
                self.pop_ext_args(1);
                self.text_state.visible = true;
                self.text_state.last_event_time_ms = self.pal_time_ms;
                self.text_state.dirty = true;
                ExtCallOutcome::Value(1)
            }
            5 => {
                let args = self.pop_ext_args(1);
                let button = args.first().copied().unwrap_or(0);
                // VmExtcall_TextSetBtn @ 0x43F1A0 pops one text button id,
                // stores it in the active text context, and clears that
                // button's per-window reaction slot.  It does not create the
                // button; surrounding text/window setup uses the id later.
                self.text_state.button = button;
                log::debug!("[trace-text] text_set_btn button={button}");
                ExtCallOutcome::Value(1)
            }
            6 => {
                self.pop_ext_args(0);
                let sprite = self.text_state.sprite;
                let name_sprite = self.text_state.name_sprite;
                let text_color = self.text_state.text_color;
                let text_effect_color = self.text_state.text_effect_color;
                self.text_state = TextSubsystemState {
                    sprite,
                    name_sprite,
                    text_color,
                    text_effect_color,
                    dirty: true,
                    ..TextSubsystemState::default()
                };
                log::debug!("[trace-text] text_uninit");
                ExtCallOutcome::Value(1)
            }
            8 => {
                self.pop_ext_args(0);
                self.text_state.last_text_value = 0;
                self.text_state.last_text_args = [0; 4];
                self.text_state.reveal_enabled = false;
                self.text_state.pending_alpha.clear();
                self.text_state.last_event_time_ms = self.pal_time_ms;
                self.text_state.dirty = true;
                log::debug!("[trace-text] text_clear");
                ExtCallOutcome::Value(1)
            }
            9 => {
                // koikake.exe category-2 index 9 is the message-speed setter:
                // the SYSTEM screen text-speed slider pushes `100 - percent`
                // here (its only script use), and native stores the value at
                // task data +0x20 with the same 0..100 units as auto speed.
                // The older "text_clear_ex" reading came from a neighboring
                // slot in a different Game.exe build and does not match this
                // call pattern.
                let args = self.pop_ext_args(1);
                let speed = args.first().copied().unwrap_or(0);
                self.system_state.set_text_speed_percent(speed);
                log::debug!(
                    "[trace-text] text_set_speed speed={}",
                    self.system_state.text_speed_percent()
                );
                ExtCallOutcome::Value(1)
            }
            10 => {
                // Category-2 index 10 returns the configured message speed
                // (native writes task data +0x20 to the destination slot).
                // The SYSTEM screen computes its slider percent as
                // `100 - speed` when the page opens.
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.system_state.text_speed_percent())
            }
            11 => {
                let args = self.pop_ext_args(2);
                self.text_state.alpha = args.first().copied().unwrap_or(self.text_state.alpha);
                self.text_state.dirty = true;
                log::debug!(
                    "[trace-text] text_window_set_alpha args={args:?} alpha={}",
                    self.text_state.alpha
                );
                ExtCallOutcome::Value(1)
            }
            12 => {
                let args = self.pop_ext_args(1);
                let voice_value = args.first().copied().unwrap_or(0);
                self.text_state.last_text_value = voice_value;
                self.try_play_text_voice(voice_value, assets, nls, resource_manager, audio);
                ExtCallOutcome::Value(1)
            }
            13 => {
                // Game.exe sub_43EE00 consumes no VM arguments and sets
                // text_ctx+4192. Native uses that flag to request the next text
                // task redraw; the portable renderer exposes the same effect by
                // marking the ADV text surface dirty.
                self.pop_ext_args(0);
                self.text_state.dirty = true;
                self.text_state.last_event_time_ms = self.pal_time_ms;
                log::debug!("[trace-text] text_task_redraw_flag");
                ExtCallOutcome::Value(1)
            }
            14 => {
                let args = self.pop_ext_args(2);
                let ordered = ext_args_source_order::<2>(&args);
                self.text_state.icon = ordered[1];
                log::debug!("[trace-text] text_set_icon_animation_time args={ordered:?}");
                ExtCallOutcome::Value(1)
            }
            15 | 16 | 17 => {
                let args = self.pop_ext_args(4);
                let ordered = ext_args_source_order::<4>(&args);
                self.text_state.last_text_args = ordered;
                self.text_state.last_text_value = ordered[1];
                self.text_state.last_event_time_ms = self.pal_time_ms;
                self.text_state.reveal_start_ms = self.pal_time_ms;
                self.text_state.reveal_duration_ms = if index == 16 {
                    0
                } else {
                    self.text_reveal_duration_ms(ordered[1], ordered[0], assets, nls)
                };
                self.text_state.reveal_enabled = self.text_state.reveal_duration_ms > 0;
                self.text_state.show_wait_mark = false;
                self.text_state.visible = true;
                self.text_state.dirty = true;
                self.push_history_text_record(ordered, assets, nls);
                self.try_play_text_voice(ordered[3], assets, nls, resource_manager, audio);
                let reveal_ms = self.text_state.reveal_duration_ms;
                log::debug!(
                    "[trace-text] text_wait index={index} args={ordered:?} reveal_ms={}",
                    reveal_ms
                );
                // Game.exe sub_43FFC0 (text_w), sub_43FC60, and sub_43F900
                // set ctx+804084 after mutating the ADV text context.  Native
                // scheduling lets the PAL text task run before the next script
                // command; model that as a text-reveal wait when the line has
                // a nonzero reveal duration.
                ExtCallOutcome::Wait {
                    value: 1,
                    request: self
                        .text_wait_request_after_submit(ordered[1], reveal_ms, assets, nls),
                }
            }
            18 | 19 => {
                self.pop_ext_args(0);
                self.text_state.last_event_time_ms = self.pal_time_ms;
                ExtCallOutcome::Value(1)
            }
            20 => {
                let args = self.pop_ext_args(1);
                self.text_state.history_enabled = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            21 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.text_state.visible))
            }
            22 => {
                let args = self.pop_ext_args(4);
                let ordered = ext_args_source_order::<4>(&args);
                self.text_state.base = ordered[3];
                log::debug!("[trace-text] text_set_base args={ordered:?}");
                ExtCallOutcome::Value(1)
            }
            23 => {
                let args = self.pop_ext_args(1);
                self.text_state.voice_cut_enabled = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            24 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.text_state.voice_cut_enabled))
            }
            25 => {
                // VmExtcall_TextTimeCheckSet @ 0x0043E590 pops
                // (string_id, x, y, width, height), resolves the string through
                // sub_44B120, then creates AdvTextTimeCheckTask via sub_43A470.
                // We currently preserve the script-visible stack/state effects;
                // the native helper's temporary check sprite is still tracked as
                // an implementation gap in the shared ExtSig.
                let args = self.pop_ext_args(5);
                let ordered = ext_args_source_order::<5>(&args);
                self.text_state.last_event_time_ms = self.pal_time_ms;
                self.text_state.dirty = true;
                log::debug!("[trace-text] texttimecheckset args={ordered:?}");
                ExtCallOutcome::Value(1)
            }
            26 => {
                // VmExtcall_TextTaskFree @ 0x0043E550. The native handler frees
                // pending text animation/task state. In this renderer the text
                // reveal task is represented by flags in TextSubsystemState, so
                // clearing those flags matches the externally visible effect.
                self.pop_ext_args(0);
                self.text_state.reveal_enabled = false;
                self.text_state.dirty = true;
                ExtCallOutcome::Value(1)
            }
            27 => {
                // Game.exe sub_43E4A0 locks the current text sprite, zeros a
                // repeated alpha byte pattern, unlocks it, and queues the text
                // redraw helper.  Our generated text surface does not keep the
                // native per-glyph alpha work buffer, so the portable effect is
                // to finish any reveal pass and force a redraw.
                self.pop_ext_args(0);
                self.text_state.reveal_enabled = false;
                self.text_state.dirty = true;
                ExtCallOutcome::Value(1)
            }
            28 => {
                // VmExtcall_TextSetColor @ 0x0043E2E0 pops
                // (slot, label_string, text_color, effect_color).  Native stores
                // this in the PAL text task's 16-entry color table.  The portable
                // renderer has one active ADV color pair, so keep the current
                // colors synchronized with the latest entry while preserving all
                // argument consumption for script/decompiler parity.
                let args = self.pop_ext_args(4);
                let ordered = ext_args_source_order::<4>(&args);
                let color = ordered[2] as u32;
                let effect_color = ordered[3] as u32;
                self.text_state.text_color = color;
                self.text_state.text_effect_color = effect_color;
                self.text_state.reveal_enabled = false;
                self.text_state.dirty = true;
                log::debug!(
                    "[trace-text] text_set_color slot={} label={} color=0x{color:08X} effect=0x{effect_color:08X}",
                    ordered[0],
                    ordered[1]
                );
                ExtCallOutcome::Value(1)
            }
            29 => {
                self.pop_ext_args(0);
                self.text_state.last_event_time_ms = self.pal_time_ms;
                ExtCallOutcome::Value(1)
            }
            30 => {
                let args = self.pop_ext_args(1);
                self.text_state.mode = args.first().copied().unwrap_or(0);
                ExtCallOutcome::Value(1)
            }
            31 => {
                let args = self.pop_ext_args(4);
                let ordered = ext_args_source_order::<4>(&args);
                self.text_state.initialized = true;
                self.text_state.visible = true;
                self.text_state.rect = [ordered[0], ordered[1], ordered[2], ordered[3]];
                self.text_state.dirty = true;
                log::debug!("[trace-text] text_init_visualnovelmode args={ordered:?}");
                ExtCallOutcome::Value(1)
            }
            32 => {
                let args = self.pop_ext_args(1);
                self.text_state.icon = args.first().copied().unwrap_or(0);
                ExtCallOutcome::Value(1)
            }
            33 => {
                self.pop_ext_args(0);
                self.text_state.last_event_time_ms = self.pal_time_ms;
                ExtCallOutcome::Value(1)
            }
            36..=46 => self.dispatch_tips_ext(index),
            56 => self.ext_bgv_play(assets, nls, resource_manager, audio),
            57 => self.ext_bgv_stop(audio),
            67 => self.ext_bgv_pause(audio),
            49 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(1)
            }
            69 => self.ext_bgv_set_volume(audio),
            50 | 60 | 71 => {
                let args = self.pop_ext_args(1);
                let value = args.first().copied().unwrap_or(100);
                match index {
                    50 | 60 => self.text_state.voice_volume = value,
                    71 => self.text_state.bgv_volume = value,
                    _ => {}
                }
                ExtCallOutcome::Value(1)
            }
            51 | 59 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.text_state.voice_volume)
            }
            53 => {
                let args = self.pop_ext_args(1);
                self.text_state.voice_enabled = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            54 | 61 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.text_state.voice_enabled))
            }
            58 => {
                let args = self.pop_ext_args(1);
                self.text_state.bgv_enabled = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            62 => {
                let args = self.pop_ext_args(1);
                self.text_state.voice_autopan_limit = args.first().copied().unwrap_or(0).wrapping_mul(100);
                ExtCallOutcome::Value(1)
            }
            63 => {
                let args = self.pop_ext_args(1);
                self.text_state.voice_autopan_enabled = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            64 => self.dispatch_voice_ext(16, assets, nls, resource_manager, audio),
            65 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.text_state.voice_autopan_enabled))
            }
            68 => self.ext_bgv_mute(audio),
            72 => {
                let args = self.pop_ext_args(1);
                self.text_state.bgv_muted = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            70 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.text_state.bgv_volume)
            }
            48 | 52 | 66 | 73 | 74 => {
                self.pop_ext_args(match index {
                    48 | 73 => 2,
                    52 => 3,
                    66 => 1,
                    _ => 0,
                });
                ExtCallOutcome::Value(1)
            }
            _ => return ExtCallOutcome::Skip,
        }
    }

    /// Game category 8 button extcalls backed by PAL button/sprite APIs.
    ///
    /// Creation and visual state calls ultimately map to PAL concepts such as
    /// `PalButtonCreate`, `PalButtonCtrl`, `PalButtonGetReaction`, and sprite
    /// rect/visibility updates.  Return value is `1` for successful mutation and
    /// integer button indices for reaction queries. High-index helpers keep
    /// the same stack and state behavior when they are script-visible but do
    /// not create separate PAL button objects.
    fn dispatch_button_ext(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
        input: Option<&PalInputState>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        let preserve_destination = assets.legacy_koikake()
            && matches!(index, 0 | 1 | 3..=8 | 10 | 11 | 13..=16 | 18..=22 | 24 | 26);
        let result = self.dispatch_button_ext_impl(index, assets, nls, resource_manager, sprites, input, audio);
        if index == 12 && (assets.legacy_koikake() || assets.registered_softpal)
            && result == ExtCallOutcome::Value(0) {
            return ExtCallOutcome::Yield { value: Some(0) };
        }
        if preserve_destination && matches!(result, ExtCallOutcome::Value(_)) {
            ExtCallOutcome::Continue
        } else {
            result
        }
    }

    fn dispatch_button_ext_impl(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
        input: Option<&PalInputState>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        match index {
            0 => {
                let result = self.ext_btn_init();
                self.prepare_button_audio(assets, nls, resource_manager, audio);
                return result;
            },
            1 => {
                let result = self.ext_btn_uninit(sprites);
                if let Some(audio) = audio { self.flush_button_audio_releases(audio); }
                return result;
            },
            3 => return self.ext_btn_set(assets, nls, resource_manager, sprites),
            4 => return self.ext_btn_view_ctrl(false, sprites),
            8 => return self.ext_btn_release(index, sprites),
            5 => return self.ext_btn_view_ctrl(true, sprites),
            6 => return self.ext_btn_set_pos(sprites),
            7 => return self.ext_btn_set_rect(sprites),
            9 if assets.legacy_koikake() || assets.registered_softpal => return self.ext_btn_slider_get_native(input, sprites),
            10 if assets.legacy_koikake() || assets.registered_softpal => return self.ext_btn_slider_set_native(sprites),
            9 => return self.ext_btn_slider_get(input, sprites),
            10 => return self.ext_btn_slider_set(sprites),
            11 => return self.ext_btn_slider_begin(input),
            12 => return self.ext_btn_on_check(input, sprites.as_deref()),
            13 => return self.ext_btn_set_toggle(sprites),
            14 => return self.ext_btn_get_pos(sprites.as_deref()),
            15 => return self.ext_btn_enable(sprites),
            16 => return self.ext_btn_set_alpha(sprites),
            17 => return self.ext_btn_get_push(input, sprites.as_deref()),
            18 => return self.ext_btn_expansion(sprites),
            19 => return self.ext_btn_lock(),
            20 => return self.ext_btn_unlock(sprites),
            21 => return self.ext_btn_set_anim(assets, nls, resource_manager, sprites),
            22 => return self.ext_btn_set_hit(),
            23 => return self.ext_btn_get_onmouse(input, sprites.as_deref()),
            24 => return self.ext_btn_anim_clear(sprites),
            25 => return self.ext_btn_get_offmouse(),
            26 => return self.ext_btn_onmouse_clear(),
            29 => return self.ext_btn_set_state(sprites),
            36 => {
                let args = self.pop_ext_args(1);
                let Some(mode) = args.first() else {
                    return ExtCallOutcome::Block;
                };
                self.button_release_mode = *mode != 0;
                return ExtCallOutcome::Value(1);
            }
            37 => return self.ext_btn_get_alpha(sprites.as_deref()),
            38 if assets.registered_softpal => return self.ext_btn_get_reaction(sprites, input, audio),
            39 if assets.registered_softpal => return self.ext_btn_get_toggle(),
            40 if assets.registered_softpal => return self.ext_btn_get_info(sprites.as_deref()),
            42..=48 if assets.registered_softpal => return self.dispatch_button_extensions(index, sprites, input),
            41 => {
                // The table-driven button constructor carries the regular
                // btn_set fields followed by its source rectangle dimensions.
                // All thirteen values belong to this extcall; leaving the
                // lower eleven on the value stack makes the caller restore
                // argument_base from a resource sentinel after title setup.
                let args = self.pop_ext_args(13);
                if args.len() < 13 {
                    return ExtCallOutcome::Value(0);
                }
                let group = args[0];
                let index = args[1];
                let name_value = args[2];
                let callback = args[3];
                log::debug!("[trace-button] btn_register_frame group={group} index={index} resource={name_value} resolved={:?}", self.resolve_resource_string(name_value, assets, nls));
                self.stack.extend_from_slice(&[
                    args[6], args[5], args[4], callback, name_value, index, group,
                ]);
                return self.ext_btn_set(assets, nls, resource_manager, sprites);
            }
            _ => {}
        }
        let arity = match index {
            13 => 2,
            42 => 3,
            43 | 45 | 50 | 52 | 53 | 54 | 57 | 58 => 1,
            44 | 46 => 0,
            47 | 48 | 49 | 51 | 55 | 56 => 2,
            _ => return ExtCallOutcome::Skip,
        };
        self.pop_ext_args(arity);
        ExtCallOutcome::Value(match index {
            43 | 45 | 50 | 52 | 53 | 54 | 57 | 58 => 0,
            _ => 1,
        })
    }

    fn dispatch_window_effect_stub(&mut self, index: u16) -> ExtCallOutcome {
        match index {
            0 => {
                // Game.exe sub_412450 (`effect_stop`) pops stop flags and a
                // fade duration. Native enters per-channel stop states; the
                // portable effect graph has one scene transition plus flash and
                // shake overlays, so finishing the selected channels is the
                // closest script-visible equivalent.
                let args = self.pop_ext_args(2);
                let flags = args.first().copied().unwrap_or(0x1D);
                // koikake.exe 0x411E0F ("effect_stop skip"): while a skip
                // state is active the fade-out duration is forced to zero.
                let duration_ms = if self.dispatch_skip_state() != 0 {
                    0
                } else {
                    args.get(1).copied().unwrap_or(0).max(0)
                };
                self.effect_system.stop_selected(flags);
                log::debug!("[trace-effect] effect_stop flags={flags} duration_ms={duration_ms}");
                ExtCallOutcome::Value(1)
            }
            1 => {
                // Game category 16 index 1 (`sub_4122A0`) is the native
                // screen-shake effect. It pops x amplitude, y amplitude,
                // duration, and a fourth phase/mode value, then records start
                // time and marks the PAL effect pipeline active.
                let args = self.pop_ext_args(4);
                let x_amp = args.first().copied().unwrap_or(0);
                let y_amp = args.get(1).copied().unwrap_or(0);
                let duration = args.get(2).copied().unwrap_or(0).max(0) as u32;
                let phase = args.get(3).copied().unwrap_or(0);
                self.effect_system
                    .set_shake(x_amp, y_amp, duration, phase, self.pal_time_ms);
                log::debug!(
                    "[trace-effect] screen_shake x={x_amp} y={y_amp} duration={duration} phase={phase}"
                );
                ExtCallOutcome::Value(1)
            }
            2 => {
                // Game category 16 index 2 (`sub_4120C0`) creates a
                // full-screen PAL sprite, paints it with the requested RGB
                // color, stores duration/start time, and sets the effect dirty
                // flags. The portable renderer draws an equivalent logical
                // full-screen quad instead of hard-coding the native 1920x1080
                // sprite.
                let args = self.pop_ext_args(4);
                let r = args.first().copied().unwrap_or(0);
                let g = args.get(1).copied().unwrap_or(0);
                let b = args.get(2).copied().unwrap_or(0);
                let duration = args.get(3).copied().unwrap_or(0).max(0) as u32;
                self.effect_system
                    .flash_color(r, g, b, duration, self.pal_time_ms);
                log::debug!("[trace-effect] flash_color r={r} g={g} b={b} duration={duration}");
                ExtCallOutcome::Value(1)
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    /// Game category 10 save/load extcalls.
    ///
    /// `savepoint` arms saving, while each ADV click wait refreshes the
    /// resumable VM and scene before the save menu is opened. `save` writes
    /// that image to `save/save%03d.dat`; `load` restores it. The file prefix
    /// carries the lock dword, title, and thumbnail RGBA `thumbnail_set` reads
    /// at `0x224`. Native `save` pops `(slot, flag)` and still returns 1 when
    /// the slot is out of range or no savepoint is armed.
    fn dispatch_save_stub(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        mut resource_manager: Option<&mut ResourceManager>,
        mut sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        match index {
            0 => {
                let args = self.pop_ext_args(2);
                let slot = args.first().copied().unwrap_or(0);
                let remember = args.get(1).copied().unwrap_or(0);
                self.save_state.last_slot = slot;
                if !self.save_state.armed || self.save_state.checkpoint.is_none() {
                    log::debug!("[trace-save] save slot={slot} skipped, no savepoint");
                    self.save_state.last_result = 1;
                    return ExtCallOutcome::Value(1);
                }
                if !(0..1000).contains(&slot) {
                    log::debug!("[trace-save] save rejected slot={slot}");
                    self.save_state.last_result = 1;
                    return ExtCallOutcome::Value(1);
                }
                if remember != 0 {
                    self.save_state.remembered_slot = slot;
                }
                let mut snapshot = self
                    .resumable_save_snapshot()
                    .expect("armed savepoint has a checkpoint");
                // Keep the pre-menu VM and scene, while taking the current
                // title and thumbnail from the actual save operation.
                {
                    let (body, name) = self.adv_text_parts_for_render(assets, nls);
                    let title = if name.is_empty() {
                        body
                    } else if body.is_empty() {
                        name
                    } else {
                        format!("{name} {body}")
                    };
                    let (title, _) = parse_pal_text_directives(&title);
                    if !title.is_empty() {
                        snapshot.title_bytes = title.into_bytes();
                    }
                }
                if let Some(sprites_ref) = sprites.as_deref() {
                    // The thumbnail was captured when the menu opened
                    // (`thumbnail_renew`), before the menu covered the scene.
                    // Re-capturing here would snapshot the menu itself. Only
                    // fall back to a fresh capture when nothing was stored.
                    if self.save_state.thumbnail_pixels.is_empty() {
                        self.capture_thumbnail(sprites_ref);
                    }
                    snapshot.thumb_width = self.save_state.thumbnail_size[0];
                    snapshot.thumb_height = self.save_state.thumbnail_size[1];
                    snapshot.thumb_pixels = self.save_state.thumbnail_pixels.clone();
                }
                self.save_state.snapshots.insert(slot, snapshot.clone());
                if let Some(manager) = resource_manager.as_deref() {
                    match self.write_original_save(manager.root(), slot, &snapshot, nls) {
                        Ok(path) => log::debug!(
                            "[trace-save] save slot={slot} pc=0x{:08X} file={}",
                            snapshot.pc,
                            path.display()
                        ),
                        Err(err) => {
                            log::warn!("[trace-save] save slot={slot} file failed: {err}")
                        }
                    }
                }
                self.save_state.last_result = 1;
                ExtCallOutcome::Value(1)
            }
            1 => {
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(0);
                self.save_state.last_slot = slot;
                let snapshot = resource_manager
                    .as_deref()
                    .and_then(|manager| read_runtime_save_snapshot(manager.root(), slot).ok())
                    .or_else(|| self.save_state.snapshots.get(&slot).cloned());
                if let Some(snapshot) = snapshot {
                    let scene = snapshot.clone();
                    let resume_wait_click = snapshot.resume_wait_click;
                    self.restore_save_snapshot(snapshot);
                    if let Some(sprites) = sprites.as_deref_mut() {
                        self.restore_checkpoint_scene(&scene, sprites);
                    }
                    self.wait_task_handle = None;
                    self.wait_task_kind = None;
                    self.save_state.last_result = 1;
                    log::debug!(
                        "[trace-save] load slot={slot} pc=0x{:08X} sprites={}",
                        self.pc,
                        scene.sprites.len()
                    );
                    return if resume_wait_click {
                        ExtCallOutcome::Wait {
                            value: 1,
                            request: WaitRequest::Click,
                        }
                    } else {
                        ExtCallOutcome::Value(1)
                    };
                }
                // Exact audited Koikake saves restore native scene records.
                // Other profiles retain the historical header-only fallback.
                if let Some(snapshot) = resource_manager
                    .as_deref_mut()
                    .and_then(|manager| self.import_original_save_with_resources(manager, slot, assets))
                {
                    let resume_pc = snapshot.pc;
                    let resume_wait_click = snapshot.resume_wait_click;
                    self.save_state.snapshots.insert(slot, snapshot.clone());
                    let scene = snapshot.clone();
                    self.restore_save_snapshot(snapshot);
                    if let Some(sprites) = sprites.as_deref_mut() {
                        self.restore_checkpoint_scene(&scene, sprites);
                    }
                    self.wait_task_handle = None;
                    self.wait_task_kind = None;
                    self.save_state.last_result = 1;
                    log::debug!(
                        "[trace-save] load slot={slot} imported original save, resume pc=0x{resume_pc:08X}"
                    );
                    return if resume_wait_click {
                        ExtCallOutcome::Wait { value: 1, request: WaitRequest::Click }
                    } else { ExtCallOutcome::Value(1) };
                }
                let has_file = resource_manager
                    .as_ref()
                    .and_then(|manager| {
                        find_loose_save_file(manager.root(), &original_save_filename(slot))
                    })
                    .is_some();
                self.save_state.last_result = 0;
                if has_file {
                    log::warn!(
                        "[trace-save] load slot={slot} file present but has no resumable snapshot"
                    );
                }
                log::debug!("[trace-save] load slot={slot} file={has_file} snapshot=false");
                ExtCallOutcome::Value(0)
            }
            2 => {
                let args = self.pop_ext_args(1);
                let title = args.first().copied().unwrap_or(0);
                self.save_state.title = title;
                if let Some(text) = self.resolve_resource_string(title, assets, nls) {
                    self.save_state.title_bytes = text.into_bytes();
                }
                log::debug!("[trace-save] save_set_title value={title}");
                ExtCallOutcome::Value(1)
            }
            5 => self.ext_thumbnail_set(resource_manager, sprites),
            12 => {
                let args = self.pop_ext_args(1);
                let enabled = args.first().copied().unwrap_or(0);
                self.save_state.mosaic_enabled = enabled != 0;
                log::debug!("[trace-save] save_thumbnail_mosaic_set enabled={enabled}");
                ExtCallOutcome::Value(1)
            }
            22 => {
                if let Some(sprites) = sprites {
                    self.capture_thumbnail(sprites);
                }
                log::debug!(
                    "[trace-save] thumbnail_renew capture={} mosaic={}",
                    self.save_state.capture_from_screen,
                    self.save_state.mosaic_enabled
                );
                ExtCallOutcome::Value(1)
            }
            30 => self.ext_copy_save_file(resource_manager),
            31 => self.ext_load_thumbnail(assets, nls, resource_manager, sprites),
            14 => self.ext_save_day_draw(resource_manager, sprites),
            16 => self.ext_save_text_draw(resource_manager, sprites),
            17 => {
                log::debug!(
                    "[trace-save] get_new_savefile slot={}",
                    self.save_state.remembered_slot
                );
                ExtCallOutcome::Value(self.save_state.remembered_slot)
            }
            28 => self.ext_delete_save_file(resource_manager),
            35 => {
                self.save_state.armed = false;
                self.save_state.checkpoint = None;
                self.save_state.resume_checkpoint = None;
                log::debug!("[trace-save] save_point_clear");
                ExtCallOutcome::Value(1)
            }
            3 | 6 | 21 | 29 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(1)
            }
            4 => {
                let args = self.pop_ext_args(2);
                self.save_state.thumbnail_size = [
                    args.first().copied().unwrap_or(0),
                    args.get(1).copied().unwrap_or(0),
                ];
                ExtCallOutcome::Value(1)
            }
            7 => {
                let args = self.pop_ext_args(1);
                self.save_state.font_size = args.first().copied().unwrap_or(0);
                ExtCallOutcome::Value(1)
            }
            8 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(0)
            }
            13 => self.ext_save_time_draw(resource_manager, sprites),
            9 => {
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(0);
                self.save_state.last_slot = slot;
                let has_save = if slot == -1 {
                    // Native is_save(-1) (Game.sqlite sub_432620) scans slots
                    // 0..=999 and reports whether any save%03d.dat exists; it
                    // never looks at continue.dat.
                    let has_snapshot = self.save_state.snapshots.keys().any(|key| (0..=999).contains(key));
                    has_snapshot
                        || resource_manager
                            .as_ref()
                            .is_some_and(|manager| (0..=999).any(|candidate| {
                                portable_save_path(manager.root(), candidate).is_file()
                                    || find_loose_save_file(
                                        manager.root(),
                                        &original_save_filename(candidate),
                                    )
                                    .is_some()
                            }))
                } else {
                    let has_portable_snapshot = self.save_state.snapshots.contains_key(&slot);
                    let has_loose_file = resource_manager
                        .as_ref()
                        .and_then(|manager| {
                            portable_save_path(manager.root(), slot)
                                .is_file()
                                .then_some(portable_save_path(manager.root(), slot))
                                .or_else(|| {
                                    find_loose_save_file(
                                        manager.root(),
                                        &original_save_filename(slot),
                                    )
                                })
                        })
                        .is_some();
                    has_portable_snapshot || has_loose_file
                };
                self.save_state.last_result = i32::from(has_save);
                log::debug!(
                    "[trace-save] is_save slot={slot} -> {}",
                    self.save_state.last_result
                );
                ExtCallOutcome::Value(self.save_state.last_result)
            }
            10 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(0)
            }
            15 => {
                let args = self.pop_ext_args(2);
                self.save_state.text_rect = [
                    args.first().copied().unwrap_or(0),
                    args.get(1).copied().unwrap_or(0),
                    self.save_state.text_rect[2],
                    self.save_state.text_rect[3],
                ];
                ExtCallOutcome::Value(1)
            }
            11 => {
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(0);
                self.save_state.last_slot = slot;
                if slot == SAVEPOINT_CLEAR_SLOT {
                    self.save_state.armed = false;
                    self.save_state.checkpoint = None;
                    self.save_state.resume_checkpoint = None;
                    log::debug!("[trace-save] savepoint clear");
                    return ExtCallOutcome::Value(1);
                }
                if self.save_state.locked {
                    log::debug!("[trace-save] savepoint locked, keeping previous image");
                    self.save_state.last_result = 1;
                    return ExtCallOutcome::Value(1);
                }
                self.capture_save_checkpoint(assets, nls, sprites.as_deref());
                self.save_state.last_result = 1;
                log::debug!(
                    "[trace-save] savepoint pc=0x{:08X} sprites={}",
                    self.save_state
                        .checkpoint
                        .as_ref()
                        .map(|snapshot| snapshot.pc)
                        .unwrap_or(0),
                    self.save_state
                        .checkpoint
                        .as_ref()
                        .map(|snapshot| snapshot.sprites.len())
                        .unwrap_or(0)
                );
                ExtCallOutcome::Value(1)
            }
            32 => self.ext_save_lock(resource_manager),
            23 => {
                let args = self.pop_ext_args(1);
                self.save_state.font_type = args.first().copied().unwrap_or(0);
                ExtCallOutcome::Value(1)
            }
            24 => {
                // `set_load_after_process(point)` registers the script point the
                // native VM jumps to after a successful `load`.
                let args = self.pop_ext_args(1);
                self.save_state.load_after_point = args.first().copied();
                log::debug!(
                    "[trace-save] set_load_after_process point={:?}",
                    self.save_state.load_after_point
                );
                ExtCallOutcome::Value(1)
            }
            25 => {
                self.pop_ext_args(0);
                if let Some(manager) = resource_manager.as_deref() {
                    if let Err(err) = self.write_portable_system_data(manager.root()) {
                        log::warn!("[trace-save] savesystemdata failed: {err}");
                        return ExtCallOutcome::Value(0);
                    }
                    // The native engine also persists its global system state
                    // (system.dat) here; sena-rs writes the system_mem bank that
                    // holds the script-side global settings.
                    if let Err(err) = self.write_portable_system_mem(manager.root()) {
                        log::warn!("[trace-save] savesystemdata system_mem failed: {err}");
                        return ExtCallOutcome::Value(0);
                    }
                    // The read-text bitmap is part of the native system.dat
                    // state as well (koikake.exe 0x438F00).
                    if let Err(err) = self.write_portable_read_lines(manager.root()) {
                        log::warn!("[trace-save] savesystemdata read lines failed: {err}");
                        return ExtCallOutcome::Value(0);
                    }
                }
                ExtCallOutcome::Value(1)
            }
            26 => {
                let args = self.pop_ext_args(1);
                self.save_state.font_effect = args.first().copied().unwrap_or(0);
                ExtCallOutcome::Value(1)
            }
            27 => {
                let args = self.pop_ext_args(2);
                self.save_state.font_color = args.first().copied().unwrap_or(0);
                ExtCallOutcome::Value(1)
            }
            33 => self.ext_is_save_lock(resource_manager),
            34 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.save_state.last_result != 0))
            }
            36 => {
                let args = self.pop_ext_args(1);
                self.save_state.locked = args.first().copied().unwrap_or(0) != 0;
                log::debug!(
                    "[trace-save] save_point_lock locked={}",
                    self.save_state.locked
                );
                ExtCallOutcome::Value(1)
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    fn reset_adv(&mut self) {
        self.text_state.visible = false;
        self.text_state.show_wait_mark = false;
        self.text_state.reveal_enabled = false;
        self.text_state.reveal_duration_ms = 0;
        self.text_state.last_text_value = 0;
        self.text_state.last_text_args = [0; 4];
        self.text_state.pending_alpha.clear();
        self.text_state.dirty = true;
    }

    fn thumbnail_size_or_default(&self) -> (i32, i32) {
        let width = if self.save_state.thumbnail_size[0] > 0 {
            self.save_state.thumbnail_size[0]
        } else {
            DEFAULT_THUMB_WIDTH
        };
        let height = if self.save_state.thumbnail_size[1] > 0 {
            self.save_state.thumbnail_size[1]
        } else {
            DEFAULT_THUMB_HEIGHT
        };
        (width, height)
    }

    fn capture_thumbnail(&mut self, sprites: &SpriteSystem) {
        let (width, height) = self.thumbnail_size_or_default();
        let (logical_width, logical_height) = self.logical_size();
        let samples = self.thumbnail_sprite_samples(sprites);
        let factor = self.save_state.mosaic_enabled.then_some(MOSAIC_FACTOR);
        self.save_state.thumbnail_pixels = composite_thumbnail(
            &samples,
            logical_width,
            logical_height,
            width as u32,
            height as u32,
            factor,
        );
        self.save_state.thumbnail_size = [width, height];
    }

    fn thumbnail_sprite_samples(&self, sprites: &SpriteSystem) -> Vec<ThumbnailSprite> {
        let mut samples = Vec::new();
        for handle in self.game_sprites.values().copied() {
            let Some(sprite) = sprites.get(handle) else {
                continue;
            };
            if !sprite.visible {
                continue;
            }
            let Some(surface) = sprites.surface(sprite.surface) else {
                continue;
            };
            let texture = surface.to_scene_texture();
            let width = texture.width.max(1);
            let height = texture.height.max(1);
            let expected = width as usize * height as usize * 4;
            if texture.pixels.len() < expected {
                continue;
            }
            samples.push(ThumbnailSprite {
                x: sprite.position.x as i32,
                y: sprite.position.y as i32,
                width,
                height,
                rgba: texture.pixels[..expected].to_vec(),
            });
        }
        samples
    }

    fn current_save_prefix(&self, slot: i32) -> OriginalSavePrefix {
        let (width, height) = self.thumbnail_size_or_default();
        let mut pixels = self.save_state.thumbnail_pixels.clone();
        let expected = width as usize * height as usize * 4;
        if pixels.len() != expected {
            pixels.resize(expected, 0);
        }
        if self.save_state.mosaic_enabled {
            pixels = mosaic_rgba(&pixels, width as u32, height as u32, MOSAIC_FACTOR);
        }
        OriginalSavePrefix {
            lock: self.save_state.locks.get(&slot).copied().unwrap_or(0),
            title: self.save_state.title_bytes.clone(),
            mosaic: i32::from(self.save_state.mosaic_enabled),
            resume_pc: self.text_state.last_text_pc as i32,
            secondary_pc: -1,
            thumb_width: width,
            thumb_height: height,
            pixels,
        }
    }

    pub(crate) fn save_host_checkpoint(&self, root: &Path, slot: i32,
        sprites: &SpriteSystem, nls: Nls) -> anyhow::Result<PathBuf> {
        anyhow::ensure!((0..1000).contains(&slot), "invalid checkpoint slot");
        let mut snapshot = self.capture_resumable_scene(Some(sprites));
        snapshot.host_checkpoint = Some(self.capture_host_state()?);
        snapshot.sprites = self.game_sprites.iter().filter_map(|(&slot, &handle)|
            saved_sprite_with_limit(sprites, slot, handle, CHECKPOINT_SPRITE_BYTES_CAP)).collect();
        for (&slot,&handle) in &self.game_sprite_transition_sources {
            if let Some(mut sprite)=saved_sprite_with_limit(sprites,slot,handle,CHECKPOINT_SPRITE_BYTES_CAP) {
                sprite.transition_source=true;snapshot.sprites.push(sprite);
            }
        }
        eprintln!("[checkpoint-save] live_pc={:#x} worker_pc={:?} sprites={} raw_bytes={}",self.pc,
            self.thread_state.target_pc,snapshot.sprites.len(),snapshot.sprites.iter().map(|s|s.rgba.len()).sum::<usize>());
        Ok(self.write_original_save(root, slot, &snapshot, nls)?)
    }

    pub(crate) fn load_host_checkpoint(&mut self, root: &Path, slot: i32,
        sprites: &mut SpriteSystem) -> anyhow::Result<(Option<WaitRequest>,u32)> {
        let snapshot = read_runtime_save_snapshot(root, slot)?;
        let state=snapshot.host_checkpoint.clone().ok_or_else(||anyhow::anyhow!("not a host diagnostic checkpoint"))?;
        self.restore_save_snapshot(snapshot.clone());
        self.restore_checkpoint_scene(&snapshot, sprites);
        self.restore_host_state(&state);
        self.wait_task_handle=None;self.wait_task_kind=None;
        Ok((state.wait,state.clock))
    }

    fn write_original_save(
        &self,
        root: &Path,
        slot: i32,
        snapshot: &RuntimeSaveSnapshot,
        nls: Nls,
    ) -> std::io::Result<PathBuf> {
        let mut prefix = self.current_save_prefix(slot);
        if !snapshot.title_bytes.is_empty() {
            prefix.title = std::str::from_utf8(&snapshot.title_bytes)
                .ok()
                .and_then(|title| nls.encode(title).ok())
                .unwrap_or_else(|| snapshot.title_bytes.clone());
        }
        if snapshot.thumb_width > 0
            && snapshot.thumb_height > 0
            && !snapshot.thumb_pixels.is_empty()
        {
            prefix.thumb_width = snapshot.thumb_width;
            prefix.thumb_height = snapshot.thumb_height;
            prefix.pixels = snapshot.thumb_pixels.clone();
        }
        let snapshot_bytes = encode_runtime_save_snapshot(snapshot)?;
        let bytes = encode_original_save(&prefix, &snapshot_bytes);
        let path = original_save_path(root, slot);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes)?;
        Ok(path)
    }

    /// `thumbnail_set` and the save text draw calls blit into the target
    /// sprite slot's canvas surface in native PAL, so the pixels inherit that
    /// canvas layer: the slot plates stay underneath and a later popup lane
    /// (cursor moved down before `btn_set`) covers them.  These drawings are
    /// separate sprites in this renderer, so they must copy the canvas
    /// sprite's depth instead of floating on a fixed top band.
    fn save_drawing_priority(&self, sprites: &SpriteSystem, slot: i32) -> i32 {
        self.game_sprites
            .get(&slot)
            .and_then(|handle| sprites.get(*handle))
            .map(|sprite| sprite.effective_priority())
            .unwrap_or_else(|| game_sprite_priority(slot, self.sprite_priority_cursor))
    }

    /// `thumbnail_set` pops `(slot, save_slot, x, y)` and blits the stored
    /// thumbnail RGBA onto that sprite. Missing files still return 1.
    fn ext_thumbnail_set(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let save_slot = args[1];
        let x = args[2];
        let y = args[3];
        let prefix = resource_manager
            .as_deref()
            .and_then(|manager| read_original_save_prefix(manager.root(), save_slot).ok())
            .or_else(|| {
                (self.save_state.last_slot == save_slot
                    && !self.save_state.thumbnail_pixels.is_empty())
                .then(|| self.current_save_prefix(save_slot))
            });
        let Some(prefix) = prefix else {
            log::debug!("[trace-save] thumbnail_set slot={slot} save_slot={save_slot} missing");
            return ExtCallOutcome::Value(1);
        };
        let Some(sprites) = sprites else {
            return ExtCallOutcome::Value(1);
        };
        if let Some(old) = self.save_state.thumbnail_sprites.remove(&(slot, x, y)) {
            sprites.release(old);
        }
        let width = prefix.thumb_width.max(1) as u32;
        let height = prefix.thumb_height.max(1) as u32;
        let mut pixels = prefix.pixels;
        let expected = width as usize * height as usize * 4;
        if pixels.len() != expected {
            pixels.resize(expected, 0);
        }
        let priority = self.save_drawing_priority(sprites, slot);
        let Some(handle) = sprites.create_rgba_sprite(
            width,
            height,
            pixels,
            PalVec3::from_f32(x as f32, y as f32, 0.0),
            priority,
            format!("thumbnail:{save_slot}"),
        ) else {
            return ExtCallOutcome::Value(0);
        };
        self.save_state
            .thumbnail_sprites
            .insert((slot, x, y), handle);
        log::debug!(
            "[trace-save] thumbnail_set slot={slot} save_slot={save_slot} pos=({x},{y}) size={width}x{height}"
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_copy_save_file(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let dest = args[0];
        let src = args[1];
        let Some(manager) = resource_manager else {
            return ExtCallOutcome::Value(0);
        };
        let src_path = find_loose_save_file(manager.root(), &original_save_filename(src))
            .unwrap_or_else(|| original_save_path(manager.root(), src));
        let dest_path = original_save_path(manager.root(), dest);
        let copied = if src_path.is_file() {
            if let Some(parent) = dest_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::copy(&src_path, &dest_path).is_ok()
        } else {
            false
        };
        if copied {
            if let Some(snapshot) = self.save_state.snapshots.get(&src).cloned() {
                self.save_state.snapshots.insert(dest, snapshot);
            }
            if let Some(lock) = self.save_state.locks.get(&src).copied() {
                self.save_state.locks.insert(dest, lock);
            }
        }
        log::debug!("[trace-save] copy_file dest={dest} src={src} copied={copied}");
        ExtCallOutcome::Value(i32::from(copied))
    }

    fn ext_load_thumbnail(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        _sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let value = args.first().copied().unwrap_or(0);
        if value == LOAD_THUMBNAIL_CAPTURE_SENTINEL {
            self.save_state.capture_from_screen = true;
            log::debug!("[trace-save] load_thumbnail capture enabled");
            return ExtCallOutcome::Value(1);
        }
        self.save_state.capture_from_screen = false;
        let Some(manager) = resource_manager else {
            return ExtCallOutcome::Value(1);
        };
        let Some(name) = self.resolve_resource_string(value, assets, nls) else {
            return ExtCallOutcome::Value(1);
        };
        let asset = match open_resource_variant(manager, &name, IMAGE_EXTENSIONS) {
            Ok(asset) => asset,
            Err(err) => {
                log::warn!("[trace-save] load_thumbnail name={name:?} open failed: {err}");
                return ExtCallOutcome::Value(1);
            }
        };
        let decoded = match decode_asset_image(manager, &asset) {
            Ok(decoded) => decoded,
            Err(err) => {
                log::warn!("[trace-save] load_thumbnail name={name:?} decode failed: {err}");
                return ExtCallOutcome::Value(1);
            }
        };
        let (width, height) = self.thumbnail_size_or_default();
        self.save_state.thumbnail_pixels = composite_thumbnail(
            &[ThumbnailSprite {
                x: 0,
                y: 0,
                width: decoded.width,
                height: decoded.height,
                rgba: decoded.rgba,
            }],
            decoded.width.max(1),
            decoded.height.max(1),
            width as u32,
            height as u32,
            self.save_state.mosaic_enabled.then_some(MOSAIC_FACTOR),
        );
        self.save_state.thumbnail_size = [width, height];
        log::debug!("[trace-save] load_thumbnail name={name:?} size={width}x{height}");
        ExtCallOutcome::Value(1)
    }

    /// `save_lock` pops `(slot, value)` and writes that dword at offset 0.
    /// A missing file still returns 1, matching the native open-error path.
    fn ext_save_lock(&mut self, resource_manager: Option<&mut ResourceManager>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let value = args[1];
        self.save_state.locks.insert(slot, value);
        self.save_state.last_slot = slot;
        let Some(manager) = resource_manager else {
            return ExtCallOutcome::Value(1);
        };
        let path = find_loose_save_file(manager.root(), &original_save_filename(slot))
            .unwrap_or_else(|| original_save_path(manager.root(), slot));
        if !path.is_file() {
            log::debug!("[trace-save] save_lock slot={slot} missing");
            return ExtCallOutcome::Value(1);
        }
        if let Err(err) = write_save_lock_dword(&path, value) {
            log::warn!("[trace-save] save_lock slot={slot} write failed: {err}");
        }
        log::debug!("[trace-save] save_lock slot={slot} value={value}");
        ExtCallOutcome::Value(1)
    }

    /// `is_save_lock` pops a slot and returns the file's first dword.
    fn ext_is_save_lock(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let slot = args.first().copied().unwrap_or(0);
        self.save_state.last_slot = slot;
        let from_file = resource_manager.as_ref().and_then(|manager| {
            let path = find_loose_save_file(manager.root(), &original_save_filename(slot))?;
            let mut file = File::open(path).ok()?;
            let mut header = [0_u8; 8];
            file.read_exact(&mut header).ok()?;
            Some(read_lock_dword(&header))
        });
        let value = from_file
            .or_else(|| self.save_state.locks.get(&slot).copied())
            .unwrap_or(0);
        log::debug!("[trace-save] is_save_lock slot={slot} value={value}");
        ExtCallOutcome::Value(value)
    }

    /// Game category 10 index 13 (`sub_431C70`, "AdvCommandSaveTimeDraw").
    ///
    /// Native pop order is `(sprite_slot, save_slot, x, y, format_mode)`.  The
    /// handler builds `continue.dat` for save_slot -1 or `save%03d.dat`,
    /// reads the file modification time with Win32 `CreateFile/GetFileTime`,
    /// and draws the formatted HH:MM[:SS] text through
    /// `PalSpriteCreateText{Ex}`.  Missing files still return 1 and simply do
    /// not create/update the text sprite.
    ///
    /// The portable runtime formats file modification time from `SystemTime`
    /// using a stable HH:MM[:SS] representation so save/load pages can show a
    /// deterministic timestamp on every supported platform.
    fn ext_save_time_draw(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let sprite_slot = args[0];
        let save_slot = args[1];
        let x = args[2];
        let y = args[3];
        let format_mode = args[4];
        let filename = if save_slot == -1 {
            "continue.dat".to_owned()
        } else {
            format!("save{:03}.dat", save_slot)
        };
        let Some(manager) = resource_manager else {
            log::debug!("[trace-save] savetimedraw filename={filename:?} no resource manager");
            return ExtCallOutcome::Value(1);
        };
        let Some(path) = find_loose_save_file(manager.root(), &filename) else {
            log::debug!(
                "[trace-save] savetimedraw slot={sprite_slot} save_slot={save_slot} filename={filename:?} missing"
            );
            return ExtCallOutcome::Value(1);
        };
        let Ok(metadata) = std::fs::metadata(&path) else {
            log::debug!(
                "[trace-save] savetimedraw slot={sprite_slot} path={} metadata failed",
                path.display()
            );
            return ExtCallOutcome::Value(1);
        };
        let Ok(modified) = metadata.modified() else {
            return ExtCallOutcome::Value(1);
        };
        let text = format_save_time(modified, format_mode);
        let Some(sprites) = sprites else {
            return ExtCallOutcome::Value(1);
        };
        let saved_size = self.font_state.font_size();
        self.font_state
            .set_font_size(self.save_state.font_size.max(18) as u16);
        let (width, height, rgba) = self.font_state.rasterize(&text);
        self.font_state.set_font_size(saved_size);
        let priority = self.save_drawing_priority(sprites, sprite_slot);
        if let Some(handle) = self
            .save_state
            .text_sprites
            .get(&(sprite_slot, x, y))
            .copied()
        {
            let _ = sprites.replace_sprite_surface(
                handle,
                width,
                height,
                rgba,
                format!("save-time:{filename}:{text}"),
            );
            let _ = sprites.set_pos(handle, x, y, 0);
            let _ = sprites.set_priority(handle, priority);
            let _ = sprites.view_ctrl(handle, true);
        } else if let Some(handle) = sprites.create_rgba_sprite(
            width,
            height,
            rgba,
            PalVec3::new(x, y, 0),
            priority,
            format!("save-time:{filename}:{text}"),
        ) {
            self.save_state
                .text_sprites
                .insert((sprite_slot, x, y), handle);
        }
        log::debug!(
            "[trace-save] savetimedraw slot={sprite_slot} save_slot={save_slot} path={} text={text:?} pos=({x},{y}) mode={format_mode}",
            path.display()
        );
        ExtCallOutcome::Value(1)
    }

    /// `savedaydraw` pops `(sprite, save_slot, x, y, format)` and paints the
    /// file date. A missing file still returns 1.
    fn ext_save_day_draw(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let text = save_file_modified(resource_manager.as_deref(), args[1])
            .map(|modified| format_save_day(modified, args[4]))
            .unwrap_or_default();
        if text.is_empty() {
            return ExtCallOutcome::Value(1);
        }
        self.place_save_label(sprites, args[0], args[2], args[3], &text, "save-day");
        log::debug!(
            "[trace-save] savedaydraw slot={} save_slot={} text={text:?} pos=({},{})",
            args[0],
            args[1],
            args[2],
            args[3]
        );
        ExtCallOutcome::Value(1)
    }

    /// `savetextdraw` pops `(sprite, save_slot, x, y)` and paints the title
    /// stored at offset 8. A missing file still returns 1.
    fn ext_save_text_draw(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let title = resource_manager
            .as_deref()
            .and_then(|manager| {
                read_original_save_prefix(manager.root(), args[1])
                    .ok()
                    .map(|prefix| decode_save_title(&prefix.title, manager.nls()))
            })
            .unwrap_or_default();
        if title.is_empty() {
            return ExtCallOutcome::Value(1);
        }
        self.place_save_label(sprites, args[0], args[2], args[3], &title, "save-text");
        log::debug!(
            "[trace-save] savetextdraw slot={} save_slot={} text={title:?} pos=({},{})",
            args[0],
            args[1],
            args[2],
            args[3]
        );
        ExtCallOutcome::Value(1)
    }

    fn place_save_label(
        &mut self,
        sprites: Option<&mut SpriteSystem>,
        sprite_slot: i32,
        x: i32,
        y: i32,
        text: &str,
        label: &str,
    ) {
        let Some(sprites) = sprites else {
            return;
        };
        let saved_size = self.font_state.font_size();
        self.font_state
            .set_font_size(self.save_state.font_size.max(16) as u16);
        let (width, height, rgba) = self.font_state.rasterize(text);
        self.font_state.set_font_size(saved_size);
        let priority = self.save_drawing_priority(sprites, sprite_slot);
        if let Some(handle) = self
            .save_state
            .text_sprites
            .get(&(sprite_slot, x, y))
            .copied()
        {
            let _ = sprites.replace_sprite_surface(
                handle,
                width,
                height,
                rgba,
                format!("{label}:{text}"),
            );
            let _ = sprites.set_pos(handle, x, y, 0);
            let _ = sprites.set_priority(handle, priority);
            let _ = sprites.view_ctrl(handle, true);
        } else if let Some(handle) = sprites.create_rgba_sprite(
            width,
            height,
            rgba,
            PalVec3::new(x, y, 0),
            priority,
            format!("{label}:{text}"),
        ) {
            self.save_state
                .text_sprites
                .insert((sprite_slot, x, y), handle);
        }
    }

    fn clear_save_drawings(&mut self, sprites: &mut SpriteSystem, slot: i32) {
        self.save_state
            .text_sprites
            .retain(|(target, _, _), handle| {
                if slot == -1 || *target == slot {
                    sprites.release(*handle);
                    false
                } else {
                    true
                }
            });
        self.save_state
            .thumbnail_sprites
            .retain(|(target, _, _), handle| {
                if slot == -1 || *target == slot {
                    sprites.release(*handle);
                    false
                } else {
                    true
                }
            });
    }

    /// `delete_file` pops a slot. `-1` removes `continue.dat`. The destination
    /// receives 1 only when a file was actually removed.
    fn ext_delete_save_file(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let slot = args.first().copied().unwrap_or(0);
        let Some(manager) = resource_manager else {
            return ExtCallOutcome::Value(0);
        };
        let path = find_loose_save_file(manager.root(), &original_save_filename(slot))
            .unwrap_or_else(|| original_save_path(manager.root(), slot));
        let deleted = path.is_file() && std::fs::remove_file(&path).is_ok();
        if deleted {
            self.save_state.snapshots.remove(&slot);
        }
        // Native sub_431130 clears the marker after DeleteFileA even when
        // the file has already disappeared and deletion reports failure.
        if self.save_state.remembered_slot == slot {
            self.save_state.remembered_slot = -1;
        }
        log::debug!(
            "[trace-save] delete_file slot={slot} path={} deleted={deleted}",
            path.display()
        );
        ExtCallOutcome::Value(i32::from(deleted))
    }

    fn dispatch_system_button_stub(&mut self, index: u16) -> ExtCallOutcome {
        // Category 12 stores script point ids and a raw KeyOnEx repeat mask.
        // Enable gates each slot; 0xFFFF addresses every registered slot.
        match index {
            0 => {
                let args = self.pop_ext_args(3);
                if args.len() < 3 {
                    return ExtCallOutcome::Block;
                }
                let slot = args[0];
                let image = args[1];
                let state = args[2];
                self.system_buttons.insert(
                    slot,
                    GameSystemButtonEntry {
                        image,
                        state,
                        // system_btn_set initializes the enable flag to 1;
                        // system_btn_enable rewrites it afterwards.
                        enabled: true,
                    },
                );
                log::debug!("[trace-system-button] set slot={slot} image={image} state={state}");
                ExtCallOutcome::Value(1)
            }
            1 => {
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(0xFFFF);
                if slot == 0xFFFF {
                    self.system_buttons.clear();
                } else {
                    self.system_buttons.remove(&slot);
                }
                log::debug!("[trace-system-button] release slot={slot}");
                ExtCallOutcome::Value(1)
            }
            2 => {
                let args = self.pop_ext_args(2);
                if args.len() < 2 {
                    return ExtCallOutcome::Block;
                }
                let slot = args[0];
                let enabled = args[1] != 0;
                let mut affected = 0usize;
                if slot == 0xFFFF {
                    for entry in self.system_buttons.values_mut() {
                        let _ = (entry.image, entry.state);
                        entry.enabled = enabled;
                        affected += 1;
                    }
                } else {
                    let entry = self.system_buttons.entry(slot).or_default();
                    let _ = (entry.image, entry.state);
                    entry.enabled = enabled;
                    affected = 1;
                }
                log::debug!(
                    "[trace-system-button] enable slot={slot} enabled={enabled} affected={affected}"
                );
                ExtCallOutcome::Value(1)
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    /// Game category 14 history/backlog extcalls.
    ///
    /// The original handlers mutate backlog layout, active state, records, and
    /// query height/text data.  This implementation records enough state for
    /// script control flow and UI sizing, returning integer status or record
    /// counts. Text wrapping and scroll geometry are implemented in the
    /// portable renderer using the recovered history layout state.
    fn dispatch_history_stub(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        mut audio: Option<&mut AudioSystem>,
        input: Option<&PalInputState>,
    ) -> ExtCallOutcome {
        if assets.registered_softpal && matches!(index, 13..=19 | 21..=24) {
            return self.dispatch_history_icons(index, assets, nls, resource_manager, audio);
        }
        match index {
            0 => {
                let args = self.pop_ext_args(9);
                if args.len() < 9 {
                    return ExtCallOutcome::Block;
                }
                // Native history_init clears the complete record array.
                self.history_state.records.clear();
                self.history_state.texts.clear();
                self.history_state.icons.records.clear();
                self.history_state.records_generation += 1;
                self.history_state.layout_cache = None;
                self.history_state.hovered_record = None;
                self.history_state.initialized = true;
                self.history_state.layout = [
                    args[0], args[1], args[2], args[3], args[4], args[5], args[6],
                ];
                self.history_state.colors = [
                    args[7] | 0xFF00_0000u32 as i32,
                    args.get(8).copied().unwrap_or(0) | 0xFF00_0000u32 as i32,
                ];
                let (logical_width, logical_height) = self.logical_size();
                self.history_state.rect = [0, 0, logical_width as i32, logical_height as i32];
                log::debug!(
                    "[trace-history] history_init layout={:?} colors=[0x{:08X},0x{:08X}]",
                    self.history_state.layout,
                    self.history_state.colors[0],
                    self.history_state.colors[1]
                );
                ExtCallOutcome::Value(1)
            }
            1 => {
                self.pop_ext_args(0);
                self.history_state.active = true;
                // Native history_begin lays the records out and then calls
                // apply_scroll(ctx, 0): the view opens on the newest records
                // (scroll position 0).
                self.history_state.scroll_y = 0;
                self.history_state.hovered_record = None;
                self.rebuild_history_layout(assets, nls);
                log::debug!(
                    "[trace-history] history_begin records={} height={}",
                    self.history_state.records.len(),
                    self.history_state.height
                );
                ExtCallOutcome::Value(1)
            }
            2 => {
                self.pop_ext_args(0);
                self.history_state.active = false;
                log::debug!("[trace-history] history_end");
                ExtCallOutcome::Value(1)
            }
            3 => {
                // Native VmExtcall_HistScroll returns ctx+0x5D9C, the history
                // record count (koikake's log screen treats 0 as "nothing to
                // show" and aborts).
                self.pop_ext_args(0);
                let count = self.history_state.records.len() as i32;
                log::debug!("[trace-history] history_scroll records={count}");
                ExtCallOutcome::Value(count)
            }
            4 => {
                // Native history_set_pos (0x41F1A0/0x41F800) clamps the new
                // position to the record count and re-lays the view out from
                // that record upward; the unit is records, not pixels.
                let args = self.pop_ext_args(1);
                let count = self.history_state.records.len() as i32;
                let pos = args.first().copied().unwrap_or(0).clamp(0, count);
                self.history_state.scroll_y = pos;
                log::debug!("[trace-history] history_set_pos pos={pos}");
                ExtCallOutcome::Value(1)
            }
            5 => {
                // Native returns ctx+0x5D68: the laid-out pixel height of all
                // records; koikake's log screen compares it against 620 to
                // decide whether the scrollbar branch is needed.
                self.pop_ext_args(0);
                self.rebuild_history_layout(assets, nls);
                ExtCallOutcome::Value(self.history_state.height)
            }
            6 => {
                self.pop_ext_args(0);
                // koikake.exe 0x41EF20: hit-test the visible records' body
                // blocks against the mouse.  Only records carrying a voice
                // clip react: hovering shows the dark plate and a left click
                // stops all voice channels (0x431A70) then replays that
                // record's voice (0x431680).  No right-click/empty handling.
                let mut clicked_voice = None;
                if self.history_state.active {
                    if let Some(input) = input {
                        let (mouse_x, mouse_y) = input.mouse_position();
                        let [left, top, right, bottom] = self.history_state.rect;
                        let mut hovered = None;
                        if mouse_x >= left && mouse_x < right && mouse_y >= top && mouse_y < bottom
                        {
                            let (start, _end, tops) = self.history_visible_window(assets, nls);
                            let (records, name_block) = {
                                let cache = self
                                    .history_state
                                    .layout_cache
                                    .as_ref()
                                    .expect("history_visible_window installs the cache");
                                (cache.records.clone(), cache.name_block)
                            };
                            let rel_y = mouse_y - top;
                            for (offset, record_top) in tops.iter().enumerate() {
                                let record = &records[start + offset];
                                if !record.has_voice || record.body_height == 0 {
                                    continue;
                                }
                                let body_top = *record_top
                                    + if record.name.is_some() { name_block } else { 0 };
                                if rel_y >= body_top && rel_y < body_top + record.body_height {
                                    hovered = Some(start + offset);
                                    break;
                                }
                            }
                        }
                        if self.history_state.hovered_record != hovered {
                            log::debug!("[trace-history] history_update hover={hovered:?}");
                        }
                        self.history_state.hovered_record = hovered;
                        if let Some(record_index) = hovered {
                            if if self.button_release_mode { input.mouse_pull(PalMouseButton::Left) }
                                else { input.mouse_push(PalMouseButton::Left) } {
                                clicked_voice = Some(self.history_record_text(record_index, assets, nls).voice);
                            }
                        }
                    }
                } else {
                    self.history_state.hovered_record = None;
                }
                if let Some(voice) = clicked_voice {
                    log::debug!("[trace-history] history_update voice replay value={voice}");
                    // Native stops all eight voice channels (0x431A70) before
                    // replaying the record's clip.
                    if let Some(audio) = audio.as_deref_mut() {
                        let keys = self
                            .game_audio
                            .keys()
                            .filter(|(category, _)| *category == 13)
                            .copied()
                            .collect::<Vec<_>>();
                        for key in keys {
                            if let Some(handle) = self.game_audio.remove(&key) {
                                let _ = audio.release(handle);
                            }
                        }
                    }
                    self.replay_history_voice(&voice, resource_manager, audio);
                }
                ExtCallOutcome::Value(1)
            }
            7 => {
                // Native returns ctx+0x5D70: current scroll position in
                // records (0 = newest).
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.history_state.scroll_y)
            }
            8 => {
                // Native returns ctx+0x5D6C: the maximum scroll position in
                // records, i.e. how many of the oldest records do not fit into
                // the view when it is filled from the newest backward.
                self.pop_ext_args(0);
                let max_pos = self.history_layout(assets, nls).max_pos;
                ExtCallOutcome::Value(max_pos)
            }
            9 => {
                // Native returns ctx+0x5D98: the number of record sprites that
                // fit in the view.
                self.pop_ext_args(0);
                let visible = self.history_layout(assets, nls).visible_from_bottom;
                ExtCallOutcome::Value(visible as i32)
            }
            10 => {
                let args = self.pop_ext_args(4);
                if args.len() < 4 {
                    return ExtCallOutcome::Block;
                }
                self.history_state.rect = [args[0], args[1], args[2], args[3]];
                log::debug!(
                    "[trace-history] history_set_rect rect={:?}",
                    self.history_state.rect
                );
                ExtCallOutcome::Value(1)
            }
            11 => {
                self.pop_ext_args(0);
                self.history_state.records.clear();
                self.history_state.texts.clear();
                self.history_state.icons.records.clear();
                self.history_state.records_generation += 1;
                self.history_state.layout_cache = None;
                self.history_state.height = 0;
                self.history_state.scroll_y = 0;
                log::debug!("[trace-history] history_clear");
                ExtCallOutcome::Value(1)
            }
            12 => {
                let args = self.pop_ext_args(1);
                self.history_state.current_text_value = args.first().copied().unwrap_or(0);
                self.history_state.icons.config.voice_icon_enabled = true;
                self.history_state.icons.config.voice_icon_name = self.resolve_resource_string(
                    self.history_state.current_text_value, assets, nls).unwrap_or_default();
                log::debug!(
                    "[trace-history] history_set resource_or_text={}",
                    self.history_state.current_text_value
                );
                ExtCallOutcome::Value(1)
            }
            17 | 18 | 19 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(1)
            }
            20 => self.dispatch_history_get_text(assets, nls),
            _ => return ExtCallOutcome::Skip,
        }
    }

    /// Game category 21 thread extcalls.
    ///
    /// These are modeled as Game-side lightweight script thread identifiers.
    /// IDB evidence: create/suspend/get/exit use VM context fields at
    /// ctx_off_thread_active/running/target_pc/id; no PAL.dll export is known.
    fn dispatch_thread_stub(&mut self, index: u16, point_table: &PointTable) -> ExtCallOutcome {
        match index {
            0 => {
                // VmExtcall_CreateThread (0x0042E900) pops a target point id,
                // sets active/running to 1, stores the resolved target pc, and
                // records the original point id as thread_id.
                let args = self.pop_ext_args(1);
                let point_id = args.first().copied().unwrap_or(0);
                let target_pc = if point_id > 0 {
                    match point_table.resolve_target_pc(point_id as u32) {
                        Ok(Some(pc)) => Some(pc),
                        Ok(None) => None,
                        Err(err) => {
                            log::warn!(
                                "[trace-thread] create_thread point[{point_id}] failed: {err}"
                            );
                            None
                        }
                    }
                } else {
                    None
                };
                self.thread_state.next_id = self.thread_state.next_id.saturating_add(1).max(1);
                self.thread_state.running = i32::from(target_pc.is_some());
                self.thread_state.target_point = point_id;
                self.thread_state.last_id = point_id;
                self.thread_state.target_pc = target_pc;
                self.thread_state.vars = vec![0; DEFAULT_VAR_COUNT];
                self.thread_state.stack.clear();
                self.thread_state.argument_stack.clear();
                self.thread_state.argument_base = 0;
                self.thread_state.call_stack.clear();
                self.thread_state.wait = None;
                self.thread_state
                    .active
                    .insert(point_id, target_pc.is_some());
                log::debug!(
                    "[trace-thread] create_thread point_id={point_id} target_pc={target_pc:?}"
                );
                ExtCallOutcome::Value(1)
            }
            1 => {
                // VmExtcall_ExitThread (0x0042E8C0) clears the native thread
                // state block and returns success without consuming arguments.
                self.pop_ext_args(0);
                self.thread_state.active.clear();
                self.thread_state.last_id = 0;
                self.thread_state.running = 0;
                self.thread_state.target_point = 0;
                self.thread_state.target_pc = None;
                self.thread_state.vars.clear();
                self.thread_state.stack.clear();
                self.thread_state.argument_stack.clear();
                self.thread_state.argument_base = 0;
                self.thread_state.call_stack.clear();
                self.thread_state.wait = None;
                log::debug!("[trace-thread] exit_thread");
                ExtCallOutcome::Value(1)
            }
            2 => {
                // VmExtcall_SuspendThread (0x0042E860) pops the new
                // thread_running flag, returns the old value, then stores the
                // new value.  A zero-pop implementation leaks stack data and
                // breaks title/system wait restoration.
                let args = self.pop_ext_args(1);
                let running = args.first().copied().unwrap_or(0);
                let old = self.thread_state.running;
                self.thread_state.running = running;
                if running == 0 && self.thread_state.ticking {
                    self.thread_state.target_pc = Some(self.pc);
                    self.thread_state.vars = self.vars.clone();
                    self.thread_state.stack = self.stack.clone();
                    self.thread_state.argument_stack = self.argument_stack.clone();
                    self.thread_state.argument_base = self.argument_base;
                    self.thread_state.call_stack = self.call_stack.clone();
                }
                if self.thread_state.last_id != 0 {
                    self.thread_state
                        .active
                        .insert(self.thread_state.last_id, running != 0);
                }
                log::debug!("[trace-thread] suspend_thread running={running} old={old}");
                ExtCallOutcome::Value(old)
            }
            3 => {
                // VmExtcall_GetThread (0x0042E9A0) returns ctx_off_thread_id.
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.thread_state.last_id)
            }
            _ => return ExtCallOutcome::Skip,
        }
    }

    /// Game category 22 run/transition extcalls.
    ///
    /// `run` consumes effect id/duration/mode and drives the PAL-compatible
    /// effect pipeline; `run_no_wait` is a zero-argument latch, and `run_stack`
    /// queues/flushed stacked transition state.  Return value is integer success;
    /// blocking runs emit a wait request.  Evidence comes from Game.sqlite run
    /// handlers, PAL effect exports, and runtime trace compare.
    fn dispatch_fx_ext(&mut self, index: u16, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        // Reference Game.exe 0x4BAFE0[4..7]: clear, stop, wait, raster start.
        let count = match index { 4 | 6 => 1, 5 => 2, 7 => 4, _ => return ExtCallOutcome::Skip };
        if self.stack.len() < count {
            return ExtCallOutcome::Block;
        }
        let args = self.pop_ext_args(count);
        let handle = self.game_sprites.get(&args[0]).copied()
            .or_else(|| self.packed_button_sprite_handle(args[0]));
        let (Some(handle), Some(sprites)) = (handle, sprites) else {
            return ExtCallOutcome::Value(1);
        };
        match index {
            4 => { sprites.release_sprite_fx_effect(handle); }
            5 => { sprites.set_sprite_fx_effect(handle, 0, 2, args[1]); }
            6 => {
                if sprites.sprite_fx_effect_state(handle) != 0 && self.dispatch_skip_state() == 0 {
                    return ExtCallOutcome::Wait { value: 1, request: WaitRequest::SpriteFx(handle) };
                }
            }
            7 => {
                let source = crate::fx::RasterSource {
                    period_ms: args[3] as u32,
                    undulate_x: (args[1] as u32) as f32 * 0.01,
                    undulate_y: (args[2] as u32) as f32 * 0.01,
                };
                sprites.set_sprite_fx_effect_with_source(handle, 0, 1, args[3], source);
            }
            _ => unreachable!(),
        }
        ExtCallOutcome::Value(1)
    }

    fn dispatch_run_ext(&mut self, index: u16) -> ExtCallOutcome {
        match index {
            0 => {
                let args = self.pop_ext_args(3);
                if args.len() < 3 {
                    return ExtCallOutcome::Block;
                }
                let effect_id = args[0];
                let arg1 = args[1];
                let arg2 = args[2];
                if !(0..=100).contains(&effect_id) {
                    log::warn!("[trace-run] run effect_id out of range: {effect_id}");
                    return ExtCallOutcome::Value(0);
                }
                if self.run_pipeline.run_stack_enabled {
                    self.run_pipeline.pending_effect_id = effect_id;
                    self.run_pipeline.pending_arg1 = arg1;
                    self.run_pipeline.pending_arg2 = arg2;
                    self.run_pipeline.pending_kind = 1;
                    log::debug!(
                        "[trace-run] run queued stack effect={effect_id} arg1={arg1} arg2={arg2}"
                    );
                    return ExtCallOutcome::Value(1);
                }
                if effect_id != 0 && self.dispatch_skip_state() != 0 {
                    // koikake.exe 0x417DD9 ("run  skip"): while any skip state
                    // is active the PalEffectEx call is skipped entirely and
                    // the script does not wait for the transition.
                    self.run_pipeline.effect_active = false;
                    log::debug!("[trace-run] run skip effect={effect_id}");
                    return ExtCallOutcome::Value(1);
                }
                self.run_pipeline.effect_active = effect_id != 0;
                self.run_pipeline.last_run_time_ms = self.pal_time_ms;
                self.run_pipeline.last_run_arg1 = arg1;
                // Game.exe sub_421FD0 pops (effect_id, duration, effect_arg),
                // flushes the pending render queue through sub_421B60, then calls
                // PalEffectEx(effect_id, duration, effect_arg, 0).  The fourth PAL
                // argument is not the script's arg2 and must remain zero; native
                // records the duration in VM run-timing fields and the outer VM
                // pipeline performs the observable wait/yield.
                let wait = effect_id != 0 && arg1 > 0;
                let _ = self.effect_system.effect_ex(
                    effect_id.max(0) as u32,
                    arg1.max(1) as u32,
                    arg2,
                    false,
                    self.pal_time_ms,
                );
                log::debug!(
                    "[trace-run] run effect={effect_id} duration_ms={arg1} effect_arg={arg2} vm_wait={wait}"
                );
                if wait {
                    ExtCallOutcome::Wait {
                        value: 1,
                        request: WaitRequest::Time(arg1 as u32),
                    }
                } else {
                    ExtCallOutcome::Value(1)
                }
            }
            1 => {
                self.pop_ext_args(0);
                if self.run_pipeline.run_stack_enabled {
                    self.run_pipeline.pending_kind = 2;
                    log::debug!("[trace-run] run_no_wait queued stack");
                } else {
                    self.run_pipeline.no_wait_latch = true;
                    log::debug!("[trace-run] run_no_wait");
                }
                ExtCallOutcome::Value(1)
            }
            2 => {
                let args = self.pop_ext_args(1);
                let enabled = args.first().copied().unwrap_or(0) != 0;
                self.run_pipeline.run_stack_enabled = enabled;
                if enabled {
                    self.run_pipeline.pending_kind = 0;
                } else {
                    match self.run_pipeline.pending_kind {
                        1 => {
                            self.run_pipeline.effect_active =
                                self.run_pipeline.pending_effect_id != 0;
                            self.run_pipeline.last_run_time_ms = self.pal_time_ms;
                            self.run_pipeline.last_run_arg1 = self.run_pipeline.pending_arg1;
                            // sub_422160 flushes the queued run through the same
                            // PalEffectEx(effect, duration, arg, 0) path used by
                            // sub_421FD0; do not pass pending_arg2 as a PAL wait flag.
                            let _ = self.effect_system.effect_ex(
                                self.run_pipeline.pending_effect_id.max(0) as u32,
                                self.run_pipeline.pending_arg1.max(1) as u32,
                                self.run_pipeline.pending_arg2,
                                false,
                                self.pal_time_ms,
                            );
                            log::debug!(
                                "[trace-run] run_stack flush effect={} arg1={} arg2={}",
                                self.run_pipeline.pending_effect_id,
                                self.run_pipeline.pending_arg1,
                                self.run_pipeline.pending_arg2
                            );
                        }
                        2 => {
                            self.run_pipeline.no_wait_latch = true;
                            log::debug!("[trace-run] run_stack flush no_wait");
                        }
                        _ => {}
                    }
                    self.run_pipeline.pending_kind = 0;
                }
                log::debug!("[trace-run] run_stack enabled={enabled}");
                ExtCallOutcome::Value(1)
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    fn dispatch_misc_system_stub(&mut self, index: u16, _extended_softpal: bool) -> ExtCallOutcome {
        match index {
            2 => {
                // The public dispatch routes this savepoint-producing call
                // through ext_set_window_text with scene access.
                return ExtCallOutcome::Block;
            }
            4 => {
                // Native 0x411090 always pops eight, including in release
                // mode where the debug-output formatting is skipped.
                if self.pop_ext_args(8).len() != 8 { return ExtCallOutcome::Block; }
            }
            5 => {
                let args = self.pop_ext_args(1);
                let new_state = args.first().copied().unwrap_or(0);
                let old_state = self.debug_window_state;
                self.debug_window_state = new_state;
                log::debug!(
                    "[trace-debug] debug_window_set state={new_state} old_state={old_state}"
                );
                return ExtCallOutcome::Value(old_state);
            }
            6 => {
                self.pop_ext_args(0);
                log::debug!(
                    "[trace-debug] debug_window_get -> {}",
                    self.debug_window_state
                );
                return ExtCallOutcome::Value(self.debug_window_state);
            }
            _ => return ExtCallOutcome::Skip,
        }
        ExtCallOutcome::Value(1)
    }

    fn queue_pending_position_action(
        &mut self,
        slot: i32,
        handle: SpriteHandle,
        native_dx: f32,
        native_dy: f32,
        native_dz: f32,
        duration_ms: u32,
    ) {
        self.game_sprite_pending_position
            .push(PendingSpritePositionAction {
                slot,
                handle,
                native_dx,
                native_dy,
                native_dz,
                started_ms: self.pal_time_ms,
                duration_ms,
            });
    }

    fn native_delta_for_slot(&self, slot: i32, x: i32, y: i32) -> (f32, f32) {
        self.game_sprite_wrapper_visuals
            .get(&slot)
            .copied()
            .map(|visual| native_delta_for_visual(visual, x, y))
            .unwrap_or_else(|| (native_delta_x(x), native_delta_y(y)))
    }

    fn clear_pending_position_actions_for_slot(&mut self, slot: i32) {
        self.game_sprite_pending_position
            .retain(|action| action.slot != slot);
    }

    fn clear_alpha_lanes_for_slot(&mut self, slot: i32) {
        if slot == -1 {
            self.game_sprite_pending_alpha.clear();
            self.game_sprite_base_alpha.clear();
            self.game_sprite_final_alpha_delta.clear();
            self.game_sprite_active_alpha.clear();
        } else {
            self.game_sprite_pending_alpha.remove(&slot);
            self.game_sprite_base_alpha.remove(&slot);
            self.game_sprite_final_alpha_delta.remove(&slot);
            self.game_sprite_active_alpha
                .retain(|action| action.slot != slot);
        }
    }

    fn active_alpha_delta_for_slot(&self, slot: i32, handle: SpriteHandle) -> i32 {
        self.game_sprite_active_alpha
            .iter()
            .filter(|action| action.slot == slot && action.handle == handle)
            .map(|action| {
                let elapsed = self.pal_time_ms.wrapping_sub(action.started_ms);
                let clamped = elapsed.min(action.duration_ms);
                if action.duration_ms == 0 {
                    action.alpha_delta
                } else {
                    ((action.alpha_delta as f32)
                        * (clamped as f32 / action.duration_ms.max(1) as f32))
                        .round() as i32
                }
            })
            .sum()
    }

    fn active_alpha_delta_for_wrapper_slot(&self, slot: i32) -> i32 {
        self.game_sprite_active_alpha
            .iter()
            .filter(|action| action.slot == slot)
            .map(|action| {
                let elapsed = self.pal_time_ms.wrapping_sub(action.started_ms);
                let clamped = elapsed.min(action.duration_ms);
                if action.duration_ms == 0 {
                    action.alpha_delta
                } else {
                    ((action.alpha_delta as f32)
                        * (clamped as f32 / action.duration_ms.max(1) as f32))
                        .round() as i32
                }
            })
            .sum()
    }

    fn computed_game_sprite_alpha_for_new_wrapper_sprite(&self, slot: i32, fallback: u8) -> u8 {
        let base = self
            .game_sprite_base_alpha
            .get(&slot)
            .copied()
            .unwrap_or(i32::from(fallback));
        let final_delta = self
            .game_sprite_final_alpha_delta
            .get(&slot)
            .copied()
            .unwrap_or(0);
        let active_delta = self.active_alpha_delta_for_wrapper_slot(slot);
        (base + final_delta + active_delta).clamp(0, 255) as u8
    }

    fn computed_game_sprite_alpha(&self, slot: i32, handle: SpriteHandle) -> u8 {
        let base = self
            .game_sprite_base_alpha
            .get(&slot)
            .copied()
            .unwrap_or(255);
        let final_delta = self
            .game_sprite_final_alpha_delta
            .get(&slot)
            .copied()
            .unwrap_or(0);
        let active_delta = self.active_alpha_delta_for_slot(slot, handle);
        (base + final_delta + active_delta).clamp(0, 255) as u8
    }

    fn submit_game_sprite_alpha(
        &self,
        sprites: &mut SpriteSystem,
        slot: i32,
        handle: SpriteHandle,
    ) {
        let source_name = sprites
            .get(handle)
            .map(|sprite| sprite.source_name.clone())
            .unwrap_or_default();
        let old_alpha = sprites
            .get(handle)
            .map(|sprite| sprite.color.alpha())
            .unwrap_or(255);
        let alpha = self.computed_game_sprite_alpha(slot, handle);
        let base = self
            .game_sprite_base_alpha
            .get(&slot)
            .copied()
            .unwrap_or(255);
        let final_delta = self
            .game_sprite_final_alpha_delta
            .get(&slot)
            .copied()
            .unwrap_or(0);
        let active_delta = self.active_alpha_delta_for_slot(slot, handle);
        sprites.set_alpha(handle, alpha);
        if source_name.starts_with("ST")
            || source_name.starts_with("EV")
            || source_name.starts_with("FA")
            || source_name.starts_with("BK")
            || source_name.starts_with('#')
            || old_alpha != alpha
        {
            log::debug!(
                "[trace-alpha] t={} slot={} handle={:?} src={:?} old={} new={} base={} final_delta={} active_delta={} active_actions={}",
                self.pal_time_ms,
                slot,
                handle,
                source_name,
                old_alpha,
                alpha,
                base,
                final_delta,
                active_delta,
                self.game_sprite_active_alpha
                    .iter()
                    .filter(|action| action.slot == slot && action.handle == handle)
                    .count()
            );
        }
    }

    fn commit_position_action_delta(
        &mut self,
        sprites: &mut SpriteSystem,
        slot: i32,
        handle: SpriteHandle,
        native_dx: f32,
        native_dy: f32,
        native_dz: f32,
        logical_dx: f32,
        logical_dy: f32,
        logical_dz: f32,
    ) {
        let mut used_wrapper = false;
        if let Some(visual) = self.game_sprite_wrapper_visuals.get_mut(&slot) {
            visual.native_x += native_dx;
            visual.native_y += native_dy;
            visual.native_z += native_dz;
            used_wrapper = true;
        }
        if used_wrapper {
            let _ = self.commit_game_sprite_wrapper_visual(sprites, slot, handle);
        } else {
            let _ = sprites.move_pos(handle, logical_dx, logical_dy, logical_dz);
        }
    }

    fn queue_pending_alpha_action(&mut self, slot: i32, alpha_delta: i32, duration_ms: u32) {
        self.game_sprite_pending_alpha
            .entry(slot)
            .or_default()
            .push(PendingAlphaAction {
                alpha_delta,
                duration_ms,
                started_ms: self.pal_time_ms,
            });
        log::debug!(
            "[trace-action] action_alpha_delta queued sprite_slot={slot} delta={alpha_delta} duration_ms={duration_ms}"
        );
    }

    fn apply_pending_alpha_actions(
        &mut self,
        sprites: &mut SpriteSystem,
        slot: i32,
        handle: SpriteHandle,
    ) {
        let Some(actions) = self.game_sprite_pending_alpha.remove(&slot) else {
            return;
        };
        for action in actions {
            let elapsed = self.pal_time_ms.wrapping_sub(action.started_ms);
            let remaining = action.duration_ms.saturating_sub(elapsed);
            if remaining > 0 {
                self.game_sprite_active_alpha
                    .push(PendingSpriteAlphaAction {
                        slot,
                        handle,
                        alpha_delta: action.alpha_delta,
                        started_ms: self.pal_time_ms,
                        duration_ms: remaining,
                    });
            } else {
                *self.game_sprite_final_alpha_delta.entry(slot).or_insert(0) += action.alpha_delta;
            }
            self.submit_game_sprite_alpha(sprites, slot, handle);
            let target = self.computed_game_sprite_alpha(slot, handle);
            log::debug!(
                "[trace-action] action_alpha_delta applied pending slot={slot} handle={:?} delta={} target={target} remaining_ms={remaining}",
                handle,
                action.alpha_delta
            );
        }
    }

    fn apply_pending_text_alpha_actions(&mut self, sprites: &mut SpriteSystem) {
        if self.text_state.pending_alpha.is_empty() {
            return;
        }
        let handles = [self.text_state.sprite, self.text_state.name_sprite]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        if handles.is_empty() {
            return;
        }
        for action in self.text_state.pending_alpha.drain(..) {
            for handle in handles.iter().copied() {
                let Some(sprite) = sprites.get(handle) else {
                    continue;
                };
                let current = sprite.color.alpha() as i32;
                let target = (current + action.alpha_delta).clamp(0, 255) as u8;
                let elapsed = self.pal_time_ms.wrapping_sub(action.started_ms);
                let remaining = action.duration_ms.saturating_sub(elapsed);
                if remaining > 0 {
                    sprites.tween_alpha_to(handle, target, remaining);
                } else {
                    sprites.set_alpha(handle, target);
                }
                log::debug!(
                    "[trace-action] action_alpha_delta applied pending text_layer handle={:?} delta={} target={target} remaining_ms={remaining}",
                    handle,
                    action.alpha_delta
                );
            }
        }
    }

    /// Game category 23 message-queue extcalls.
    ///
    /// `create_message` creates a small Game-side message record, `get_message`
    /// consumes the first active message id, and `get_message_param` returns the
    /// resolved point/parameter value.  These are Game VM scheduling helpers; no
    /// PAL.dll export is assigned.
    fn dispatch_message_stub(&mut self, index: u16, point_table: &PointTable) -> ExtCallOutcome {
        match index {
            0 => {
                let args = self.pop_ext_args(1);
                let value = args.first().copied().unwrap_or(0);
                self.message_state.slots = [GameMessage::default(); 8];
                self.message_state.next_id = self.message_state.next_id.saturating_add(1).max(1);
                let slot = (value.max(0) as usize).min(7);
                let message = GameMessage {
                    active: true,
                    id: self.message_state.next_id,
                    value,
                    param: if value != 0 {
                        point_table
                            .resolve_target_pc(value as u32)
                            .ok()
                            .flatten()
                            .unwrap_or(0) as i32
                    } else {
                        0
                    },
                };
                self.message_state.slots[slot] = message;
                self.message_state.queue.push(message);
                ExtCallOutcome::Value(1)
            }
            1 => {
                self.pop_ext_args(0);
                for (idx, message) in self.message_state.slots.iter_mut().enumerate() {
                    if message.active {
                        message.active = false;
                        return ExtCallOutcome::Value(idx as i32);
                    }
                }
                ExtCallOutcome::Value(-1)
            }
            2 => {
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(-1);
                if (0..8).contains(&slot) {
                    ExtCallOutcome::Value(self.message_state.slots[slot as usize].param)
                } else {
                    ExtCallOutcome::Value(0)
                }
            }
            _ => ExtCallOutcome::Skip,
        }
    }

    fn dispatch_sprite_ext(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
        task_system: Option<&mut TaskSystem>,
    ) -> ExtCallOutcome {
        match index {
            2 => self.ext_sp_set(
                4,
                assets,
                nls,
                resource_manager,
                sprites,
                task_system,
                false,
            ),
            3 => self.ext_sp_set(
                5,
                assets,
                nls,
                resource_manager,
                sprites,
                task_system,
                false,
            ),
            4 => self.ext_sp_set_pos_ex(sprites),
            5 | 13 => self.ext_sp_cls(sprites, task_system),
            11 => self.ext_sp_cls_ex(sprites, task_system),
            6 => self.ext_sp_set_alpha(sprites),
            7 => self.ext_sp_set_priority_lane(),
            8 => self.ext_sp_get_filename(sprites),
            9 => self.ext_sp_set_center(sprites),
            12 => self.ext_sprite_filter(sprites),
            14 => self.ext_sp_move_ex(sprites),
            15 => self.ext_sp_set_rect_pos(sprites),
            16 => self.ext_sp_set_render_mode(sprites),
            17 => self.ext_sp_set_scale(sprites),
            18 => self.ext_sp_set_rotate(sprites),
            19 => self.ext_face_init(),
            20 => self.ext_face_set(assets, nls, resource_manager, sprites),
            21 => self.ext_sp_get_color(sprites),
            22 => self.ext_sp_text(assets, nls, sprites),
            23 => self.ext_face_cls(sprites),
            24 => self.ext_sp_set_rect(sprites),
            25 => self.ext_sp_set_pos_move(sprites),
            26 => self.ext_sp_get_alpha(sprites),
            27 => self.ext_sp_get_rotate(sprites),
            28 => self.ext_sp_get_pos_to_mem(sprites),
            29 => self.ext_sp_get_dimension(sprites, true),
            30 => self.ext_sp_get_dimension(sprites, false),
            31 => self.ext_sp_surface_image(assets, nls, resource_manager, sprites),
            32 => self.ext_sp_create(sprites),
            34 => self.ext_sp_set_anim_param(),
            35 => self.ext_sp_get_anim_param(),
            36 => self.ext_sp_get_scale(sprites),
            37 => self.ext_sp_set_color(sprites),
            38 => self.ext_sp_bitblt(sprites),
            39 => self.ext_sp_set_shake(sprites),
            40 => self.ext_sprite_paint(assets, nls, sprites),
            41 => self.ext_sp_set_anim(assets, nls, resource_manager, sprites, task_system),
            44 => self.ext_sp_set_vis_clip(),
            46 => self.ext_sp_view_ctrl(sprites, true),
            47 => self.ext_sp_view_ctrl(sprites, false),
            48 => self.ext_sp_is(sprites),
            49 => self.ext_sp_set_child(sprites),
            50 => self.ext_sp_set_transition(assets, nls, resource_manager, sprites),
            51 => self.ext_sp_copy_image(sprites),
            52 => self.ext_sp_transition(sprites),
            53 => self.ext_sp_set_aspect_position_type(sprites),
            54 => self.ext_sp_get_backbuffer(sprites),
            55 => self.ext_sp_set_mask(assets, nls, resource_manager, sprites),
            56 => self.ext_sp_set_motion_pos(assets, nls, resource_manager, sprites),
            57 => self.ext_sp_set_anim(assets, nls, resource_manager, sprites, task_system),
            _ => ExtCallOutcome::Skip,
        }
    }

    fn ext_btn_set(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(7);
        if args.len() < 7 {
            return ExtCallOutcome::Block;
        }
        let name_value = args[2];
        let Some(name) = self.resolve_resource_string(name_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        self.create_button_from_name(&args, name, resource_manager, sprites)
    }

    fn create_button_from_name(
        &mut self, args: &[i32], name: String,
        resource_manager: Option<&mut ResourceManager>, sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let (group, index, entry_flag) = (args[0], args[1], args[6]);
        if name.is_empty() {
            return ExtCallOutcome::Value(1);
        }
        let (Some(resource_manager), Some(sprites)) = (resource_manager, sprites) else {
            return ExtCallOutcome::Value(0);
        };
        if let Some(old) = self.game_buttons.remove(&(group, index)) {
            sprites.release(old.handle);
        }
        let surface_id = sprites.allocate_surface_id();
        let mut pending_button_animation = None;
        let (mut desc, source_name, log_size) = if let Some((a, r, g, b)) =
            parse_solid_color_name_argb(&name)
        {
            // Native sub_447FE0 accepts names beginning with '#'/BK_* as
            // synthetic solid surfaces.  Save/load menus use these through
            // btn_set for list-cell backgrounds; treating them as PGD files
            // leaves the UI with missing slot panels.
            let width = 288;
            let height = 96;
            let mut pixels = vec![0u8; width as usize * height as usize * 4];
            for px in pixels.chunks_exact_mut(4) {
                px.copy_from_slice(&[r, g, b, a]);
            }
            let surface = match SpriteSurface::rgba8(surface_id, 1, width, height, pixels) {
                Ok(surface) => surface,
                Err(err) => {
                    log::warn!(
                            "[trace-button] btn_set group={group} index={index} solid surface failed: {err}"
                        );
                    return ExtCallOutcome::Value(0);
                }
            };
            sprites.insert_surface(surface);
            let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), width, height);
            desc.cell_width = width;
            desc.cell_height = height;
            desc.position = PalVec3::new(0, 0, 0);
            (desc, name.clone(), format!("{width}x{height} solid"))
        } else if is_named_animation_resource(&name) {
            let asset = match open_resource_variant(resource_manager, &name, ANIMATION_EXTENSIONS) {
                Ok(asset) => asset,
                Err(err) => {
                    log::warn!(
                        "[trace-button] btn_set group={group} index={index} name={name:?} animation open failed: {err}"
                    );
                    return ExtCallOutcome::Value(0);
                }
            };
            // Some save/load button entries are pure named animations (for
            // scroll/highlight movement) rather than PGD images.  Native
            // `sub_40F550` resolves these through `sub_448710` before loading
            // the underlying sprite info; the portable path keeps a transparent
            // hit surface and applies the movement/alpha tween parsed from ANI.
            let width = 288;
            let height = 96;
            let pixels = vec![0u8; width as usize * height as usize * 4];
            let surface = match SpriteSurface::rgba8(surface_id, 1, width, height, pixels) {
                Ok(surface) => surface,
                Err(err) => {
                    log::warn!(
                        "[trace-button] btn_set group={group} index={index} animation surface failed: {err}"
                    );
                    return ExtCallOutcome::Value(0);
                }
            };
            sprites.insert_surface(surface);
            pending_button_animation = Some((asset.name.clone(), asset.bytes));
            let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), width, height);
            desc.cell_width = width;
            desc.cell_height = height;
            desc.position = PalVec3::new(0, 0, 0);
            (
                desc,
                asset.name.clone(),
                format!("{width}x{height} named-animation"),
            )
        } else {
            let asset = match open_resource_variant(resource_manager, &name, IMAGE_EXTENSIONS) {
                Ok(asset) => asset,
                Err(err) => {
                    log::warn!(
                            "[trace-button] btn_set group={group} index={index} name={name:?} open failed: {err}"
                        );
                    return ExtCallOutcome::Value(0);
                }
            };
            let decoded = match decode_asset_image(resource_manager, &asset) {
                Ok(decoded) => decoded,
                Err(err) => {
                    log::warn!(
                            "[trace-button] btn_set group={group} index={index} asset={:?} decode failed: {err}",
                            asset.name
                        );
                    return ExtCallOutcome::Value(0);
                }
            };
            let surface = match SpriteSurface::rgba8(
                surface_id,
                1,
                decoded.width,
                decoded.height,
                decoded.rgba,
            ) {
                Ok(surface) => surface,
                Err(err) => {
                    log::warn!(
                        "[trace-button] btn_set group={group} index={index} surface failed: {err}"
                    );
                    return ExtCallOutcome::Value(0);
                }
            };
            sprites.insert_surface(surface);
            let mut desc =
                SpriteDesc::new(SceneTextureId(surface_id.0), decoded.width, decoded.height);
            desc.cell_width = decoded.cell_width;
            desc.cell_height = decoded.cell_height;
            desc.position = PalVec3::new(decoded.offset_x, decoded.offset_y, 0);
            (
                desc,
                asset.name.clone(),
                format!(
                    "{}x{} cell={}x{}",
                    decoded.width, decoded.height, decoded.cell_width, decoded.cell_height
                ),
            )
        };
        desc.visible = true;
        desc.base_priority = button_render_priority(index, self.sprite_priority_cursor);
        desc.smooth_upscale = true;
        desc.source_name = source_name.clone();
        let handle = sprites.create(desc);
        if let Some((asset_name, bytes)) = pending_button_animation {
            if self.apply_named_sprite_animation(sprites, handle, &bytes) {
                log::debug!(
                    "[trace-button] btn_set group={group} index={index} asset={asset_name:?} applied named animation"
                );
            } else {
                log::warn!(
                    "[trace-button] btn_set group={group} index={index} asset={asset_name:?} unsupported named animation"
                );
            }
        }
        // args[3] is the script callback point invoked as a gosub when this button is clicked.
        let gosub_point = args.get(3).copied().filter(|&v| v > 0).map(|v| v as u32);
        let registration_order = self.next_button_registration;
        self.next_button_registration = self.next_button_registration.wrapping_add(1);
        self.game_buttons.insert(
            (group, index),
            GameButtonEntry {
                registration_order,
                native_flags: entry_flag as u32,
                hover_callbacks: [args[4], args[5]].map(|v| (v > 0).then_some(v as u32)),
                handle,
                name: name.clone(),
                visible: true,
                enabled: true,
                locked: self.button_groups.get(&group).is_some_and(|group| group.locked),
                toggle: 0,
                alpha: 255,
                slider_offset: 0,
                base_position: sprites.get(handle).map_or([0; 2], |s| [s.position.x as i32, s.position.y as i32]),
                slider_position: [0; 2],
                hit_rect: None,
                gosub_point,
                anim_resource: None,
                anim_play_flag: 0,
            },
        );
        log::debug!(
            "[trace-button] btn_set group={group} index={index} name={name:?} asset={source_name:?} size={log_size} entry_flag={entry_flag} gosub_point={gosub_point:?}",
        );
        ExtCallOutcome::Value(1)
    }

    /// Release every registered button group and its sprites. Scripts re-run
    /// the boot-time global init (text_init et al.) when long-jumping back to
    /// the title; that path never btn_uninits the abandoned screen groups, so
    /// their buttons would linger over the title and steal its clicks.
    fn clear_all_buttons(&mut self, sprites: Option<&mut SpriteSystem>) {
        self.button_children.clear();
        self.clear_button_capture(-1, -1);
        self.retire_button_sounds(-1);
        if self.game_buttons.is_empty() && self.button_groups.is_empty() {
            return;
        }
        if let Some(sprites) = sprites {
            let handles = self
                .game_buttons
                .values()
                .map(|entry| entry.handle)
                .collect::<Vec<_>>();
            for handle in handles {
                sprites.release(handle);
            }
        }
        self.game_buttons.clear();
        self.pending_button_reactions.clear();
        self.button_sound_hover.clear();
        self.button_callback_hover.clear();
        self.button_groups.clear();
        self.button_push_queue.clear();
        self.adv_menu_expanded = false;
        log::debug!("[trace-button] clear_all_buttons");
    }

    /// Game category 8 index 0 (`sub_40FC60`) pops
    /// `(group, normal_image, hover_image)`, resolves the two optional resource
    /// ids, and creates the native PAL button group.  Position is not part of
    /// this extcall; later btn_set/btn_set_pos calls own entry placement.
    fn ext_btn_init(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        if self.button_groups.contains_key(&group) { return ExtCallOutcome::Continue; }
        if group == 0 {
            self.adv_menu_expanded = false;
        }
        let normal_image = args[1];
        let hover_image = args[2];
        self.button_groups.insert(
            group,
            GameButtonGroup {
                normal_image,
                hover_image,
                onmouse_index: 0,
                offmouse_index: 0,
                push_index: 0,
                slider_ended: false,
                raw_toggle: 0,
                locked: false,
                lock_started_ms: 0,
                lock_duration_ms: 0,
            },
        );
        self.button_push_queue.remove(&group);
        log::debug!(
            "[trace-button] btn_init group={group} normal_image={normal_image} hover_image={hover_image}"
        );
        ExtCallOutcome::Continue
    }

    /// `btn_uninit(group)` releases all registered buttons for `group` (`-1`
    /// means all groups), clears queued reactions, and returns `1`.  This is the
    /// runtime counterpart of Game category 8 index 1; native evidence points to
    /// PAL button release/delete plus sprite release side effects.
    fn ext_btn_uninit(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let group = args.first().copied().unwrap_or(-1);
        self.clear_button_capture(group, -1);
        self.button_children.retain(|&(g, _), _| group >= 0 && g != group);
        self.retire_button_sounds(group);
        self.pending_button_reactions.retain(|&g, _| group >= 0 && g != group);
        self.button_callback_hover.retain(|&g, _| group >= 0 && g != group);
        self.button_sound_hover.retain(|&g, _| group >= 0 && g != group);
        if group < 0 || group == 0 {
            self.adv_menu_expanded = false;
        }
        let keys: Vec<_> = self
            .game_buttons
            .keys()
            .copied()
            .filter(|(button_group, _)| group < 0 || *button_group == group)
            .collect();
        let Some(sprites) = sprites else {
            for key in &keys {
                self.game_buttons.remove(key);
            }
            if group < 0 {
                self.button_push_queue.clear();
                self.button_groups.clear();
            } else {
                self.button_push_queue.remove(&group);
                self.button_groups.remove(&group);
            }
            return ExtCallOutcome::Value(1);
        };
        for key in keys {
            if let Some(entry) = self.game_buttons.remove(&key) {
                sprites.release(entry.handle);
            }
        }
        if group < 0 {
            self.button_push_queue.clear();
            self.button_groups.clear();
        } else {
            self.button_push_queue.remove(&group);
            self.button_groups.remove(&group);
        }
        log::debug!("[trace-button] btn_uninit group={group}");
        ExtCallOutcome::Value(1)
    }

    fn ext_btn_release(
        &mut self,
        opcode_index: u16,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let group = args.first().copied().unwrap_or(-1);
        let index = args.get(1).copied().unwrap_or(-1);
        self.clear_button_capture(group, index);
        let hovered: Vec<_> = self.button_callback_hover.iter()
            .filter(|(g, i)| (group < 0 || **g == group) && (index < 0 || **i == index))
            .map(|(&g, _)| g).collect();
        for group in hovered { self.leave_button_hover(group); }
        self.pending_button_reactions.retain(|&g, pending|
            !((group < 0 || g == group) && (index < 0 || pending.index == index)));
        if group < 0 || (group == 0 && index < 0) {
            self.adv_menu_expanded = false;
        }
        let Some(sprites) = sprites else {
            self.forget_button_handles(group, index);
            return ExtCallOutcome::Value(1);
        };
        let keys = self
            .game_buttons
            .keys()
            .copied()
            .filter(|(button_group, button_index)| {
                (group < 0 || *button_group == group) && (index < 0 || *button_index == index)
            })
            .collect::<Vec<_>>();
        for key in keys {
            if let Some(entry) = self.game_buttons.remove(&key) {
                sprites.release(entry.handle);
            }
        }
        if group < 0 {
            self.button_push_queue.clear();
        } else {
            self.button_push_queue.remove(&group);
        }
        log::debug!("[trace-button] btn_release opcode={opcode_index} group={group} index={index}");
        ExtCallOutcome::Value(1)
    }

    /// Koikake 0x40E700 reads the stored hover-callback index, without polling.
    fn ext_btn_get_onmouse(&mut self, _input: Option<&PalInputState>, _sprites: Option<&SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(&group) = args.first() else { return ExtCallOutcome::Block; };
        ExtCallOutcome::Value(self.button_groups.get(&group).map_or(0, |g| g.onmouse_index))
    }

    fn ext_btn_set_pos(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        let index = args[1];
        let x = args[2];
        let y = args[3];
        let mut sprites = sprites;
        for entry in self.matching_button_entries_mut(group, index) {
            entry.base_position = [x, y];
            if let Some(sprites) = sprites.as_deref_mut() {
                sprites.set_pos(entry.handle, x.wrapping_add(entry.slider_position[0]), y.wrapping_add(entry.slider_position[1]), 0);
            }
        }
        log::debug!("[trace-button] btn_set_pos group={group} index={index} pos=({x}, {y})");
        ExtCallOutcome::Value(1)
    }

    /// Category 8 index 9 is the native slider-position query used by the
    /// SYSTEM/SOUND menu.  Game.sqlite sub_40EC10 pops
    /// `(group,index,max_offset,axis,snap_to_100)` and returns the live offset,
    /// using the current mouse position when available.  The returned value is
    /// later converted into text-window/audio volume percentages by script.
    fn ext_btn_slider_get(
        &mut self,
        input: Option<&PalInputState>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        let index = args[1];
        let max_offset = args[2].max(0);
        let axis = args[3];
        let snap_to_100 = args[4] != 0;
        let mut offset = self
            .game_buttons
            .get(&(group, index))
            .map_or(0, |entry| entry.slider_offset);
        let mut knob_handle = None;
        let mut knob_anchor: Option<(i32, i32)> = None;
        if let (Some(input), Some(sprites)) = (input, sprites) {
            if let Some(entry) = self.game_buttons.get(&(group, index)) {
                let (mouse_x, mouse_y) = input.mouse_position();
                if mouse_x >= 0 && mouse_y >= 0 {
                    if let Some(sprite) = sprites.get(entry.handle) {
                        let anchor = self
                            .slider_anchor_position(sprites, group, index, axis)
                            .unwrap_or_else(|| {
                                let pos = sprite.effective_position();
                                (pos.x, pos.y)
                            });
                        knob_anchor = Some(anchor);
                        let size = sprite.source_rect;
                        let raw = if axis == 1 {
                            mouse_y - anchor.1 - (size.height() / 2)
                        } else {
                            mouse_x - anchor.0 - (size.width() / 2)
                        };
                        offset = raw.clamp(0, max_offset);
                        if snap_to_100
                            && max_offset >= 100
                            && offset != 0
                            && offset != max_offset
                        {
                            // Native hundred-step snapping: round the offset so
                            // the script-side `offset * 100 / max` percent is
                            // integral.
                            let percent = (offset as i64 * 100 + max_offset as i64 / 2)
                                / max_offset as i64;
                            offset = (percent * max_offset as i64 / 100) as i32;
                        }
                        knob_handle = Some(entry.handle);
                    }
                }
            }
            if let Some(entry) = self.game_buttons.get_mut(&(group, index)) {
                entry.slider_offset = offset;
            }
            if let Some(handle) = knob_handle {
                if let Some(sprite) = sprites.get_mut(handle) {
                    if axis == 1 {
                        if let Some((_, base_y)) = knob_anchor {
                            sprite.position.y = (base_y + offset) as f32;
                        }
                    } else if let Some((base_x, _)) = knob_anchor {
                        sprite.position.x = (base_x + offset) as f32;
                    }
                }
            }
        } else if let Some(entry) = self.game_buttons.get_mut(&(group, index)) {
            entry.slider_offset = offset;
        }
        log::debug!(
            "[trace-button] btn_slider_get group={group} index={index} max={max_offset} axis={axis} snap={snap_to_100} -> {offset}"
        );
        ExtCallOutcome::Value(offset)
    }

    /// Category 8 index 10 initializes or updates a slider button from its
    /// logical value.  Observed scripts pass `(group,index,travel,axis,value)`
    /// before entering a poll loop: `travel` is the knob's pixel travel
    /// (slider width minus end caps) and `value` is the current setting as a
    /// percentage of that travel (BGM deliberately passes up to 250 for a
    /// pinned-max knob).  Native stores the knob offset and moves the button
    /// cell to `anchor + travel * value / 100`.
    fn ext_btn_slider_set(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        let index = args[1];
        let travel = args[2].max(0);
        let axis = args[3];
        let value = args[4];
        let offset = (travel as i64 * value as i64 / 100).clamp(0, travel as i64) as i32;
        // Resolve the anchor from the pre-update offset: the knob still rests
        // at `anchor + old_offset`, so the old value pins the anchor down.
        let anchor = sprites
            .as_deref()
            .and_then(|sprites| self.slider_anchor_position(sprites, group, index, axis));
        for entry in self.matching_button_entries_mut(group, index) {
            entry.slider_offset = offset;
        }
        if let Some(sprites) = sprites {
            for handle in self.matching_button_handles(group, index) {
                if let Some(sprite) = sprites.get_mut(handle) {
                    if axis == 1 {
                        if let Some((_, base_y)) = anchor {
                            sprite.position.y = (base_y + offset) as f32;
                        } else {
                            sprite.position.y = offset as f32;
                        }
                    } else {
                        if let Some((base_x, _)) = anchor {
                            sprite.position.x = (base_x + offset) as f32;
                        } else {
                            sprite.position.x = offset as f32;
                        }
                    }
                }
            }
        }
        log::debug!(
            "[trace-button] btn_slider_set group={group} index={index} travel={travel} axis={axis} value={value} -> offset={offset}"
        );
        ExtCallOutcome::Value(1)
    }

    /// Koikake 0x40EBD0 stores the group slider-end latch from stable held/pull.
    fn ext_btn_slider_begin(&mut self, input: Option<&PalInputState>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() != 2 { return ExtCallOutcome::Block; }
        self.button_groups.entry(args[0]).or_default().slider_ended = input.is_none_or(|input|
            input.raw_key_on() & (1 << 23) == 0 || input.raw_key_pull() & (1 << 23) != 0);
        ExtCallOutcome::Continue
    }

    /// Koikake 0x40EB20 returns -1 after the group latch/pull, otherwise 0.
    fn ext_btn_on_check(&mut self, input: Option<&PalInputState>, _sprites: Option<&SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() != 2 { return ExtCallOutcome::Block; }
        let group = self.button_groups.entry(args[0]).or_default();
        if input.is_some_and(|i| i.raw_key_pull() & (1 << 23) != 0) { group.slider_ended = true; }
        ExtCallOutcome::Value(if group.slider_ended { -1 } else { 0 })
    }

    /// Koikake 0x40E770 reads the completed-callback latch without consuming it.
    fn ext_btn_get_push(&mut self, _input: Option<&PalInputState>, _sprites: Option<&SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(&group) = args.first() else { return ExtCallOutcome::Block; };
        ExtCallOutcome::Value(self.button_groups.get(&group).map_or(0, |g| g.push_index))
    }

    /// `btn_hide(group,index)` / `btn_show(group,index)` correspond to
    /// Game.exe sub_40F290/sub_40F1B0.  The native handlers pop group/index and
    /// enqueue visibility commands; the portable renderer applies visibility
    /// directly to matching sprite handles.
    fn ext_btn_view_ctrl(
        &mut self,
        visible: bool,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let group = args.first().copied().unwrap_or(-1);
        let index = args.get(1).copied().unwrap_or(-1);
        for entry in self.matching_button_entries_mut(group, index) {
            entry.visible = visible;
        }
        if let Some(sprites) = sprites {
            for handle in self.matching_button_handles(group, index) {
                sprites.view_ctrl(handle, visible);
            }
        }
        log::debug!(
            "[trace-button] btn_{} group={group} index={index}",
            if visible { "show" } else { "hide" }
        );
        ExtCallOutcome::Value(1)
    }

    /// Koikake Game.exe 0x40E890 calls PalButtonCtrl without changing pixels,
    /// alpha, or the selected sprite cell. Its return is VM control flow.
    fn ext_btn_enable(&mut self, _sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let (group, index, enabled) = (args[0], args[1], args[2] != 0);
        for entry in self.matching_button_entries_mut(group, index) {
            entry.enabled = enabled;
        }
        log::debug!("[trace-button] btn_enable group={group} index={index} enabled={enabled}");
        ExtCallOutcome::Continue
    }

    /// `btn_lock(group,duration_ms)` matches Game.exe sub_40E0C0.  Native state
    /// is group-scoped: it stores a lock flag, duration/timer value, and start
    /// time.  Koikake locks with `duration_ms == 0` around menu transitions
    /// (title menu handler, settings/save screen entry) and pairs every one
    /// with `btn_unlock` at the end of the transition, so the zero case means
    /// "locked until btn_unlock", not "no lock": the shared title-menu handler
    /// depends on it to block re-entrant clicks while it is still running.
    fn ext_btn_lock(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let group = args.first().copied().unwrap_or(-1);
        let duration_ms = args.get(1).copied().unwrap_or(0);
        for (&id, state) in &mut self.button_groups {
            if group >= 0 && id != group { continue; }
            state.locked = true;
            state.lock_started_ms = self.pal_time_ms;
            state.lock_duration_ms = duration_ms as u32;
        }
        for entry in self.matching_button_entries_mut(group, -1) {
            entry.locked = true;
        }
        log::debug!("[trace-button] btn_lock group={group} duration_ms={duration_ms}");
        ExtCallOutcome::Value(1)
    }

    /// `btn_unlock(group)` matches Game.exe sub_40E040 and clears native
    /// group-level lock fields.  The compatibility runtime unlocks every entry
    /// in the group.
    fn ext_btn_unlock(&mut self, _sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let group = args.first().copied().unwrap_or(-1);
        for (&id, state) in &mut self.button_groups {
            if group < 0 || id == group { state.locked = false; }
        }
        for entry in self.matching_button_entries_mut(group, -1) {
            entry.locked = false;
        }
        log::debug!("[trace-button] btn_unlock group={group}");
        ExtCallOutcome::Value(1)
    }

    /// Koikake render-command consumer 0x417EA0 handles command 0x8000D
    /// by setting the PAL button base row to three times the raw toggle word.
    fn ext_btn_set_toggle(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        let index = args[1];
        let toggle = args[2];
        if self.game_buttons.contains_key(&(group, index)) {
            self.button_groups.entry(group).or_default().raw_toggle = toggle;
        }
        for entry in self.matching_button_entries_mut(group, index) {
            entry.toggle = toggle.wrapping_mul(3);
        }
        if let Some(sprites) = sprites {
            let cell_y = toggle.wrapping_mul(3);
            for handle in self.matching_button_handles(group, index) {
                sprites.rect_set_pos(handle, 0, cell_y);
            }
        }
        log::debug!("[trace-button] btn_set_toggle group={group} index={index} toggle={toggle}");
        ExtCallOutcome::Value(1)
    }

    /// Category 8 index 14 writes a button's current x/y position into two
    /// caller-chosen temporary-memory slots. Both games read those slots
    /// before placing another button. Leaving them stale can turn File.dat
    /// string handles into offscreen coordinates.
    fn ext_btn_get_pos(&mut self, sprites: Option<&SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let (group, index, x_slot, y_slot) = (args[0], args[1], args[2], args[3]);
        let position = self
            .game_buttons
            .get(&(group, index))
            .and_then(|entry| sprites?.get(entry.handle))
            .map(|sprite| {
                (
                    sprite.position.x.round() as i32,
                    sprite.position.y.round() as i32,
                )
            });
        let Some((x, y)) = position else {
            return ExtCallOutcome::Value(0);
        };
        self.write_temp_mem_absolute(x_slot, x);
        self.write_temp_mem_absolute(y_slot, y);
        log::debug!("[trace-button] btn_get_pos group={group} index={index} slots=({x_slot},{y_slot}) pos=({x},{y})");
        ExtCallOutcome::Value(1)
    }

    /// Category 8 index 29 selects the button control mode and visual cell.
    fn ext_btn_set_state(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        let index = args[1];
        let ctrl = args[2];
        let state = args[3];
        for entry in self.matching_button_entries_mut(group, index) {
            entry.enabled = ctrl != 0;
            entry.toggle = state;
        }
        if let Some(sprites) = sprites {
            for handle in self.matching_button_handles(group, index) {
                sprites.rect_set_pos(handle, 0, state);
            }
        }
        log::debug!(
            "[trace-button] btn_set_state group={group} index={index} ctrl={ctrl} state={state}"
        );
        ExtCallOutcome::Value(1)
    }

    /// `btn_set_alpha_0x(group,index,alpha)` matches sub_40E4B0.  It writes
    /// the alpha byte into the button sprite color field; the portable path
    /// clamps to 0..255 and updates the live sprite color.
    fn ext_btn_set_alpha(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        let index = args[1];
        let alpha = args[2] as u8;
        for entry in self.matching_button_entries_mut(group, index) {
            entry.alpha = alpha;
        }
        if let Some(sprites) = sprites {
            for handle in self.matching_button_handles(group, index) {
                if let Some(sprite) = sprites.get_mut(handle) {
                    let raw = sprite.color.0 & 0x00FF_FFFF;
                    sprite.color = PalColor::from_argb(raw | ((alpha as u32) << 24));
                }
            }
        }
        log::debug!("[trace-button] btn_set_alpha group={group} index={index} alpha={alpha}");
        ExtCallOutcome::Value(1)
    }

    fn ext_btn_get_alpha(&mut self, sprites: Option<&SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let (group, index) = (args[0], args[1]);
        let alpha = self
            .game_buttons
            .get(&(group, index))
            .map(|entry| {
                sprites
                    .and_then(|sprites| sprites.get(entry.handle))
                    .map_or(entry.alpha, |sprite| sprite.color.alpha())
            })
            .unwrap_or(0);
        ExtCallOutcome::Value(i32::from(alpha))
    }

    fn ext_btn_set_anim(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let group = args[0];
        let index = args[1];
        let anim_value = args[2];
        let play_flag = args[3];
        let anim_name = self.resolve_resource_string(anim_value, assets, nls);
        let matched = self.matching_button_handles(group, index);
        if matched.is_empty() {
            // Game.exe sub_40DE40 checks the button-table entry pointer before
            // resolving/opening the animation resource.  Some reachable traces
            // pass large table coordinates that do not name a live portable
            // button entry; native returns success without touching resources.
            log::debug!(
                "[trace-button] btn_set_anim group={group} index={index} anim_value={anim_value} name={anim_name:?} play_flag={play_flag} missing_entry"
            );
            return ExtCallOutcome::Value(1);
        }
        let matching_keys = self
            .game_buttons
            .keys()
            .copied()
            .filter(|(entry_group, entry_index)| {
                (group == -1 || *entry_group == group) && (index == -1 || *entry_index == index)
            })
            .collect::<Vec<_>>();
        for key in matching_keys {
            if let Some(entry) = self.game_buttons.get_mut(&key) {
                entry.anim_resource = anim_name.clone();
                entry.anim_play_flag = play_flag;
            }
        }
        match (sprites, anim_name.as_ref(), resource_manager) {
            (Some(sprites), Some(name), Some(resource_manager)) => {
                match open_resource_variant(resource_manager, name, ANIMATION_EXTENSIONS) {
                    Ok(asset) => {
                        for handle in matched {
                            if !self.apply_named_sprite_animation(sprites, handle, &asset.bytes) {
                                log::warn!(
                                    "[trace-button] btn_set_anim group={group} index={index} asset={:?} unsupported named animation",
                                    asset.name
                                );
                            }
                        }
                    }
                    Err(err) => {
                        log::warn!(
                            "[trace-button] btn_set_anim group={group} index={index} name={name:?} open failed: {err}"
                        );
                    }
                }
            }
            (Some(sprites), _, _) => {
                let row = play_flag;
                for handle in matched {
                    sprites.rect_set_pos(handle, 0, row);
                }
            }
            _ => {}
        }
        log::debug!(
            "[trace-button] btn_set_anim group={group} index={index} anim_value={anim_value} name={anim_name:?} play_flag={play_flag}"
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_btn_expansion(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let mode = args[0];
        let index = args[1];
        let state = args[2];
        if mode == 0 {
            match index {
                // MAIN_BTN_SKIP/AUTO are registered as compact sprite buttons; the
                // expansion call selects their visual row/state after the script
                // has made the base controls transparent.
                0 | 1 => {
                    for entry in self.matching_button_entries_mut(0, index) {
                        entry.toggle = state;
                    }
                    if let Some(sprites) = sprites {
                        for handle in self.matching_button_handles(0, index) {
                            sprites.rect_set_pos(handle, 0, state);
                        }
                    }
                }
                // MAIN_BTN_VOICE is a wide pop-out strip. State 2/4 in the normal
                // ADV setup means it is registered for later use but not drawn as
                // the default text-window chrome.
                14 => {
                    for entry in self.matching_button_entries_mut(0, 14) {
                        entry.enabled = false;
                        entry.alpha = 0;
                    }
                    if let Some(sprites) = sprites {
                        for handle in self.matching_button_handles(0, 14) {
                            sprites.view_ctrl(handle, false);
                            if let Some(sprite) = sprites.get_mut(handle) {
                                sprite.color = PalColor::from_argb(sprite.color.0 & 0x00FF_FFFF);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        log::debug!(
            "[trace-button] btn_expansion mode={} index={} state={}",
            mode,
            index,
            state
        );
        ExtCallOutcome::Value(1)
    }

    /// Synthetic PalButtonSetReaction starts sound on the next poll and
    /// completes on a later poll after that sound stops. An existing pending
    /// reaction rejects replacement, including a second script shortcut.
    fn ext_btn_set_hit(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let group = args.first().copied().unwrap_or(-1);
        let index = args.get(1).copied().unwrap_or(-1);
        if self.game_buttons.contains_key(&(group, index)) {
            self.begin_button_reaction(group, index, true, None);
        }
        log::debug!("[trace-button] btn_set_hit group={group} index={index}");
        ExtCallOutcome::Continue
    }

    /// Game category 4 BGM extcalls.
    ///
    /// Calls dispatch to PAL-style audio groups: play/load/stop mutate
    /// `game_audio`, volume calls mutate group volume and return integer status
    /// or current volume.  Evidence: shared ExtSig audio rows, Game.sqlite audio
    /// handlers, and PAL audio-group behavior captured in runtime fixtures.
    fn dispatch_bgm_ext(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        match index {
            0 => self.ext_bgm_play(assets, nls, resource_manager, audio),
            1 => self.ext_bgm_stop(assets, audio),
            2 => self.ext_bgm_set_volume(audio),
            3 => {
                // Native zero-argument query returns configured BGM percent,
                // independently of the live muted or ducked group level.
                ExtCallOutcome::Value(self.bgm_volume_percent)
            }
            4 => {
                // bgm_get_auto_volume(): SOUND menu queries this zero-arg value
                // to seed the auto/BGM slider.  Native keeps this separate
                // from the ordinary BGM group volume.
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.bgm_auto_volume_percent)
            }
            6 => self.ext_bgm_set_auto_volume(audio),
            8 => self.ext_bgm_filename(assets, nls),
            9 => self.ext_bgm_load(assets, nls, resource_manager, audio),
            10 => self.ext_bgm_play_loaded(assets, audio),
            11 => self.ext_set_master_volume(audio),
            12 => {
                // get_master_volume(): native returns the PAL primary volume as
                // a script percent.  The setting page expects 0..100.
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.master_volume_percent)
            }
            13 => self.ext_mute_master_volume(audio),
            14 => self.ext_bgm_mute(audio),
            15 => self.ext_mute_bgm_auto_volume(),
            16 => ExtCallOutcome::Value(0),
            17 => ExtCallOutcome::Value(0),
            18 => ExtCallOutcome::Value(0),
            _ => ExtCallOutcome::Skip,
        }
    }

    /// Game category 5 SE extcalls.
    ///
    /// SE play/load/stop/wait operations map to PAL sound-effect group state.
    /// Wait/query calls return integer completion status; exact native async
    /// completion flags are approximated by current audio handle state.
    fn dispatch_se_ext(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        match index {
            0 | 1 | 2 => self.ext_se_play(index, assets, nls, resource_manager, audio),
            3 => self.ext_audio_stop(5, PalSoundGroup::GROUP4, audio),
            4 => {
                let args = self.pop_ext_args(2);
                let volume = args.first().copied().unwrap_or(100);
                let slot = args.get(1).copied().unwrap_or(0);
                self.se_volume_percent.insert(slot, clamp_percent(volume));
                self.se_enabled.entry(slot).or_insert(true);
                if let Some(audio) = audio {
                    let _ = audio
                        .set_group_volume(PalSoundGroup::GROUP4, self.effective_se_group_volume());
                }
                log::debug!(
                    "[trace-audio] se_set_volume slot={slot} volume_percent={} effective_group_raw={}",
                    clamp_percent(volume),
                    self.effective_se_group_volume().raw()
                );
                ExtCallOutcome::Value(1)
            }
            5 => {
                // se_get_volume(slot) returns the configured percent for that
                // SE lane; the SOUND menu seeds its SE/SSE sliders from it.
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(0);
                ExtCallOutcome::Value(self.se_volume_percent.get(&slot).copied().unwrap_or(100))
            }
            6 => self.ext_audio_stop(5, PalSoundGroup::GROUP4, audio),
            7 => self.ext_se_wait(audio),
            14 => self.ext_se_mute(audio),
            _ => ExtCallOutcome::Skip,
        }
    }

    /// Game category 13 voice/BGV extcalls.
    ///
    /// Evidence: Game.sqlite `sub_4445F0` pops one percent value and stores the
    /// PAL-scaled voice volume; `sub_4445A0` pops no script argument and writes
    /// that cached volume to the extcall destination; `sub_443800` pops the slot
    /// only on the first `voice_wait` pass, then rewinds the script PC by 12
    /// bytes until the voice wait mask is cleared.
    fn dispatch_voice_ext(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        match index {
            0 => self.ext_voice_play(assets, nls, resource_manager, audio),
            1 => self.ext_audio_stop(13, PalSoundGroup::GROUP1, audio),
            2 => {
                let args = self.pop_ext_args(1);
                self.text_state.voice_volume = clamp_percent(args.first().copied().unwrap_or(100));
                if let Some(audio) = audio {
                    let _ = audio.set_group_volume(
                        PalSoundGroup::GROUP1,
                        percent_to_volume(self.text_state.voice_volume),
                    );
                }
                ExtCallOutcome::Value(1)
            }
            3 => {
                // voice_get_volume(): native sub_4445A0 does not pop a slot.
                ExtCallOutcome::Value(self.text_state.voice_volume)
            }
            4 => {
                self.ext_set_voice_info(assets, nls)
            }
            5 => {
                // voice_enable(slot, enabled, extra): category 13 index 5 is
                // observed popping three values; the SOUND menu per-character
                // mute checks drive it with the unit slot.  Negative slots keep
                // the historical global latch behavior.
                let args = self.pop_ext_args(3);
                let slot = args.first().copied().unwrap_or(-1);
                let enabled = args.get(1).copied().unwrap_or(1) != 0;
                if slot < 0 {
                    self.text_state.voice_enabled = enabled;
                } else {
                    self.voice_ex_enabled.insert(slot, enabled);
                }
                ExtCallOutcome::Value(1)
            }
            6 | 13 => {
                // is_voice_enable(slot): per-slot answer when the unit grid
                // queries a character, global latch for negative slots.
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(-1);
                let enabled = if slot < 0 {
                    self.text_state.voice_enabled
                } else {
                    self.voice_ex_enabled
                        .get(&slot)
                        .copied()
                        .unwrap_or(self.text_state.voice_enabled)
                };
                ExtCallOutcome::Value(i32::from(enabled))
            }
            7 => {
                // Game.exe 0x444190 stores ctx+660028: the enable gate read
                // by the voice-info consumer at 0x4425F0, not a fade time.
                let args = self.pop_ext_args(1);
                let Some(&enabled) = args.first() else { return ExtCallOutcome::Block; };
                self.text_state.voice_info_enabled = Some(enabled != 0);
                ExtCallOutcome::Value(1)
            }
            8 => self.ext_bgv_play(assets, nls, resource_manager, audio),
            9 => self.ext_bgv_stop(audio),
            10 => {
                let args = self.pop_ext_args(1);
                self.text_state.bgv_enabled = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            11 => {
                // get_voice_ex_volume(slot): per-character voice percent for the
                // SOUND menu unit grid; slots without an entry follow the global
                // voice volume.
                let args = self.pop_ext_args(1);
                let slot = args.first().copied().unwrap_or(0);
                let value = self
                    .voice_ex_volume_percent
                    .get(&slot)
                    .copied()
                    .unwrap_or(self.text_state.voice_volume);
                ExtCallOutcome::Value(value)
            }
            12 => {
                // set_voice_ex_volume(slot, volume, extra): the drag loop pushes
                // the character slot, the slider percent, and a third scratch
                // value (unmapped in native notes); all three are consumed.
                let args = self.pop_ext_args(3);
                let slot = args.first().copied().unwrap_or(0);
                let volume = clamp_percent(args.get(1).copied().unwrap_or(100));
                self.voice_ex_volume_percent.insert(slot, volume);
                ExtCallOutcome::Value(1)
            }
            14 => {
                let args = self.pop_ext_args(1);
                self.text_state.voice_autopan_limit = args.first().copied().unwrap_or(0).wrapping_mul(100);
                ExtCallOutcome::Value(1)
            }
            15 => {
                let args = self.pop_ext_args(1);
                self.text_state.voice_autopan_enabled = args.first().copied().unwrap_or(1) != 0;
                ExtCallOutcome::Value(1)
            }
            16 => {
                // Game.exe 0x004438D0 (`VmExtcall_VoiceAutopanSizeOver`) pops
                // (slot, target, name, mode) from the VM stack.  If target is
                // -1 and mode is 0, native clears the 40-byte autopan entry at
                // ctx+658964+40*slot; otherwise it stores mode, target (or the
                // resolved sprite wrapper when mode == 0), and a <=32 byte
                // string copied through sub_44B1E0. Playback snapshots the
                // first matching prefix and follows that target every frame.
                let args = self.pop_ext_args(4);
                let slot = args.first().copied().unwrap_or(0);
                let target = args.get(1).copied().unwrap_or(-1);
                let name_value = args.get(2).copied().unwrap_or(0);
                let mode = args.get(3).copied().unwrap_or(0);
                if target == -1 && mode == 0 {
                    self.text_state.voice_autopan_entries.remove(&slot);
                    log::debug!("[trace-voice] set_voice_autopan_size_over clear slot={slot}");
                    return ExtCallOutcome::Value(1);
                }
                let name = self
                    .resolve_script_string(name_value, assets, nls)
                    .unwrap_or_default();
                let encoded_len = nls
                    .encode(&name)
                    .map(|bytes| bytes.len())
                    .unwrap_or_else(|_| name.len());
                if encoded_len > 32 {
                    log::warn!(
                        "[trace-voice] set_voice_autopan_size_over name too long slot={slot} name={name:?} encoded_len={encoded_len}"
                    );
                    return ExtCallOutcome::Value(0);
                }
                self.text_state.voice_autopan_entries.insert(
                    slot,
                    VoiceAutopanEntry {
                        target,
                        name_value,
                        mode,
                        name: name.clone(),
                    },
                );
                log::debug!(
                    "[trace-voice] set_voice_autopan_size_over slot={slot} target={target} name={name:?} mode={mode}"
                );
                ExtCallOutcome::Value(1)
            }
            17 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(i32::from(self.text_state.voice_autopan_enabled))
            }
            18 => {
                let slot = match self.pending_voice_wait_slot {
                    Some(slot) => slot,
                    None => {
                        let args = self.pop_ext_args(1);
                        args.first().copied().unwrap_or(-1)
                    }
                };
                let handles = self
                    .game_audio
                    .iter()
                    .filter_map(|((category, audio_slot), handle)| {
                        // Native 0x443800 expands -1 to mask 0xFF:
                        // the eight ADV lanes, excluding independent lane 8.
                        (*category == 13
                            && if slot == -1 {
                                (0..8).contains(audio_slot)
                            } else {
                                *audio_slot == slot
                            })
                            .then_some(*handle)
                    })
                    .collect::<Vec<_>>();
                let any_playing = audio.is_some_and(|audio| {
                    handles
                        .iter()
                        .copied()
                        .any(|handle| audio.is_playing(handle).unwrap_or(false))
                });
                if any_playing {
                    self.pending_voice_wait_slot = Some(slot);
                    self.pc = self.pc.saturating_sub(self.extcall_instruction_size);
                    log::debug!("[trace-audio] voice_wait slot={slot} still playing");
                    return ExtCallOutcome::Wait {
                        value: 1,
                        request: WaitRequest::Frame(1),
                    };
                }
                self.pending_voice_wait_slot = None;
                log::debug!("[trace-audio] voice_wait slot={slot} complete");
                ExtCallOutcome::Value(1)
            }
            19 => self.ext_bgv_pause(audio),
            20 => self.ext_bgv_mute(audio),
            24 => self.ext_voice_mute(audio),
            21 => self.ext_bgv_set_volume(audio),
            23 => {
                self.pop_ext_args(1);
                ExtCallOutcome::Value(1)
            }
            22 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(self.text_state.bgv_volume)
            }
            25 => {
                self.pop_ext_args(2);
                ExtCallOutcome::Value(1)
            }
            26 => {
                self.pop_ext_args(0);
                ExtCallOutcome::Value(1)
            }
            28 => self.dispatch_wait_ext(0),
            29 => self.dispatch_wait_ext(1),
            30 => self.dispatch_wait_ext(2),
            31 => self.dispatch_wait_ext(3),
            32 => self.dispatch_wait_ext(4),
            34 => self.dispatch_wait_ext(6),
            35 => self.dispatch_wait_ext(7),
            36 => self.dispatch_wait_ext(8),
            _ => ExtCallOutcome::Skip,
        }
    }

    fn ext_sp_create(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.is_empty() {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let Some(sprites) = sprites else {
            return ExtCallOutcome::Value(0);
        };
        if let Some(old) = self.game_sprites.remove(&slot) {
            sprites.release(old);
        }
        self.game_sprite_placements.remove(&slot);
        self.game_sprite_native_scales.remove(&slot);
        self.game_sprite_wrapper_visuals.remove(&slot);
        self.game_sprite_child_offsets.remove(&slot);
        self.game_sprite_vis_clip_slots.remove(&slot);
        self.clear_pending_position_actions_for_slot(slot);
        let surface_id = sprites.allocate_surface_id();
        let pixels = vec![0u8; 4];
        let surface = match SpriteSurface::rgba8(surface_id, 1, 1, 1, pixels) {
            Ok(surface) => surface,
            Err(err) => {
                log::warn!("[trace-sprite] sp_create slot={slot} surface failed: {err}");
                return ExtCallOutcome::Value(0);
            }
        };
        sprites.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), 1, 1);
        desc.visible = false;
        desc.base_priority = game_sprite_priority(slot, self.sprite_priority_cursor);
        desc.source_name = format!("sp_create:{slot}");
        let handle = sprites.create(desc);
        self.game_sprites.insert(slot, handle);
        self.apply_pending_alpha_actions(sprites, slot, handle);
        log::debug!("[trace-sprite] sp_create slot={slot}");
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set(
        &mut self,
        arg_count: usize,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        mut sprites: Option<&mut SpriteSystem>,
        task_system: Option<&mut TaskSystem>,
        fade_replace: bool,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(arg_count);
        if args.len() < arg_count {
            return ExtCallOutcome::Block;
        }
        let name_value = args[0];
        let slot = args[1];
        let raw_x = args.get(2).copied().unwrap_or(0);
        let raw_y = args.get(3).copied().unwrap_or(0);
        let raw_z = args.get(4).copied().unwrap_or(0);
        if name_value == 0x0FFF_FFFF {
            return ExtCallOutcome::Value(1);
        }
        let Some(mut name) = self.resolve_resource_string(name_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        if name.is_empty() {
            return ExtCallOutcome::Value(1);
        }
        if is_resource_clear_sentinel(&name) {
            if let Some(state) = self.game_msprites.remove(&slot) {
                if let Some(handle) = state.handle { self.msprite_system.release(handle); }
            }
            if let Some(transition) = self.game_sprite_transitions.remove(&slot) {
                if let Some(sprites) = sprites.as_deref_mut() {
                    sprites.release_transition_handle(transition);
                }
            }
            if let (Some(sprites), Some(old)) =
                (sprites.as_deref_mut(), self.game_sprites.remove(&slot))
            {
                sprites.release(old);
            }
            self.game_sprite_placements.remove(&slot);
            self.game_sprite_native_scales.remove(&slot);
            self.game_sprite_wrapper_visuals.remove(&slot);
            self.game_sprite_child_offsets.remove(&slot);
            self.game_sprite_vis_clip_slots.remove(&slot);
            self.game_sprite_child_lanes
                .retain(|(parent, _), lane| *parent != slot && lane.child_slot != slot);
            self.game_sprite_pending_named_animations.remove(&slot);
            self.clear_alpha_lanes_for_slot(slot);
            self.clear_pending_position_actions_for_slot(slot);
            log::debug!("[trace-sprite] sp_set slot={slot} sentinel=# action=clear");
            return ExtCallOutcome::Value(1);
        }
        let (Some(resource_manager), Some(sprites)) = (resource_manager, sprites) else {
            return ExtCallOutcome::Value(0);
        };
        if is_named_animation_resource(&name) {
            match open_resource_variant(resource_manager, &name, ANIMATION_EXTENSIONS) {
                Ok(asset) => {
                    if let (Some(old_anim), Some(task_system)) =
                        (self.game_sprite_animations.remove(&slot), task_system)
                    {
                        task_system.animation_release(old_anim);
                    }
                    if let Some(handle) = self.game_sprites.get(&slot).copied() {
                        if self.apply_named_sprite_animation(sprites, handle, &asset.bytes) {
                            log::debug!(
                                "[trace-sprite] sp_set slot={slot} name={name:?} asset={:?} applied named animation to existing sprite",
                                asset.name
                            );
                            return ExtCallOutcome::Value(1);
                        }
                        log::warn!(
                            "[trace-sprite] sp_set slot={slot} name={name:?} asset={:?} unsupported named animation",
                            asset.name
                        );
                        return ExtCallOutcome::Value(0);
                    }
                    self.game_sprite_pending_named_animations.insert(
                        slot,
                        PendingNamedSpriteAnimation {
                            asset_name: asset.name.clone(),
                            bytes: asset.bytes,
                        },
                    );
                    log::debug!(
                        "[trace-sprite] sp_set slot={slot} name={name:?} queued named animation asset={:?}",
                        asset.name
                    );
                    return ExtCallOutcome::Value(1);
                }
                Err(err) => {
                    log::warn!(
                        "[trace-sprite] sp_set slot={slot} name={name:?} animation open failed: {err}"
                    );
                    return ExtCallOutcome::Value(0);
                }
            }
        }
        let mut graphic_animation_name = None;
        let mut graphic_record = None;
        if let Some(record) = assets
            .graphic_index
            .as_ref()
            .and_then(|index| index.lookup(&name))
        {
            if let Some(replacement) = record.replacement_resource() {
                log::debug!(
                    "[trace-sprite] sp_set graphic.dat key={name:?} image={replacement:?} animation={:?} flags=0x{:X} priority={} scale={} tint={:#x}",
                    record.animation_resource(),
                    record.flags,
                    record.priority_lane,
                    record.scale_percent,
                    record.alpha
                );
                name = replacement;
            }
            graphic_animation_name = record.animation_resource();
            graphic_record = Some(record.clone());
        }
        if let (Some(old_anim), Some(task_system)) =
            (self.game_sprite_animations.remove(&slot), task_system)
        {
            task_system.animation_release(old_anim);
        }
        if let Some(old_state) = self.game_msprites.remove(&slot) {
            if let Some(handle) = old_state.handle {
                self.msprite_system.release(handle);
            }
        }
        let fade_duration_ms = if fade_replace {
            self.action_state.last_duration_ms.max(250)
        } else {
            0
        };
        self.clear_alpha_lanes_for_slot(slot);
        self.game_sprite_vis_clip_slots.remove(&slot);
        if let Some(old) = self.game_sprites.remove(&slot) {
            if fade_replace {
                let _ = sprites.tween_alpha_to(old, 0, fade_duration_ms);
                self.retired_sprites.push(RetiredSprite {
                    slot,
                    handle: old,
                    release_at_ms: self.pal_time_ms.wrapping_add(fade_duration_ms),
                });
            } else {
                sprites.release(old);
            }
        }
        let loaded = registered_images::load_sprite_resource(resource_manager, &mut self.msprite_system, &name, assets.extended_softpal || assets.registered_softpal);
        let (asset, decoded, movie) = match loaded {
            Ok(asset) => asset,
            Err(err) => {
                if let Some((a, r, g, b)) = parse_solid_color_name_argb(&name) {
                    return self.create_solid_sprite(
                        sprites, slot, arg_count, raw_x, raw_y, raw_z, a, r, g, b, &name,
                    );
                }
                log::warn!("[trace-sprite] sp_set slot={slot} name={name:?} open failed: {err}");
                return ExtCallOutcome::Value(0);
            }
        };
        let (logical_width, logical_height) = self.logical_size();
        // Native placement reads the unscaled source dimensions. Oversized
        // art is intentionally cropped; only the script scale lane resizes it.
        // See ADR-0002 for the rejected height-fit approximation and evidence.
        let (native_x, native_y, native_z) = native_place_sprite(
            arg_count,
            raw_x,
            raw_y,
            raw_z,
            decoded.cell_width,
            decoded.cell_height,
            decoded.offset_x,
            decoded.offset_y,
            logical_width,
            logical_height,
        );
        let initial_visual = GameSpriteWrapperVisual {
            native_x,
            native_y,
            native_z,
            raw_scale: 100,
            width: decoded.width,
            height: decoded.height,
            arg_count,
            project_native_draw: false,
        };
        let project_native_draw = initial_visual.project_native_draw;
        let (x, y, z, scale) = if project_native_draw {
            (
                native_x,
                native_y,
                native_z,
                pal_scale_to_factor(100),
            )
        } else {
            render_from_native_wrapper(initial_visual, logical_width, logical_height)
        };
        let surface_id = sprites.allocate_surface_id();
        let surface = match SpriteSurface::rgba8(
            surface_id,
            1,
            decoded.width,
            decoded.height,
            decoded.rgba,
        ) {
            Ok(surface) => surface,
            Err(err) => {
                log::warn!("[trace-sprite] sp_set slot={slot} surface failed: {err}");
                if let Some(decoder) = movie { self.msprite_system.release(decoder); }
                return ExtCallOutcome::Value(0);
            }
        };
        sprites.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), decoded.width, decoded.height);
        desc.cell_width = decoded.cell_width;
        desc.cell_height = decoded.cell_height;
        desc.position = PalVec3::from_f32(x, y, z);
        desc.scale = scale;
        // Ordinary Game wrappers retain PAL's source-centered scaling.
        desc.center_scale = !project_native_draw;
        desc.native_projection = project_native_draw.then_some((
            logical_width.max(1) as f32 / 1920.0,
            logical_height.max(1) as f32 / 1080.0,
        ));
        desc.base_priority = game_sprite_priority(slot, self.sprite_priority_cursor);
        desc.visible = true;
        if fade_replace {
            desc.color = PalColor::from_argb(0x00FF_FFFF);
        }
        if let Some(record) = graphic_record.as_ref() {
            apply_graphic_record_lanes(&mut desc, record, fade_replace);
        }
        desc.source_name = asset.name.clone();
        if let Some(decoder) = movie {
            desc.kind = crate::sprite::SpriteKind::MSprite { decoder };
        }
        let handle = sprites.create(desc);
        if fade_replace {
            let _ = sprites.tween_alpha_to(handle, 255, fade_duration_ms);
        }
        self.game_sprites.insert(slot, handle);
        if let Some(decoder) = movie {
            self.game_msprites.insert(slot, GameMSpriteState {
                handle: Some(decoder), playing: false, locked: false, loop_mode: 0,
                loop_start: 0, loop_end: 0, last_play: 0, finished: false,
            });
        }
        self.game_sprite_native_scales.insert(slot, 100);
        self.game_sprite_base_alpha.entry(slot).or_insert(255);
        self.game_sprite_final_alpha_delta.insert(slot, 0);
        self.game_sprite_wrapper_visuals
            .insert(slot, initial_visual);
        self.game_sprite_placements.insert(
            slot,
            GameSpritePlacement {
                arg_count,
                raw_x,
                raw_y,
                raw_z,
                width: decoded.width,
                height: decoded.height,
                default_x: decoded.offset_x,
                default_y: decoded.offset_y,
            },
        );
        let aspect_type = self
            .game_sprite_aspect_position_types
            .get(&slot)
            .copied()
            .unwrap_or(0);
        self.apply_aspect_position_type(sprites, handle, aspect_type);
        if let Some(animation_name) = graphic_animation_name {
            match open_resource_variant(resource_manager, &animation_name, ANIMATION_EXTENSIONS) {
                Ok(animation_asset) => {
                    if self.apply_named_sprite_animation(sprites, handle, &animation_asset.bytes) {
                        log::debug!(
                            "[trace-sprite] sp_set slot={slot} graphic.dat animation={animation_name:?} asset={:?}",
                            animation_asset.name
                        );
                    } else {
                        log::warn!(
                            "[trace-sprite] sp_set slot={slot} graphic.dat animation={animation_name:?} unsupported"
                        );
                    }
                }
                Err(err) => {
                    log::debug!(
                        "[trace-sprite] sp_set slot={slot} graphic.dat animation={animation_name:?} open failed: {err}"
                    );
                }
            }
        }
        self.apply_pending_named_sprite_animation(sprites, slot, handle);
        self.apply_pending_alpha_actions(sprites, slot, handle);
        log::debug!(
            "[trace-sprite] sp_set slot={slot} handle={} name={name:?} asset={:?} size={}x{} raw=({}, {}, {}) pos=({}, {}, {}) priority={} fade_replace={} fade_ms={}",
            handle.0,
            asset.name,
            decoded.width,
            decoded.height,
            raw_x,
            raw_y,
            raw_z,
            x,
            y,
            z,
            z,
            fade_replace,
            fade_duration_ms
        );
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 19 (`sub_4273C0`) is named `SpSetPosEx` by one
    /// IDB table but its decompiled body logs `face_init` and writes the native
    /// face table at VM offsets +710420..+710432. Pop/display order is:
    /// face id, sprite slot, center x, center y, priority lane.
    fn ext_face_init(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let face_id = args[0];
        let sprite_slot = args[1];
        let center_x = args[2];
        let center_y = args[3];
        let priority_lane = args[4];
        self.game_face_slots.insert(
            face_id,
            GameFaceSlot {
                sprite_slot,
                center_x,
                center_y,
                priority_lane,
            },
        );
        log::debug!(
            "[trace-sprite] face_init face_id={face_id} sprite_slot={sprite_slot} center=({center_x},{center_y}) priority_lane={priority_lane}"
        );
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 20 (`sub_426E50`) pops resource, face id, dx, dy,
    /// clears the mapped old sprite, and creates the new face/part sprite using
    /// the center/slot table written by index 19. The portable renderer keeps
    /// that table explicitly so expression parts are not treated as anonymous
    /// zero-argument side effects.
    fn ext_face_set(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let name_value = args[0];
        let face_id = args[1];
        let dx = args[2];
        let dy = args[3];
        let face = self
            .game_face_slots
            .get(&face_id)
            .copied()
            .unwrap_or(GameFaceSlot {
                sprite_slot: face_id,
                center_x: -1,
                center_y: -1,
                priority_lane: 0,
            });
        let Some(sprites) = sprites else {
            return ExtCallOutcome::Value(0);
        };
        if let Some(old) = self.game_sprites.remove(&face.sprite_slot) {
            sprites.release(old);
        }
        self.game_sprite_placements.remove(&face.sprite_slot);
        self.game_sprite_native_scales.remove(&face.sprite_slot);
        self.game_sprite_wrapper_visuals.remove(&face.sprite_slot);
        self.game_sprite_child_offsets.remove(&face.sprite_slot);
        self.clear_alpha_lanes_for_slot(face.sprite_slot);
        self.clear_pending_position_actions_for_slot(face.sprite_slot);
        self.game_sprite_child_lanes.retain(|(parent, _), lane| {
            *parent != face.sprite_slot && lane.child_slot != face.sprite_slot
        });
        self.game_sprite_pending_named_animations
            .remove(&face.sprite_slot);

        if name_value == 0x0FFF_FFFF {
            log::debug!("[trace-sprite] face_set face_id={face_id} clear sentinel");
            return ExtCallOutcome::Value(1);
        }
        let Some(name) = self.resolve_resource_string(name_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        if name.is_empty() {
            return ExtCallOutcome::Value(1);
        }
        let Some(resource_manager) = resource_manager else {
            return ExtCallOutcome::Value(0);
        };
        let asset = match open_resource_variant(resource_manager, &name, IMAGE_EXTENSIONS) {
            Ok(asset) => asset,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] face_set face_id={face_id} slot={} name={name:?} open failed: {err}",
                    face.sprite_slot
                );
                return ExtCallOutcome::Value(0);
            }
        };
        let decoded = match decode_asset_image(resource_manager, &asset) {
            Ok(decoded) => decoded,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] face_set face_id={face_id} slot={} asset={:?} decode failed: {err}",
                    face.sprite_slot,
                    asset.name
                );
                return ExtCallOutcome::Value(0);
            }
        };
        let (native_x, native_y) = if face.center_x == -1 || face.center_y == -1 {
            (
                (decoded.offset_x + dx) as f32,
                (decoded.offset_y + dy) as f32,
            )
        } else {
            (
                (face.center_x as f32 * 1.5 - (decoded.cell_width as f32 * 0.5) + dx as f32),
                (face.center_y as f32 * 1.5 - decoded.cell_height as f32 + dy as f32),
            )
        };
        let z = 4999i32
            .saturating_add(self.sprite_priority_cursor)
            .saturating_add(face.priority_lane.saturating_mul(134))
            .saturating_sub(face.sprite_slot);
        let face_visual = GameSpriteWrapperVisual {
            native_x,
            native_y,
            native_z: z as f32,
            raw_scale: 100,
            width: decoded.width,
            height: decoded.height,
            arg_count: 5,
            project_native_draw: true,
        };
        let project_native_draw = face_visual.project_native_draw;
        let (x, y, _, _) = if project_native_draw {
            (native_x, native_y, z as f32, pal_scale_to_factor(100))
        } else {
            let (logical_width, logical_height) = self.logical_size();
            render_from_native_wrapper(face_visual, logical_width, logical_height)
        };
        let surface_id = sprites.allocate_surface_id();
        let surface = match SpriteSurface::rgba8(
            surface_id,
            1,
            decoded.width,
            decoded.height,
            decoded.rgba,
        ) {
            Ok(surface) => surface,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] face_set face_id={face_id} slot={} surface failed: {err}",
                    face.sprite_slot
                );
                return ExtCallOutcome::Value(0);
            }
        };
        sprites.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), decoded.width, decoded.height);
        desc.cell_width = decoded.cell_width;
        desc.cell_height = decoded.cell_height;
        desc.position = PalVec3::new(x.round() as i32, y.round() as i32, z);
        if project_native_draw {
            let (logical_width, logical_height) = self.logical_size();
            desc.native_projection = Some((
                logical_width.max(1) as f32 / 1920.0,
                logical_height.max(1) as f32 / 1080.0,
            ));
        }
        // face_set has already folded the cursor, lane and slot into `z`.
        // PalSprite adds position.z to base_priority, so compensate here to
        // keep the native depth's front-to-back direction.
        desc.base_priority = 0i32.saturating_sub(z).saturating_sub(z);
        desc.visible = true;
        desc.source_name = asset.name.clone();
        let face_alpha =
            self.computed_game_sprite_alpha_for_new_wrapper_sprite(face.sprite_slot, 255);
        desc.color =
            PalColor::from_argb((u32::from(face_alpha) << 24) | (desc.color.0 & 0x00FF_FFFF));
        let handle = sprites.create(desc);
        self.game_sprites.insert(face.sprite_slot, handle);
        self.game_sprite_native_scales.insert(face.sprite_slot, 100);
        self.game_sprite_wrapper_visuals
            .insert(face.sprite_slot, face_visual);
        self.game_sprite_base_alpha
            .entry(face.sprite_slot)
            .or_insert(255);
        self.game_sprite_final_alpha_delta
            .insert(face.sprite_slot, 0);
        self.submit_game_sprite_alpha(sprites, face.sprite_slot, handle);
        self.apply_pending_alpha_actions(sprites, face.sprite_slot, handle);
        self.game_sprite_placements.insert(
            face.sprite_slot,
            GameSpritePlacement {
                arg_count: 5,
                raw_x: native_x.round() as i32,
                raw_y: native_y.round() as i32,
                raw_z: z,
                width: decoded.width,
                height: decoded.height,
                default_x: decoded.offset_x,
                default_y: decoded.offset_y,
            },
        );
        let _ = self.commit_game_sprite_wrapper_visual(sprites, face.sprite_slot, handle);
        log::debug!(
            "[trace-sprite] face_set face_id={face_id} slot={} name={name:?} asset={:?} dxdy=({},{}) native=({native_x:.1},{native_y:.1},{z}) logical=({x:.1},{y:.1},{z})",
            face.sprite_slot,
            asset.name,
            args[2],
            args[3]
        );
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 23 (`sub_426DB0`) pops a face id, resolves the
    /// sprite slot through the face table, and releases that sprite.
    fn ext_face_cls(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(face_id) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let sprite_slot = self
            .game_face_slots
            .get(&face_id)
            .map(|face| face.sprite_slot)
            .unwrap_or(face_id);
        if let Some(handle) = self.game_sprites.remove(&sprite_slot) {
            if let Some(sprites) = sprites {
                sprites.release(handle);
            }
        }
        self.game_sprite_placements.remove(&sprite_slot);
        self.game_sprite_native_scales.remove(&sprite_slot);
        self.game_sprite_wrapper_visuals.remove(&sprite_slot);
        self.game_sprite_child_offsets.remove(&sprite_slot);
        self.game_sprite_vis_clip_slots.remove(&sprite_slot);
        self.game_sprite_child_lanes
            .retain(|(parent, _), lane| *parent != sprite_slot && lane.child_slot != sprite_slot);
        self.game_sprite_pending_named_animations
            .remove(&sprite_slot);
        self.clear_alpha_lanes_for_slot(sprite_slot);
        self.clear_pending_position_actions_for_slot(sprite_slot);
        log::debug!("[trace-sprite] face_cls face_id={face_id} sprite_slot={sprite_slot}");
        ExtCallOutcome::Value(1)
    }

    fn create_solid_sprite(
        &mut self,
        sprites: &mut SpriteSystem,
        slot: i32,
        arg_count: usize,
        raw_x: i32,
        raw_y: i32,
        raw_z: i32,
        a: u8,
        r: u8,
        g: u8,
        b: u8,
        source_name: &str,
    ) -> ExtCallOutcome {
        let (logical_width, logical_height) = self.logical_size();
        // Native synthetic color resources (`#AARRGGBB`, BK_BLACK/BK_WHITE)
        // are PAL-side solid surfaces.  Absolute negative
        // placement is used for transition masks such as `#FFFFFFFF` at
        // (-200,-104); the backing surface must expand by that signed offset or
        // the right/bottom edge leaks the clear color during fades.  Keep
        // expansion bounded and only apply it to sane negative absolute values
        // so sentinel coordinates like 0x0fffffff cannot allocate huge masks.
        let width = expanded_solid_extent(logical_width, raw_x);
        let height = expanded_solid_extent(logical_height, raw_y);
        let (x, y, z) = place_sprite(
            arg_count,
            raw_x,
            raw_y,
            raw_z,
            width,
            height,
            0,
            0,
            logical_width,
            logical_height,
        );
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        for px in pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&[r, g, b, a]);
        }
        let surface_id = sprites.allocate_surface_id();
        let surface = match SpriteSurface::rgba8(surface_id, 1, width, height, pixels) {
            Ok(surface) => surface,
            Err(err) => {
                log::warn!("[trace-sprite] sp_set slot={slot} solid surface failed: {err}");
                return ExtCallOutcome::Value(0);
            }
        };
        sprites.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), width, height);
        // `BK_BLACK` and `BK_WHITE` are compatibility stand-ins
        // for PAL-side full-screen mask resources that are absent from the
        // testcase archive.  Native scripts still drive their wrapper with
        // sprite scale/position extcalls, but the portable renderer must keep
        // the generated solid texture as a viewport-covering layer instead of
        // sending it through the ordinary bitmap square-scale work-buffer path.
        if is_fullscreen_solid_layer(source_name) {
            desc.kind = SpriteKind::SolidLayer;
        }
        desc.position = PalVec3::new(x, y, z);
        desc.center_scale = true;
        desc.base_priority = game_sprite_priority(slot, self.sprite_priority_cursor);
        desc.visible = true;
        desc.source_name = source_name.to_owned();
        let handle = sprites.create(desc);
        self.game_sprites.insert(slot, handle);
        self.game_sprite_native_scales.insert(slot, 100);
        self.game_sprite_base_alpha.entry(slot).or_insert(255);
        self.game_sprite_final_alpha_delta.insert(slot, 0);
        let (mut native_x, mut native_y, native_z) = native_place_sprite(
            arg_count,
            raw_x,
            raw_y,
            raw_z,
            width,
            height,
            0,
            0,
            logical_width,
            logical_height,
        );
        if arg_count < 5 && height <= logical_height && raw_x != 0xFFFF && raw_y != 0xFFFF {
            native_x = unproject_script_x(x, logical_width);
            native_y = unproject_script_y(y, logical_height);
        }
        self.game_sprite_wrapper_visuals.insert(
            slot,
            GameSpriteWrapperVisual {
                native_x,
                native_y,
                native_z,
                raw_scale: 100,
                width,
                height,
                arg_count,
                project_native_draw: false,
            },
        );
        self.game_sprite_placements.insert(
            slot,
            GameSpritePlacement {
                arg_count,
                raw_x,
                raw_y,
                raw_z,
                width,
                height,
                default_x: 0,
                default_y: 0,
            },
        );
        let aspect_type = self
            .game_sprite_aspect_position_types
            .get(&slot)
            .copied()
            .unwrap_or(0);
        self.apply_aspect_position_type(sprites, handle, aspect_type);
        self.apply_pending_named_sprite_animation(sprites, slot, handle);
        self.apply_pending_alpha_actions(sprites, slot, handle);
        log::debug!(
            "[trace-sprite] sp_set slot={slot} handle={} name={source_name:?} solid=rgb({r},{g},{b}) size={}x{} raw=({}, {}, {}) pos=({}, {}, {}) priority={}",
            handle.0,
            width,
            height,
            raw_x,
            raw_y,
            raw_z,
            x,
            y,
            z,
            z
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_text(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let text_value = args[1];
        let x = args[2];
        let y = args[3];
        let z = args[4];
        if !(-1..=128).contains(&slot) {
            log::warn!("[trace-sprite] sptext slot out of range: {slot}");
            return ExtCallOutcome::Value(0);
        }
        let Some(text) = self
            .resolve_script_string(text_value, assets, nls)
            .or_else(|| self.resolve_resource_string(text_value, assets, nls))
        else {
            return ExtCallOutcome::Value(0);
        };
        let Some(sprites) = sprites else {
            return ExtCallOutcome::Value(0);
        };
        let (text, temporary_size) = parse_pal_text_directives(&text);
        if text.is_empty() {
            return ExtCallOutcome::Value(0);
        }
        let saved_size = self.font_state.font_size();
        if let Some(size) = temporary_size {
            self.font_state.set_font_size(size);
        }
        let (width, height, rgba) = self.font_state.rasterize(&text);
        if temporary_size.is_some() {
            self.font_state.set_font_size(saved_size);
        }
        if let Some(old_state) = self.game_msprites.remove(&slot) {
            if let Some(handle) = old_state.handle {
                self.msprite_system.release(handle);
            }
        }
        if let Some(handle) = self.game_sprites.get(&slot).copied() {
            if !sprites.replace_sprite_surface(handle, width, height, rgba, format!("text:{text}"))
            {
                return ExtCallOutcome::Value(0);
            }
            sprites.set_pos(handle, x, y, z);
            sprites.set_priority(
                handle,
                game_sprite_priority(slot, self.sprite_priority_cursor),
            );
            self.apply_pending_alpha_actions(sprites, slot, handle);
        } else {
            let Some(handle) = sprites.create_rgba_sprite(
                width,
                height,
                rgba,
                PalVec3::new(x, y, z),
                game_sprite_priority(slot, self.sprite_priority_cursor),
                format!("text:{text}"),
            ) else {
                return ExtCallOutcome::Value(0);
            };
            self.game_sprites.insert(slot, handle);
            self.apply_pending_alpha_actions(sprites, slot, handle);
        }
        self.text_state.last_text_value = text_value;
        self.text_state.last_event_time_ms = self.pal_time_ms;
        log::debug!(
            "[trace-sprite] sptext slot={slot} text_value={text_value} size={}x{} pos=({}, {}, {})",
            width,
            height,
            x,
            y,
            z
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_anim(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
        task_system: Option<&mut TaskSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let name_value = args[1];
        let entry_flag = args[2];
        let name = self
            .resolve_resource_string(name_value, assets, nls)
            .unwrap_or_else(|| format!("0x{name_value:08X}"));

        let Some(handle) = self.game_sprites.get(&slot).copied() else {
            log::debug!("[trace-sprite] sp_set_anim slot={slot} name={name:?} missing sprite");
            return ExtCallOutcome::Value(1);
        };
        let (Some(sprites), Some(task_system)) = (sprites, task_system) else {
            return ExtCallOutcome::Value(1);
        };

        if let Some(old_anim) = self.game_sprite_animations.remove(&slot) {
            task_system.animation_release(old_anim);
        }

        if let Some(resource_manager) = resource_manager {
            match open_resource_variant(resource_manager, &name, ANIMATION_EXTENSIONS) {
                Ok(asset) => {
                    if self.apply_named_sprite_animation(sprites, handle, &asset.bytes) {
                        log::debug!(
                            "[trace-sprite] sp_set_anim slot={slot} name={name:?} asset={:?} named entry_flag={entry_flag}",
                            asset.name
                        );
                        return ExtCallOutcome::Value(1);
                    }
                    log::debug!(
                        "[trace-sprite] sp_set_anim slot={slot} name={name:?} asset={:?} unsupported named animation",
                        asset.name
                    );
                }
                Err(err) => {
                    log::debug!(
                        "[trace-sprite] sp_set_anim slot={slot} name={name:?} animation open failed: {err}"
                    );
                }
            }
        }

        let flags = match sprites.get(handle) {
            Some(sprite) if sprite.frame_count(crate::sprite::PalAnimationAxis::Horizontal) > 1 => {
                PalAnimationFlags::HORIZONTAL
            }
            Some(sprite) if sprite.frame_count(crate::sprite::PalAnimationAxis::Vertical) > 1 => {
                PalAnimationFlags::VERTICAL
            }
            Some(_) => {
                log::debug!(
                    "[trace-sprite] sp_set_anim slot={slot} name={name:?} single-frame sprite"
                );
                return ExtCallOutcome::Value(1);
            }
            None => return ExtCallOutcome::Value(1),
        };

        let desc = PalSheetAnimationDesc {
            sprite: handle,
            flags,
            frame_delay_ms: 100,
            running: true,
        };
        match task_system.create_animation_sheet(sprites, desc, None) {
            Ok(anim_handle) => {
                self.game_sprite_animations.insert(slot, anim_handle);
                log::debug!(
                    "[trace-sprite] sp_set_anim slot={slot} name={name:?} entry_flag={entry_flag} flags=0x{:02X} handle=0x{:08X}",
                    flags.raw(),
                    anim_handle.0
                );
                ExtCallOutcome::Value(1)
            }
            Err(err) => {
                log::warn!("[trace-sprite] sp_set_anim slot={slot} name={name:?} failed: {err}");
                ExtCallOutcome::Value(0)
            }
        }
    }

    fn apply_named_sprite_animation(
        &self,
        sprites: &mut SpriteSystem,
        handle: SpriteHandle,
        bytes: &[u8],
    ) -> bool {
        let Some(anim) = parse_named_sprite_animation(bytes) else {
            return false;
        };
        match anim {
            NamedSpriteAnimation::MoveBy {
                dx,
                dy,
                dz,
                duration_ms,
            } => sprites.tween_pos_by(handle, dx as f32, dy as f32, dz as f32, duration_ms),
            NamedSpriteAnimation::ScaleBy {
                delta_percent,
                duration_ms,
            } => {
                let Some(sprite) = sprites.get(handle) else {
                    return false;
                };
                let to = (sprite.scale + delta_percent as f32 / 100.0).max(0.01);
                sprites.tween_scale_to(handle, to, duration_ms)
            }
            NamedSpriteAnimation::AlphaBy { delta, duration_ms } => {
                let Some(sprite) = sprites.get(handle) else {
                    return false;
                };
                let alpha = i32::from(sprite.color.alpha())
                    .saturating_add(delta)
                    .clamp(0, 255) as u8;
                sprites.tween_alpha_to(handle, alpha, duration_ms)
            }
        }
    }

    fn ext_sp_wait_draw(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let frames = args.first().copied().unwrap_or(0).max(0);
        log::debug!("[trace-sprite] sp_wait_draw frames={frames}");
        if frames == 0 {
            ExtCallOutcome::Value(1)
        } else {
            ExtCallOutcome::Wait {
                value: 1,
                request: WaitRequest::Frame(frames),
            }
        }
    }

    /// Game category 3 index 44 (`sub_424FD0`) pops one sprite slot and sets
    /// native wrapper bit `0x01000000` when the slot has a PAL sprite. The bit
    /// survives until the sprite slot is cleared/replaced and marks sprites
    /// whose visible region must follow PAL's wrapper submit path.
    fn ext_sp_set_vis_clip(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let active = self.game_sprites.contains_key(&slot);
        if active {
            self.game_sprite_vis_clip_slots.insert(slot);
        }
        log::debug!("[trace-sprite] sp_set_vis_clip slot={slot} active={active}");
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 54 (`sub_41A680` in this Koikake build) pops a
    /// sprite slot and calls `PalSpriteBackBafferCopy` on that sprite's surface.
    /// The copy is kept as its own sprite so the pixels survive after the source
    /// slot is replaced, and it is drawn behind later sprites.
    fn ext_sp_get_backbuffer(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        if !(-1..=0x80).contains(&slot) {
            log::debug!("[trace-sprite] get_backbuffer slot={slot} out of range");
            return ExtCallOutcome::Value(0);
        }
        let Some(sprites) = sprites else {
            return ExtCallOutcome::Value(1);
        };
        let Some(source) = self.game_sprites.get(&slot).copied() else {
            log::debug!("[trace-sprite] get_backbuffer slot={slot} empty");
            return ExtCallOutcome::Value(1);
        };
        let Some(copied) = sprites.copy_sprite_pixels(source, i32::MIN / 2, "backbuffer") else {
            log::debug!("[trace-sprite] get_backbuffer slot={slot} copy failed");
            return ExtCallOutcome::Value(1);
        };
        if let Some(previous) = self.backbuffer_sprite.replace(copied) {
            sprites.release(previous);
        }
        log::debug!("[trace-sprite] get_backbuffer slot={slot} copied");
        ExtCallOutcome::Value(1)
    }

    fn ext_msp_lock(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.is_empty() {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        if let Some(state) = self.game_msprites.get_mut(&slot) {
            state.locked = true;
            if let Some(handle) = state.handle {
                self.msprite_system.lock(handle);
            }
        }
        log::debug!("[trace-msprite] msp_lock slot={slot}");
        ExtCallOutcome::Continue
    }

    fn ext_msp_unlock(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.is_empty() {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        if let Some(state) = self.game_msprites.get_mut(&slot) {
            state.locked = false;
            if let Some(handle) = state.handle {
                self.msprite_system.unlock(handle);
            }
        }
        log::debug!("[trace-msprite] msp_unlock slot={slot}");
        ExtCallOutcome::Continue
    }

    fn ext_msp_play(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let play = args[1];
        if let Some(state) = self.game_msprites.get_mut(&slot) {
            state.playing = true;
            state.finished = false;
            state.last_play = play;
            if let Some(handle) = state.handle {
                self.msprite_system.play(handle, play);
            }
        }
        log::debug!("[trace-msprite] msp_play slot={slot} play={play}");
        ExtCallOutcome::Continue
    }

    fn ext_msp_stop(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.is_empty() {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        if let Some(state) = self.game_msprites.get_mut(&slot) {
            state.playing = false;
            state.finished = false;
            if let Some(handle) = state.handle {
                self.msprite_system.stop(handle);
            }
        }
        log::debug!("[trace-msprite] msp_stop slot={slot}");
        ExtCallOutcome::Continue
    }

    /// `sp_cls_ex(first, count)` clears a consecutive range of sprite slots.
    /// The confirmation popup creates its canvas and frame in adjacent slots.
    fn ext_sp_cls_ex(
        &mut self,
        mut sprites: Option<&mut SpriteSystem>,
        mut task_system: Option<&mut TaskSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let first = args[0];
        let count = args[1].clamp(0, 1000);
        if first == -1 {
            self.stack.push(-1);
            return self.ext_sp_cls(sprites, task_system);
        }
        for offset in 0..count {
            let Some(slot) = first.checked_add(offset) else {
                break;
            };
            self.stack.push(slot);
            let _ = self.ext_sp_cls(sprites.as_deref_mut(), task_system.as_deref_mut());
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_cls(
        &mut self,
        sprites: Option<&mut SpriteSystem>,
        task_system: Option<&mut TaskSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let slot = args.first().copied().unwrap_or(-1);
        if let Some(task_system) = task_system {
            if slot == -1 {
                let handles = self
                    .game_sprite_animations
                    .values()
                    .copied()
                    .collect::<Vec<_>>();
                self.game_sprite_animations.clear();
                for handle in handles {
                    task_system.animation_release(handle);
                }
            } else if let Some(handle) = self.game_sprite_animations.remove(&slot) {
                task_system.animation_release(handle);
            }
        } else if slot == -1 {
            self.game_sprite_animations.clear();
        } else {
            self.game_sprite_animations.remove(&slot);
        }
        if slot == -1 {
            for state in self.game_msprites.values() {
                if let Some(handle) = state.handle { self.msprite_system.release(handle); }
            }
            self.game_msprites.clear();
            // The priority cursor is sprite-system state; a full clear resets
            // it or screens abandoned via script long jumps (e.g. Koikake's
            // save-screen -> title return) leave later sprites on stale lanes.
            self.sprite_priority_cursor = 0;
            self.game_sprite_placements.clear();
            self.game_sprite_native_scales.clear();
            self.game_sprite_wrapper_visuals.clear();
            self.game_sprite_aspect_position_types.clear();
            self.game_sprite_vis_clip_slots.clear();
            self.game_sprite_child_lanes.clear();
            self.game_sprite_child_offsets.clear();
            self.game_sprite_render_lanes.clear();
            self.game_sprite_pending_named_animations.clear();
            self.clear_alpha_lanes_for_slot(-1);
            self.game_sprite_pending_position.clear();
        } else {
            self.game_sprite_pending_named_animations.remove(&slot);
            self.game_sprite_placements.remove(&slot);
            self.game_sprite_native_scales.remove(&slot);
            self.game_sprite_wrapper_visuals.remove(&slot);
            self.game_sprite_child_offsets.remove(&slot);
            self.game_sprite_aspect_position_types.remove(&slot);
            self.game_sprite_vis_clip_slots.remove(&slot);
            self.game_sprite_child_lanes
                .retain(|(parent, _), lane| *parent != slot && lane.child_slot != slot);
            if let Some(old_state) = self.game_msprites.remove(&slot) {
                if let Some(handle) = old_state.handle {
                    self.msprite_system.release(handle);
                }
            }
            self.clear_alpha_lanes_for_slot(slot);
            self.clear_pending_position_actions_for_slot(slot);
        }
        let Some(sprites) = sprites else {
            if slot == -1 {
                self.game_sprite_transitions.clear();
                self.game_sprite_transition_sources.clear();
                self.retired_sprites.clear();
            } else {
                self.game_sprite_transitions.remove(&slot);
                self.game_sprite_transition_sources.remove(&slot);
                self.retired_sprites.retain(|retired| retired.slot != slot);
            }
            return ExtCallOutcome::Value(1);
        };
        self.clear_save_drawings(sprites, slot);
        if slot == -1 {
            let transitions = self
                .game_sprite_transitions
                .values()
                .copied()
                .collect::<Vec<_>>();
            self.game_sprite_transitions.clear();
            for transition in transitions {
                sprites.release_transition_handle(transition);
            }
            self.game_sprite_pending_position.clear();
            let handles = self.game_sprites.values().copied().collect::<Vec<_>>();
            self.game_sprites.clear();
            for handle in handles {
                sprites.release(handle);
            }
            let transition_sources = self
                .game_sprite_transition_sources
                .values()
                .copied()
                .collect::<Vec<_>>();
            self.game_sprite_transition_sources.clear();
            for handle in transition_sources {
                sprites.release(handle);
            }
            let retired = self
                .retired_sprites
                .drain(..)
                .map(|entry| entry.handle)
                .collect::<Vec<_>>();
            for handle in retired {
                sprites.release(handle);
            }
        } else {
            if let Some(transition) = self.game_sprite_transitions.remove(&slot) {
                sprites.release_transition_handle(transition);
            }
            if let Some(handle) = self.game_sprites.remove(&slot) {
                sprites.release(handle);
            }
            if let Some(source) = self.game_sprite_transition_sources.remove(&slot) {
                sprites.release(source);
            }
            self.release_retired_sprites_for_slot(slot, sprites);
            self.clear_pending_position_actions_for_slot(slot);
        }
        log::debug!("[trace-sprite] sp_cls slot={slot}");
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_alpha(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let (Some(slot), Some(alpha)) = (args.first().copied(), args.get(1).copied()) else {
            return ExtCallOutcome::Block;
        };
        let Some((target_slot, encoded_layer)) = decode_pal_sprite_slot(slot) else {
            log::debug!("[trace-sprite] sp_set_alpha unsupported slot={slot} alpha={alpha}");
            return ExtCallOutcome::Value(1);
        };
        if let Some(sprites) = sprites {
            if let Some(layer) = encoded_layer {
                let alpha = alpha.clamp(0, 255) as u8;
                if self.apply_encoded_button_alpha_to(sprites, layer, target_slot, alpha, 0) {
                    log::debug!(
                        "[trace-sprite] sp_set_alpha slot={slot:#010X} layer={layer} button_index={target_slot} alpha={alpha}"
                    );
                } else {
                    log::debug!(
                        "[trace-sprite] sp_set_alpha slot={slot:#010X} layer={layer} missing native target index={target_slot} alpha={alpha}"
                    );
                }
                return ExtCallOutcome::Value(1);
            }
            if let Some(handle) = self.game_sprites.get(&target_slot).copied() {
                let alpha = alpha.clamp(0, 255);
                // Game.exe sub_4281D0 writes the wrapper base alpha byte and
                // clears the running action alpha lane.  It does not preserve a
                // previous tween destination; later action sections add their
                // temporary/final deltas on top of this base.
                self.game_sprite_base_alpha.insert(target_slot, alpha);
                self.game_sprite_active_alpha
                    .retain(|action| action.slot != target_slot);
                self.submit_game_sprite_alpha(sprites, target_slot, handle);
                let submitted = self.computed_game_sprite_alpha(target_slot, handle);
                log::debug!(
                    "[trace-sprite] sp_set_alpha slot={slot} target_slot={target_slot} layer={encoded_layer:?} base_alpha={alpha} submitted_alpha={submitted}"
                );
            } else {
                // Game.exe sub_4281D0 resolves the wrapper through sub_449120;
                // if it returns -1, the handler does not queue alpha for a
                // future sprite.  The previous pending-alpha behavior leaked
                // stale opacity into later helper/menu sprites and produced
                // full-screen flashes when scripts toggled absent slots.
                log::debug!(
                    "[trace-sprite] sp_set_alpha missing slot={slot} target_slot={target_slot} alpha={alpha} no_queue_native"
                );
            }
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_color(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let (Some(slot), Some(rgb)) = (args.first().copied(), args.get(1).copied()) else {
            return ExtCallOutcome::Block;
        };
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            if let Some(sprite) = sprites.get_mut(handle) {
                let raw = rgb as u32;
                let alpha = sprite.color.0 & 0xFF00_0000;
                sprite.color = PalColor::from_argb(alpha | (raw & 0x00FF_FFFF));
            }
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_get_color(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let value = if let (Some(handle), Some(sprites)) =
            (self.game_sprites.get(&slot).copied(), sprites)
        {
            sprites
                .get(handle)
                .map(|sprite| (sprite.color.0 & 0x00FF_FFFF) as i32)
                .unwrap_or(-1)
        } else {
            -1
        };
        ExtCallOutcome::Value(value)
    }

    fn ext_sp_get_alpha(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let value = if let (Some(handle), Some(sprites)) =
            (self.game_sprites.get(&slot).copied(), sprites)
        {
            sprites
                .get(handle)
                .map(|sprite| sprite.info().color.alpha() as i32)
                .unwrap_or(-1)
        } else {
            -1
        };
        ExtCallOutcome::Value(value)
    }

    fn ext_sp_get_rotate(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let (Some(slot), Some(axis)) = (args.first().copied(), args.get(1).copied()) else {
            return ExtCallOutcome::Block;
        };
        let value = if let (Some(handle), Some(sprites)) =
            (self.game_sprites.get(&slot).copied(), sprites)
        {
            sprites
                .get(handle)
                .map(|sprite| match axis {
                    0 => sprite.info().rotation.x as i32,
                    1 => sprite.info().rotation.y as i32,
                    2 => sprite.info().rotation.z as i32,
                    _ => slot,
                })
                .unwrap_or(-1)
        } else {
            -1
        };
        ExtCallOutcome::Value(value)
    }

    fn ext_sp_view_ctrl(
        &mut self,
        sprites: Option<&mut SpriteSystem>,
        visible: bool,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            sprites.view_ctrl(handle, visible);
        }
        log::debug!(
            "[trace-sprite] sp_{} slot={slot}",
            if visible { "show" } else { "hide" }
        );
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 7 (`sub_427780`) advances the VM sprite priority
    /// cursor by `134 * lane`; it does not take a sprite slot.
    fn ext_sp_set_priority_lane(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(lane) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let next = self
            .sprite_priority_cursor
            .saturating_add(lane.saturating_mul(134));
        self.sprite_priority_cursor = next.min(5000);
        log::debug!(
            "[trace-sprite] set_priority lane={lane} cursor={}",
            self.sprite_priority_cursor
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_center(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        let (Some(slot), Some(x), Some(y)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            sprites.set_center_offset(handle, x, y);
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_pos_ex(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        let (Some(slot), Some(x), Some(y), Some(z)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
            args.get(3).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        log::debug!(
            "[trace-sprite] sp_set_pos pc=0x{:08X} slot={slot} pos=({x},{y},{z})",
            self.pc
        );
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            sprites.set_pos(handle, x, y, z);
        }
        ExtCallOutcome::Value(1)
    }

    /// Category 3 index 14 is still Blocked.  The latest name table calls it
    /// `SpMoveEx`, but Game.sqlite handler `sub_427E90` writes the wrapper
    /// rotation lanes, not x/y/z position (`wrapper[10..12]` and float lanes
    /// `33..35`).  Applying it as movement corrupts scene placement, so the
    /// portable runtime keeps the observable stack discipline and records the
    /// values until the dispatch-table/name conflict is resolved against the IDB.
    fn ext_sp_move_ex(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        let (Some(slot), Some(x), Some(y), Some(z)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
            args.get(3).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        log::debug!(
            "[trace-sprite] sp_move_ex/rotate-conflict pc=0x{:08X} slot={slot} args=({x},{y},{z})",
            self.pc
        );
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            sprites.set_rotate_ex(handle, x as f32, y as f32, z as f32);
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_pos_move(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        let (Some(slot), Some(x), Some(y), Some(z)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
            args.get(3).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        // Newer Game.exe 0x44B900 updates the resolved wrapper's integer and
        // float base coordinates directly. Packed button references use the
        // same path, including while the button is still fully transparent.
        if self.input_registered_profile {
            if let Some(key) = Self::packed_button_key(slot) {
                if let (Some(entry), Some(sprites)) = (self.game_buttons.get_mut(&key), sprites) {
                    sprites.move_pos(entry.handle, x as f32, y as f32, z as f32);
                    entry.base_position[0] = entry.base_position[0].wrapping_add(x);
                    entry.base_position[1] = entry.base_position[1].wrapping_add(y);
                }
                return ExtCallOutcome::Continue;
            }
        }
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            // Game.exe sub_428640 (`sp_set_pos_move`) is an immediate relative
            // wrapper-position update, not an action-duration tween.  Ordinary
            // script-space deltas use normalized wrapper position units.
            // Art dimensions and script scale do not modify these deltas.
            // Keep wrapper state authoritative so later scale/transition calls
            // do not accumulate already-projected logical coordinates.
            let mut used_wrapper = false;
            if let Some(visual) = self.game_sprite_wrapper_visuals.get_mut(&slot) {
                let (native_dx, native_dy) = native_delta_for_visual(*visual, x, y);
                visual.native_x += native_dx;
                visual.native_y += native_dy;
                visual.native_z += z as f32;
                used_wrapper = true;
            }
            if used_wrapper {
                let _ = self.commit_game_sprite_wrapper_visual(sprites, slot, handle);
            } else {
                let (logical_width, logical_height) = self.logical_size();
                let dx = scale_script_x(x, logical_width) as f32;
                let dy = scale_script_y(y, logical_height) as f32;
                let dz = z as f32;
                let _ = sprites.move_pos(handle, dx, dy, dz);
            }
            log::debug!(
                "[trace-sprite] sp_set_pos_move slot={slot} raw_delta=({x},{y},{z}) wrapper={used_wrapper}"
            );
        }
        ExtCallOutcome::Value(1)
    }

    /// Game.exe sub_4246C0 (`set_aspect_position_type`) pops the sprite slot first
    /// and then a type-table index, looks up the native sprite wrapper, and writes
    /// `xmmword_4A3FC0[type_index]` into wrapper field 26.  PAL later passes that
    /// field to `PalSpriteConvertAspectPosition` from the shared `sub_4494D0`
    /// commit path.  The 2015 build's table at VA 0x4A3FC0 is `[0, 1, 2, 3]`.
    fn ext_sp_set_aspect_position_type(
        &mut self,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let (Some(slot), Some(type_index)) = (args.first().copied(), args.get(1).copied()) else {
            return ExtCallOutcome::Block;
        };
        let aspect_type = match type_index {
            0..=3 => type_index,
            _ => 0,
        };
        self.game_sprite_aspect_position_types
            .insert(slot, aspect_type);
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            self.apply_aspect_position_type(sprites, handle, aspect_type);
        }
        log::debug!(
            "[trace-sprite] set_aspect_position_type pc=0x{:08X} slot={slot} type_index={type_index} aspect_type={aspect_type}",
            self.pc
        );
        ExtCallOutcome::Value(1)
    }

    /// Portable equivalent of PAL.dll `PalSpriteConvertAspectPosition`.
    ///
    /// PAL.dll 0x1025B770 only mutates sprite x/y when aspect-position is
    /// enabled and aspect mode is 3. It compares the active content rectangle
    /// against the base logical rectangle, then shifts or scales position based
    /// on the per-sprite type: 1 = x+y, 2 = x only, 3 = y only, 4 = scale around
    /// stage center. The native submit path (`sub_4494D0`) calls this after
    /// every wrapper position commit, not just on sprite creation.
    fn apply_aspect_position_type(
        &self,
        sprites: &mut SpriteSystem,
        handle: SpriteHandle,
        aspect_type: i32,
    ) {
        let Some(pos) = sprites.position(handle) else {
            return;
        };
        let Some((x, y)) = self.convert_aspect_position(pos.x, pos.y, aspect_type) else {
            return;
        };
        let _ = sprites.set_pos_float(handle, x, y, pos.z);
    }

    fn convert_aspect_position(&self, x: f32, y: f32, aspect_type: i32) -> Option<(f32, f32)> {
        if aspect_type == 0
            || self.system_state.aspect_position_enabled() == 0
            || self.system_state.aspect_mode() != 3
        {
            return None;
        }
        let (logical_width, logical_height) = self.logical_size();
        let rect = self.system_state.active_content_rect();
        let base_w = logical_width.max(1) as f32;
        let base_h = logical_height.max(1) as f32;
        let scale_x = rect.width() as f32 / base_w;
        let scale_y = rect.height() as f32 / base_h;
        let half_w = base_w * 0.5;
        let half_h = base_h * 0.5;
        let shift_x = half_w - half_w * scale_x;
        let shift_y = half_h - half_h * scale_y;
        let mut adjusted_x = x;
        let mut adjusted_y = y;
        match aspect_type {
            1 | 2 | 3 => {
                if aspect_type == 1 || aspect_type == 2 {
                    adjusted_x = if x <= half_w {
                        x + shift_x
                    } else {
                        x - shift_x
                    };
                }
                if aspect_type == 1 || aspect_type == 3 {
                    adjusted_y = y - shift_y;
                }
            }
            4 => {
                adjusted_x = (x - half_w) * scale_x + half_w;
                adjusted_y = (y - half_h) * scale_y + half_h;
            }
            _ => return None,
        }
        log::trace!(
            "[trace-sprite] aspect_convert type={aspect_type} mode={} enabled={} rect=({},{}..{},{}), in=({x:.1},{y:.1}) out=({adjusted_x:.1},{adjusted_y:.1})",
            self.system_state.aspect_mode(),
            self.system_state.aspect_position_enabled(),
            rect.left,
            rect.top,
            rect.right,
            rect.bottom
        );
        Some((adjusted_x, adjusted_y))
    }

    fn ext_sp_set_rect_pos(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        let (Some(slot), Some(cell_x), Some(cell_y)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            sprites.rect_set_pos(handle, cell_x, cell_y);
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_rect(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        let (Some(slot), Some(left), Some(top), Some(right), Some(bottom)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
            args.get(3).copied(),
            args.get(4).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            sprites.set_rect(handle, Some(PalRect::new(left, top, right, bottom)));
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_scale(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let (Some(slot), Some(scale)) = (args.first().copied(), args.get(1).copied()) else {
            return ExtCallOutcome::Block;
        };
        self.game_sprite_native_scales.insert(slot, scale);
        if let Some(visual) = self.game_sprite_wrapper_visuals.get_mut(&slot) {
            visual.raw_scale = scale;
        }
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            let raw_scale = scale;
            // VmExtcall_SpSetScale @ 0x428000 pops (slot, raw_scale) and stores
            // raw_scale / 100.0 into the Game sprite wrapper. It does not mutate
            // x/y. Native PAL later centers the scaled source rect during draw,
            // so placement coordinates must not be recomputed here; doing that
            // double-applied the center-scale compensation and lifted tall
            // standing sprites above the ADV window.
            let scale_factor = self.sprite_pal_scale_factor(slot, raw_scale);
            if let Some(placement) = self.game_sprite_placements.get(&slot) {
                log::debug!(
                    "[trace-sprite] sp_set_scale keeps placement slot={slot} mode_args={} raw=({},{},{})",
                    placement.arg_count,
                    placement.raw_x,
                    placement.raw_y,
                    placement.raw_z
                );
            }
            if self.game_sprite_wrapper_visuals.contains_key(&slot) {
                let _ = self.commit_game_sprite_wrapper_visual(sprites, slot, handle);
            } else {
                sprites.set_scale(handle, scale_factor);
            }
            log::debug!(
                "[trace-sprite] sp_set_scale slot={slot} raw_scale={raw_scale} scale={scale_factor:.3}"
            );
        }
        ExtCallOutcome::Value(1)
    }

    fn sprite_pal_scale_factor(&self, slot: i32, raw_scale: i32) -> f32 {
        let (logical_width, logical_height) = self.logical_size();
        let arg_count = self
            .game_sprite_placements
            .get(&slot)
            .map(|placement| placement.arg_count)
            .unwrap_or(5);
        Self::sprite_pal_scale_factor_for_placement(
            arg_count,
            raw_scale,
            logical_width,
            logical_height,
        )
    }

    fn commit_game_sprite_wrapper_visual(
        &self,
        sprites: &mut SpriteSystem,
        slot: i32,
        handle: SpriteHandle,
    ) -> bool {
        let Some(visual) = self.game_sprite_wrapper_visuals.get(&slot).copied() else {
            return false;
        };
        let (logical_width, logical_height) = self.logical_size();
        let temp = self.active_position_delta_for_slot(slot, handle);
        let visual = GameSpriteWrapperVisual {
            native_x: visual.native_x + temp.0,
            native_y: visual.native_y + temp.1,
            native_z: visual.native_z + temp.2,
            ..visual
        };
        let project_native_draw = visual.project_native_draw;
        let (mut x, mut y, z, scale) = if project_native_draw {
            (
                visual.native_x,
                visual.native_y,
                visual.native_z,
                pal_scale_to_factor(visual.raw_scale),
            )
        } else {
            render_from_native_wrapper(visual, logical_width, logical_height)
        };
        let aspect_type = self
            .game_sprite_aspect_position_types
            .get(&slot)
            .copied()
            .unwrap_or(0);
        if let Some((adjusted_x, adjusted_y)) = self.convert_aspect_position(x, y, aspect_type) {
            x = adjusted_x;
            y = adjusted_y;
        }
        let pos_ok = sprites.set_pos_float(handle, x, y, z);
        let scale_ok = sprites.set_scale(handle, scale);
        let projection_ok = sprites.set_native_projection(
            handle,
            project_native_draw.then_some((
                logical_width.max(1) as f32 / 1920.0,
                logical_height.max(1) as f32 / 1080.0,
            )),
        );
        log::debug!(
            "[trace-sprite-native] commit slot={slot} handle={handle:?} native=({:.1},{:.1},{:.1}) temp=({:.1},{:.1},{:.1}) raw_scale={} aspect={} project_native_draw={} submitted=({:.1},{:.1},{:.1}) scale={:.3}",
            visual.native_x,
            visual.native_y,
            visual.native_z,
            temp.0,
            temp.1,
            temp.2,
            visual.raw_scale,
            aspect_type,
            project_native_draw,
            x,
            y,
            z,
            scale
        );
        pos_ok && scale_ok && projection_ok
    }

    fn active_position_delta_for_slot(&self, slot: i32, handle: SpriteHandle) -> (f32, f32, f32) {
        self.game_sprite_pending_position
            .iter()
            .filter(|action| action.slot == slot && action.handle == handle)
            .fold((0.0, 0.0, 0.0), |(sum_x, sum_y, sum_z), action| {
                let elapsed = self.pal_time_ms.wrapping_sub(action.started_ms);
                let clamped = elapsed.min(action.duration_ms);
                let t = if action.duration_ms == 0 {
                    1.0
                } else {
                    clamped as f32 / action.duration_ms.max(1) as f32
                };
                (
                    sum_x + action.native_dx * t,
                    sum_y + action.native_dy * t,
                    sum_z + action.native_dz * t,
                )
            })
    }

    /// Convert the Game sprite wrapper scale into the PAL sprite scale.
    ///
    /// Evidence: Game.exe `sp_set_scale` stores `raw_scale / 100.0` into wrapper
    /// offset +84, and PAL.dll `PalSpriteSetScale_0 @ 0x10260DF0` stores that
    /// float directly at `PalSprite +0x70`.
    ///
    /// Both the 5-argument `sp_set_ex` path and the 3-argument placement-helper
    /// path ultimately call the same `PalSpriteSetScale` submit in
    /// `sub_4494D0`, so the PAL sprite receives the bare `raw_scale / 100.0`.
    /// Native-frame projection belongs to the wrapper position lanes, not to
    /// this scale value; multiplying scale by the window projection shrinks
    /// standing sprites a second time.
    fn sprite_pal_scale_factor_for_placement(
        _arg_count: usize,
        raw_scale: i32,
        _logical_width: u32,
        _logical_height: u32,
    ) -> f32 {
        pal_scale_to_factor(raw_scale)
    }

    fn ext_sp_set_rotate(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        let (Some(slot), Some(x), Some(y), Some(z)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
            args.get(3).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            sprites.set_rotate_ex(handle, x as f32, y as f32, z as f32);
        }
        ExtCallOutcome::Value(1)
    }

    /// Paint an image resource into an existing sprite's surface. Popup scripts
    /// use this to put the question artwork on top of POP_BASE before showing
    /// the sprite; no separate text sprite is created for that artwork.
    fn ext_sp_surface_image(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(9);
        if args.len() < 9 {
            return ExtCallOutcome::Block;
        }
        let [slot, dst_x, dst_y, width, height, resource_value, src_x, src_y, mode]: [i32; 9] =
            args.try_into().expect("nine surface-image arguments");
        let (Some(resource_manager), Some(sprites)) = (resource_manager, sprites) else {
            return ExtCallOutcome::Value(0);
        };
        let Some(handle) = self.game_sprites.get(&slot).copied() else {
            return ExtCallOutcome::Value(0);
        };
        let Some(name) = self.resolve_resource_string(resource_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        let asset = match open_resource_variant(resource_manager, &name, IMAGE_EXTENSIONS) {
            Ok(asset) => asset,
            Err(err) => {
                log::warn!("[trace-sprite] surface_image resource={name:?} open failed: {err}");
                return ExtCallOutcome::Value(0);
            }
        };
        let decoded = match decode_asset_image(resource_manager, &asset) {
            Ok(decoded) => decoded,
            Err(err) => {
                log::warn!("[trace-sprite] surface_image resource={name:?} decode failed: {err}");
                return ExtCallOutcome::Value(0);
            }
        };
        if mode != 0 {
            log::debug!("[trace-sprite] surface_image resource={name:?} mode={mode}");
        }
        let painted = sprites.composite_rgba_to_sprite(
            handle,
            dst_x,
            dst_y,
            decoded.width,
            decoded.height,
            &decoded.rgba,
            src_x,
            src_y,
            width.max(0) as u32,
            height.max(0) as u32,
        );
        log::debug!(
            "[trace-sprite] surface_image slot={slot} resource={name:?} dst=({dst_x},{dst_y}) src=({src_x},{src_y}) size=({width},{height}) painted={painted}"
        );
        ExtCallOutcome::Value(i32::from(painted))
    }

    /// Game category 3 index 55 (`sub_425EF0`, native log `sp_set_mask`).
    ///
    /// The handler pops, in native top-first order:
    /// `slot, dst_x, dst_y, width, height, mask_resource, mask_x, mask_y, lane`.
    /// It writes those fields into one of the wrapper's mask lanes and immediately
    /// calls `sub_422FD0`, which resolves `mask_resource + ".tga"` and calls
    /// `PalSpriteMaskAlpha(live_sprite, dst_x, dst_y, width, height, mask, mask_x,
    /// mask_y)`. These are sprite-surface pixel coordinates, not stage
    /// coordinates, so they deliberately do not use `scale_script_x/y`.
    fn ext_sp_set_mask(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(9);
        if args.len() < 9 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let dst_x = args[1];
        let dst_y = args[2];
        let width = args[3].max(0) as u32;
        let height = args[4].max(0) as u32;
        let resource_value = args[5];
        let mask_x = args[6];
        let mask_y = args[7];
        let lane = args[8];

        if !(-1..=128).contains(&slot) || !(-1..=8).contains(&lane) {
            log::debug!("[trace-sprite] sp_set_mask graph-error slot={slot} lane={lane}");
            return ExtCallOutcome::Value(0);
        }
        if resource_value == 0x0FFF_FFFF || width == 0 || height == 0 {
            return ExtCallOutcome::Value(1);
        }
        let Some(name) = self.resolve_resource_string(resource_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        if name.is_empty() || is_resource_clear_sentinel(&name) {
            return ExtCallOutcome::Value(1);
        }
        let (Some(sprites), Some(resource_manager), Some(target)) = (
            sprites,
            resource_manager,
            self.game_sprites.get(&slot).copied(),
        ) else {
            log::debug!(
                "[trace-sprite] sp_set_mask slot={slot} lane={lane} name={name:?} missing target/resource manager"
            );
            return ExtCallOutcome::Value(1);
        };
        let asset = match open_resource_variant(resource_manager, &name, MASK_IMAGE_EXTENSIONS) {
            Ok(asset) => asset,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] sp_set_mask slot={slot} lane={lane} name={name:?} open failed: {err}"
                );
                return ExtCallOutcome::Value(0);
            }
        };
        let decoded = match decode_asset_image(resource_manager, &asset) {
            Ok(decoded) => decoded,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] sp_set_mask slot={slot} lane={lane} asset={:?} decode failed: {err}",
                    asset.name
                );
                return ExtCallOutcome::Value(0);
            }
        };
        let surface_id = sprites.allocate_surface_id();
        let surface = match SpriteSurface::rgba8(
            surface_id,
            1,
            decoded.width,
            decoded.height,
            decoded.rgba,
        ) {
            Ok(surface) => surface,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] sp_set_mask slot={slot} lane={lane} surface failed: {err}"
                );
                return ExtCallOutcome::Value(0);
            }
        };
        sprites.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), decoded.width, decoded.height);
        desc.visible = false;
        desc.source_name = asset.name.clone();
        let mask = sprites.create(desc);
        let masked = sprites.mask_alpha(target, dst_x, dst_y, width, height, mask, mask_x, mask_y);
        sprites.release(mask);
        log::debug!(
            "[trace-sprite] sp_set_mask slot={slot} lane={lane} asset={:?} dst=({dst_x},{dst_y}) src=({mask_x},{mask_y}) size={}x{} masked={masked}",
            asset.name,
            width,
            height
        );
        ExtCallOutcome::Value(i32::from(masked))
    }

    /// Game category 3 index 56 (`sub_426180`, logged by native as
    /// `sp_bitblt_file`) pops:
    ///
    /// `slot, dst_x, dst_y, width, height, resource, src_x, src_y, lane`.
    ///
    /// Native stores these fields in the sprite wrapper's 8 motion/bitblt
    /// lanes, marks the wrapper dirty, and `sub_422E10` calls
    /// `PalCopySpriteToSpriteRGB` from the named resource into the existing PAL
    /// sprite.  It does not create a new visible sprite when the target wrapper
    /// is missing.  The old portable fallback only popped nine arguments, which
    /// kept the stack safe but lost the copy semantics and let placeholder
    /// resources such as `BGM_SECRET` leak through unrelated visible layers.
    fn ext_sp_set_motion_pos(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(9);
        if args.len() < 9 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let dst_x = args[1];
        let dst_y = args[2];
        let width = args[3].max(0) as u32;
        let height = args[4].max(0) as u32;
        let resource_value = args[5];
        let src_x = args[6];
        let src_y = args[7];
        let lane = args[8];

        if !(-1..=128).contains(&slot) || !(-1..=8).contains(&lane) {
            log::debug!("[trace-sprite] sp_set_motion_pos graph-error slot={slot} lane={lane}");
            return ExtCallOutcome::Value(0);
        }
        if resource_value == 0x0FFF_FFFF {
            return ExtCallOutcome::Value(1);
        }
        let Some(name) = self.resolve_resource_string(resource_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        if name.is_empty() || is_resource_clear_sentinel(&name) {
            return ExtCallOutcome::Value(1);
        }
        let (Some(sprites), Some(target)) = (sprites, self.game_sprites.get(&slot).copied()) else {
            log::debug!(
                "[trace-sprite] sp_set_motion_pos slot={slot} lane={lane} name={name:?} missing target"
            );
            return ExtCallOutcome::Value(1);
        };
        if width == 0 || height == 0 {
            log::debug!(
                "[trace-sprite] sp_set_motion_pos slot={slot} lane={lane} name={name:?} empty rect dst=({dst_x},{dst_y}) src=({src_x},{src_y}) size={}x{}",
                width,
                height
            );
            return ExtCallOutcome::Value(1);
        }

        let source = if let Some((a, r, g, b)) = parse_solid_color_name_argb(&name) {
            let (logical_width, logical_height) = self.logical_size();
            let mut pixels = vec![0u8; logical_width as usize * logical_height as usize * 4];
            for px in pixels.chunks_exact_mut(4) {
                px.copy_from_slice(&[r, g, b, a]);
            }
            let surface_id = sprites.allocate_surface_id();
            let Ok(surface) =
                SpriteSurface::rgba8(surface_id, 1, logical_width, logical_height, pixels)
            else {
                return ExtCallOutcome::Value(0);
            };
            sprites.insert_surface(surface);
            let mut desc =
                SpriteDesc::new(SceneTextureId(surface_id.0), logical_width, logical_height);
            desc.visible = false;
            desc.source_name = name.clone();
            Some(sprites.create(desc))
        } else {
            let Some(resource_manager) = resource_manager else {
                return ExtCallOutcome::Value(0);
            };
            let asset = match open_resource_variant(resource_manager, &name, IMAGE_EXTENSIONS) {
                Ok(asset) => asset,
                Err(err) => {
                    log::warn!(
                        "[trace-sprite] sp_set_motion_pos slot={slot} lane={lane} name={name:?} open failed: {err}"
                    );
                    return ExtCallOutcome::Value(0);
                }
            };
            let decoded = match decode_asset_image(resource_manager, &asset) {
                Ok(decoded) => decoded,
                Err(err) => {
                    log::warn!(
                        "[trace-sprite] sp_set_motion_pos slot={slot} lane={lane} asset={:?} decode failed: {err}",
                        asset.name
                    );
                    return ExtCallOutcome::Value(0);
                }
            };
            let surface_id = sprites.allocate_surface_id();
            let surface = match SpriteSurface::rgba8(
                surface_id,
                1,
                decoded.width,
                decoded.height,
                decoded.rgba,
            ) {
                Ok(surface) => surface,
                Err(err) => {
                    log::warn!(
                        "[trace-sprite] sp_set_motion_pos slot={slot} lane={lane} surface failed: {err}"
                    );
                    return ExtCallOutcome::Value(0);
                }
            };
            sprites.insert_surface(surface);
            let mut desc =
                SpriteDesc::new(SceneTextureId(surface_id.0), decoded.width, decoded.height);
            desc.cell_width = decoded.cell_width;
            desc.cell_height = decoded.cell_height;
            desc.visible = false;
            desc.source_name = asset.name;
            Some(sprites.create(desc))
        };

        let Some(source) = source else {
            return ExtCallOutcome::Value(0);
        };
        let copied = sprites
            .copy_sprite_to_sprite_rgb(target, dst_x, dst_y, source, src_x, src_y, width, height);
        sprites.release(source);
        log::debug!(
            "[trace-sprite] sp_set_motion_pos slot={slot} lane={lane} name={name:?} dst=({dst_x},{dst_y}) src=({src_x},{src_y}) size={}x{} copied={copied}",
            width,
            height
        );
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 8 (`sub_4244E0`) pops a sprite slot and dynamic
    /// string destination, copies the sprite filename/name into that string
    /// storage when possible, and returns the native resource id.  The portable
    /// renderer stores source names directly on sprites, so return 1 for a live
    /// sprite and keep the exact two-argument stack discipline.
    fn ext_sp_get_filename(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let dst_slot = args[1];
        let Some(handle) = self.game_sprites.get(&slot).copied() else {
            return ExtCallOutcome::Value(-1);
        };
        if let Some(sprite) = sprites.and_then(|sprites| sprites.get(handle)) {
            let value = sprite.source_name.clone();
            if is_dynamic_string_handle(dst_slot) {
                self.replace_dynamic_string(dst_slot, value);
            } else if dst_slot >= 0 {
                let dyn_id = self.store_dynamic_string(value);
                self.write_temp_mem_relative(dst_slot, dyn_id);
            }
        }
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 16 (`sub_427530`) pops sprite slot and render
    /// mode, then calls PalSpriteSetRenderMode(mode + 1).
    fn ext_sp_set_render_mode(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let mode = args[1].saturating_add(1).max(0) as u32;
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            let _ = sprites.set_render_mode(handle, PalRenderMode::new(mode));
        }
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 28 (`sub_427D10`) pops sprite slot and three
    /// Mem.dat destination indexes, then writes the composite native x/y/z
    /// position into each non--1 destination.
    fn ext_sp_get_pos_to_mem(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let dst_x = args[1];
        let dst_y = args[2];
        let dst_z = args[3];
        let (x, y, z) = if let (Some(handle), Some(sprites)) =
            (self.game_sprites.get(&slot).copied(), sprites)
        {
            sprites
                .get(handle)
                .map(|sprite| {
                    (
                        sprite.info().position.x as i32,
                        sprite.info().position.y as i32,
                        sprite.info().position.z as i32,
                    )
                })
                .unwrap_or((0, 0, 0))
        } else {
            (0, 0, 0)
        };
        for (dst, value) in [(dst_x, x), (dst_y, y), (dst_z, z)] {
            if dst >= 0 {
                self.write_mem_dat_word(dst as usize, value);
            }
        }
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 34 (`sub_425870`) stores one native per-sprite
    /// parameter at ctx+666396.
    fn ext_sp_set_anim_param(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        self.game_sprite_anim_params.insert(args[0], args[1]);
        ExtCallOutcome::Value(1)
    }

    /// Game category 3 index 35 (`sub_4257F0`) returns the native per-sprite
    /// parameter stored by index 34.
    fn ext_sp_get_anim_param(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        ExtCallOutcome::Value(
            self.game_sprite_anim_params
                .get(&slot)
                .copied()
                .unwrap_or(0),
        )
    }

    /// Game category 3 index 48 (newer `0x447B40`) checks the wrapper handle,
    /// including packed button references resolved by `0x47ABA0`. Alpha and
    /// visibility do not participate: scripts query this before fading in.
    fn ext_sp_is(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        if self.input_registered_profile && slot < 0 {
            return ExtCallOutcome::Yield {
                value: (slot == -1).then_some(0),
            };
        }
        let handle = self.packed_button_sprite_handle(slot).or_else(|| {
            self.game_sprites.get(&slot).copied()
        });
        let live = handle
            .and_then(|handle| sprites.as_deref().and_then(|sprites| sprites.get(handle)))
            .is_some();
        ExtCallOutcome::Value(i32::from(live))
    }

    fn ext_sp_get_dimension(
        &mut self,
        sprites: Option<&mut SpriteSystem>,
        width: bool,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let handle = self
            .game_sprites
            .get(&slot)
            .copied()
            .or_else(|| self.packed_button_sprite_handle(slot));
        let value = if let (Some(handle), Some(sprites)) = (handle, sprites) {
            (if width {
                sprites.get_width(handle).unwrap_or(0)
            } else {
                sprites.get_height(handle).unwrap_or(0)
            }) as i32
        } else {
            0
        };
        ExtCallOutcome::Value(value)
    }

    /// Menu scripts address button sprites from generic sprite extcalls through
    /// a packed reference `0x02000000 | group << 20 | index` (observed in the
    /// SOUND menu passing slider base/knob refs to `sp_get_width`). Newer
    /// Game.exe `0x47ABA0` masks the tag and group to four bits and the index
    /// to sixteen bits; bits 16..19 are not part of the button index.
    fn packed_button_sprite_handle(&self, slot: i32) -> Option<SpriteHandle> {
        self.game_buttons.get(&Self::packed_button_key(slot)?).map(|entry| entry.handle)
    }

    fn packed_button_key(slot: i32) -> Option<(i32, i32)> {
        let raw = slot as u32;
        if raw & 0x0F00_0000 != 0x0200_0000 {
            return None;
        }
        let group = ((raw >> 20) & 0x0F) as i32;
        let index = (raw & 0x0000_FFFF) as i32;
        Some((group, index))
    }

    fn ext_sp_get_scale(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let value = if let (Some(handle), Some(sprites)) =
            (self.game_sprites.get(&slot).copied(), sprites)
        {
            self.game_sprite_native_scales
                .get(&slot)
                .copied()
                .or_else(|| {
                    sprites
                        .get(handle)
                        .map(|sprite| factor_to_pal_scale(sprite.scale))
                })
                .unwrap_or(100)
        } else {
            100
        };
        ExtCallOutcome::Value(value)
    }

    fn ext_sp_bitblt(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let dst_slot = args[0];
        let src_slot = args[1];
        let x = args[2];
        let y = args[3];
        if let Some(sprites) = sprites {
            if let (Some(dst), Some(src)) = (
                self.game_sprites.get(&dst_slot).copied(),
                self.game_sprites.get(&src_slot).copied(),
            ) {
                if let Some(src_sprite) = sprites.get(src).cloned() {
                    let right = x.saturating_add(src_sprite.cell_size.width as i32);
                    let bottom = y.saturating_add(src_sprite.cell_size.height as i32);
                    let _ = sprites.set_rect(dst, Some(PalRect::new(x, y, right, bottom)));
                }
            }
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_shake(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let slot = args[0];
        let amplitude = args[1].abs().min(32);
        let phase = ((self.pal_time_ms / 33) & 1) as i32;
        let offset = if amplitude == 0 {
            0
        } else if phase == 0 {
            amplitude
        } else {
            -amplitude
        };
        if let (Some(handle), Some(sprites)) = (self.game_sprites.get(&slot).copied(), sprites) {
            let _ = sprites.set_offset_pos(handle, offset, 0);
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_set_transition(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        mut sprites: Option<&mut SpriteSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        // Game.exe sub_428F10 (`sp_set_transition`) pops:
        //   pop[0]=resource name, pop[1]=slot, pop[2]=transition/effect id,
        //   pop[3]=duration.  It keeps the old PAL sprite pointer in a wrapper
        // lane, installs the newly loaded image as the live sprite, and queues
        // a PAL transition old -> new.  The old implementation treated pop[0]
        // as a transition slot, which made page changes transition the wrong
        // layer and left baked copy artifacts on SYSTEM/SOUND screens.
        let name_value = args[0];
        let slot = args[1];
        let transition_id = args[2].max(0) as u32;
        let duration_ms = args[3].max(1) as u32;
        if name_value == 0x0FFF_FFFF {
            return ExtCallOutcome::Value(1);
        }
        let Some(mut name) = self.resolve_resource_string(name_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        if name.is_empty() {
            return ExtCallOutcome::Value(1);
        }
        if is_resource_clear_sentinel(&name) {
            if let (Some(sprites), Some(old)) =
                (sprites.as_deref_mut(), self.game_sprites.remove(&slot))
            {
                sprites.release(old);
            }
            if let Some(old) = self.game_sprite_transition_sources.remove(&slot) {
                if let Some(sprites) = sprites.as_deref_mut() {
                    sprites.release(old);
                }
            }
            self.game_sprite_placements.remove(&slot);
            self.game_sprite_native_scales.remove(&slot);
            self.game_sprite_wrapper_visuals.remove(&slot);
            self.game_sprite_child_offsets.remove(&slot);
            self.game_sprite_vis_clip_slots.remove(&slot);
            self.game_sprite_pending_named_animations.remove(&slot);
            self.clear_alpha_lanes_for_slot(slot);
            self.clear_pending_position_actions_for_slot(slot);
            if let Some(sprites) = sprites.as_deref_mut() {
                self.release_retired_sprites_for_slot(slot, sprites);
            } else {
                self.retired_sprites.retain(|retired| retired.slot != slot);
            }
            log::debug!(
                "[trace-sprite] sp_set_transition slot={slot} sentinel=# action=clear transition_id={transition_id} duration={duration_ms}"
            );
            return ExtCallOutcome::Value(1);
        }
        let (Some(resource_manager), Some(sprites)) = (resource_manager, sprites) else {
            return ExtCallOutcome::Value(0);
        };
        if is_named_animation_resource(&name) {
            match open_resource_variant(resource_manager, &name, ANIMATION_EXTENSIONS) {
                Ok(asset) => {
                    if let Some(handle) = self.game_sprites.get(&slot).copied() {
                        if self.apply_named_sprite_animation(sprites, handle, &asset.bytes) {
                            log::debug!(
                                "[trace-sprite] sp_set_transition slot={slot} name={name:?} asset={:?} applied named animation transition_id={transition_id} duration={duration_ms}",
                                asset.name
                            );
                            return ExtCallOutcome::Value(1);
                        }
                        log::warn!(
                            "[trace-sprite] sp_set_transition slot={slot} name={name:?} asset={:?} unsupported named animation",
                            asset.name
                        );
                        return ExtCallOutcome::Value(0);
                    }
                    // Native sub_428F10 sends the resource through the same
                    // sprite-wrapper loader/commit pipeline used for image
                    // resources.  Background scroll transitions can pass ANI_*
                    // resources before the destination layer is materialized;
                    // keep the parsed animation pending for the next sp_set on
                    // that slot instead of treating the missing PGD as failure.
                    self.game_sprite_pending_named_animations.insert(
                        slot,
                        PendingNamedSpriteAnimation {
                            asset_name: asset.name.clone(),
                            bytes: asset.bytes,
                        },
                    );
                    log::debug!(
                        "[trace-sprite] sp_set_transition slot={slot} name={name:?} queued named animation asset={:?}",
                        asset.name
                    );
                    return ExtCallOutcome::Value(1);
                }
                Err(err) => {
                    log::warn!(
                        "[trace-sprite] sp_set_transition slot={slot} name={name:?} animation open failed: {err}"
                    );
                    return ExtCallOutcome::Value(0);
                }
            }
        }
        if let Some(record) = assets
            .graphic_index
            .as_ref()
            .and_then(|index| index.lookup(&name))
        {
            if let Some(replacement) = record.replacement_resource() {
                name = replacement;
            }
        }
        let asset = match open_resource_variant(resource_manager, &name, IMAGE_EXTENSIONS) {
            Ok(asset) => asset,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] sp_set_transition slot={slot} name={name:?} open failed: {err}"
                );
                return ExtCallOutcome::Value(0);
            }
        };
        let decoded = match decode_asset_image(resource_manager, &asset) {
            Ok(decoded) => decoded,
            Err(err) => {
                log::warn!(
                    "[trace-sprite] sp_set_transition slot={slot} asset={:?} decode failed: {err}",
                    asset.name
                );
                return ExtCallOutcome::Value(0);
            }
        };
        let Some(from_handle) = self.game_sprites.remove(&slot) else {
            // Game.exe sub_428F10 checks the live PAL sprite pointer in the
            // wrapper before installing the replacement.  If the slot has no
            // current image it logs "sprite transition error : no image" and
            // returns 0.  Creating a fresh sprite here made expression changes
            // appear at the default origin and survive later page clears.
            log::warn!(
                "[trace-sprite] sp_set_transition slot={slot} name={name:?} asset={:?} no current image",
                asset.name
            );
            return ExtCallOutcome::Value(0);
        };
        if let Some(previous_source) = self.game_sprite_transition_sources.remove(&slot) {
            // `sp_copy_image` is the only path that intentionally parks an
            // old-image lane for a later `sp_transition`.  `sp_set_transition`
            // itself installs the newly loaded PAL sprite as the live wrapper
            // pointer, so a stale parked source must not survive across
            // expression changes.
            sprites.release(previous_source);
        }
        let from = Some(from_handle);
        let native_scale = self
            .game_sprite_native_scales
            .get(&slot)
            .copied()
            .unwrap_or(100);
        let inherited_visual = self.game_sprite_wrapper_visuals.get(&slot).copied();
        let inherited_state = from.and_then(|handle| {
            sprites.get(handle).map(|sprite| {
                (
                    sprite.position,
                    sprite.color,
                    sprite.offset,
                    sprite.center_offset,
                    sprite.scale,
                    sprite.rotation,
                    sprite.render_mode,
                    sprite.option_type,
                    sprite.info_extra,
                    sprite.base_priority,
                    sprite.source_rect,
                )
            })
        });
        let (logical_width, logical_height) = self.logical_size();
        let placement =
            self.game_sprite_placements
                .get(&slot)
                .copied()
                .unwrap_or(GameSpritePlacement {
                    arg_count: 5,
                    raw_x: 0,
                    raw_y: 0,
                    raw_z: 0,
                    width: decoded.width,
                    height: decoded.height,
                    default_x: decoded.offset_x,
                    default_y: decoded.offset_y,
                });
        let mut next_visual = inherited_visual.unwrap_or_else(|| {
            let (native_x, native_y, native_z) = native_place_sprite(
                placement.arg_count,
                placement.raw_x,
                placement.raw_y,
                placement.raw_z,
                decoded.width,
                decoded.height,
                decoded.offset_x,
                decoded.offset_y,
                logical_width,
                logical_height,
            );
            GameSpriteWrapperVisual {
                native_x,
                native_y,
                native_z,
                raw_scale: native_scale,
                width: decoded.width,
                height: decoded.height,
                arg_count: placement.arg_count,
                project_native_draw: false,
            }
        });
        next_visual.width = decoded.width;
        next_visual.height = decoded.height;
        next_visual.raw_scale = native_scale;
        next_visual.arg_count = placement.arg_count;
        next_visual.project_native_draw = inherited_visual
            .map(|visual| visual.project_native_draw)
            .unwrap_or(false);
        let project_native_draw = next_visual.project_native_draw;
        let (x, y, z, render_scale) = if project_native_draw {
            (
                next_visual.native_x,
                next_visual.native_y,
                next_visual.native_z,
                pal_scale_to_factor(next_visual.raw_scale),
            )
        } else {
            render_from_native_wrapper(next_visual, logical_width, logical_height)
        };
        let surface_id = sprites.allocate_surface_id();
        let surface = match SpriteSurface::rgba8(
            surface_id,
            1,
            decoded.width,
            decoded.height,
            decoded.rgba,
        ) {
            Ok(surface) => surface,
            Err(err) => {
                log::warn!("[trace-sprite] sp_set_transition slot={slot} surface failed: {err}");
                return ExtCallOutcome::Value(0);
            }
        };
        sprites.insert_surface(surface);
        let mut desc = SpriteDesc::new(SceneTextureId(surface_id.0), decoded.width, decoded.height);
        desc.cell_width = decoded.cell_width;
        desc.cell_height = decoded.cell_height;
        desc.position = PalVec3::from_f32(x, y, z);
        desc.scale = render_scale;
        // Image replacement preserves the wrapper's coordinate contract.
        desc.center_scale = !project_native_draw;
        desc.native_projection = project_native_draw.then_some((
            logical_width.max(1) as f32 / 1920.0,
            logical_height.max(1) as f32 / 1080.0,
        ));
        desc.base_priority = game_sprite_priority(slot, self.sprite_priority_cursor);
        desc.visible = true;
        desc.source_name = asset.name.clone();
        if let Some((
            position,
            color,
            offset,
            center_offset,
            inherited_scale,
            rotation,
            render_mode,
            option_type,
            info_extra,
            base_priority,
            old_rect,
        )) = inherited_state
        {
            // Game.exe `sp_set_transition` swaps the PAL sprite pointer inside
            // the same wrapper.  Visual wrapper lanes survive the image swap;
            // without carrying them forward, expression transitions reset
            // placement/scale and make standing sprites jump under the ADV box.
            //
            // Do not write the inherited rendered position back into
            // `next_visual`: native action processing keeps running position
            // deltas in temporary lanes and folds the final delta into the
            // wrapper only when the action completes. Baking the temporary
            // position here double-counted the pending move after expression
            // transitions.
            desc.position = position;
            desc.color = color;
            desc.offset = offset;
            desc.center_offset = center_offset;
            // Game.exe swaps the PAL sprite pointer inside the same wrapper,
            // but the committed PAL sprite scale can include temporary action
            // lanes from sub_4494D0.  Rebuild the base scale from the native
            // raw wrapper lane instead of copying the previous rendered
            // composite scale; otherwise action/transition scratch values leak
            // into the new image and produce 4096x4096 ST standing sprites.
            desc.rotation = rotation;
            desc.render_mode = render_mode;
            desc.option_type = option_type;
            desc.info_extra = info_extra;
            desc.base_priority = base_priority;
            let initial_alpha =
                self.computed_game_sprite_alpha_for_new_wrapper_sprite(slot, color.alpha());
            desc.color = PalColor::from_argb(
                (u32::from(initial_alpha) << 24) | (desc.color.0 & 0x00FF_FFFF),
            );
            let new_rect_width = i32::try_from(decoded.cell_width.max(1)).unwrap_or(i32::MAX);
            let new_rect_height = i32::try_from(decoded.cell_height.max(1)).unwrap_or(i32::MAX);
            let old_rect_width = old_rect.width().max(1);
            let old_rect_height = old_rect.height().max(1);
            // Native uses arithmetic shifts of the unscaled size differences.
            // Normalize only position units, never the replacement image size.
            let mut adjust_x = ((new_rect_width - old_rect_width) >> 1) as f32;
            let mut adjust_y = ((new_rect_height - old_rect_height) >> 1) as f32;
            if !project_native_draw {
                adjust_x = unproject_script_x(adjust_x as i32, logical_width);
                adjust_y = unproject_script_y(adjust_y as i32, logical_height);
            }
            if adjust_x != 0.0 || adjust_y != 0.0 {
                // Game.exe sub_428F10 calls PalSpriteGetRect on the old and
                // newly loaded PAL sprites, then subtracts half of the rect
                // size delta from wrapper x/y before sub_4494D0 commits the
                // swap. Face/standing part transitions depend on this center
                // compensation; otherwise replacing a differently-sized part
                // shifts relative to the body.
                next_visual.native_x -= adjust_x;
                next_visual.native_y -= adjust_y;
                let (adj_x, adj_y, adj_z, adj_scale) = if project_native_draw {
                    (
                        next_visual.native_x,
                        next_visual.native_y,
                        next_visual.native_z,
                        pal_scale_to_factor(next_visual.raw_scale),
                    )
                } else {
                    render_from_native_wrapper(next_visual, logical_width, logical_height)
                };
                let base_visual = inherited_visual.unwrap_or(next_visual);
                let (base_x, base_y, base_z, _) = if project_native_draw {
                    (
                        base_visual.native_x,
                        base_visual.native_y,
                        base_visual.native_z,
                        pal_scale_to_factor(base_visual.raw_scale),
                    )
                } else {
                    render_from_native_wrapper(base_visual, logical_width, logical_height)
                };
                desc.position = PalVec3::from_f32(
                    position.x + (adj_x - base_x),
                    position.y + (adj_y - base_y),
                    position.z + (adj_z - base_z),
                );
                desc.scale = adj_scale;
                log::debug!(
                    "[trace-sprite] sp_set_transition rect_adjust slot={slot} asset={:?} old={}x{} new={}x{} delta=({adjust_x},{adjust_y}) pos=({:.0},{:.0},{:.0})",
                    asset.name,
                    old_rect_width,
                    old_rect_height,
                    new_rect_width,
                    new_rect_height,
                    desc.position.x,
                    desc.position.y,
                    desc.position.z
                );
            }
            if asset.name.to_ascii_uppercase().starts_with("ST") || inherited_scale > 8.0 {
                log::warn!(
                    "[trace-sprite-scale] sp_set_transition scale-inherit pc=0x{:08X} slot={slot} asset={:?} inherited_render_scale={inherited_scale:.3} native_raw_scale={native_scale} rebuilt_scale={:.3}",
                    self.pc,
                    asset.name,
                    desc.scale
                );
            }
        }
        let to = sprites.create(desc);
        self.game_sprites.insert(slot, to);
        if let Some(from_handle) = from {
            for action in &mut self.game_sprite_active_alpha {
                if action.slot == slot && action.handle == from_handle {
                    action.handle = to;
                }
            }
            for action in &mut self.game_sprite_pending_position {
                if action.slot == slot && action.handle == from_handle {
                    let elapsed = self.pal_time_ms.wrapping_sub(action.started_ms);
                    let duration = action.duration_ms.max(1);
                    let remaining_ms = action.duration_ms.saturating_sub(elapsed);
                    if remaining_ms > 0 {
                        let remaining_num = remaining_ms as f32 / duration as f32;
                        let project_x = if project_native_draw {
                            1.0
                        } else {
                            logical_width.max(1) as f32 / 1920.0
                        };
                        let project_y = if project_native_draw {
                            1.0
                        } else {
                            logical_height.max(1) as f32 / 1080.0
                        };
                        let dx = action.native_dx * project_x * remaining_num;
                        let dy = action.native_dy * project_y * remaining_num;
                        let dz = action.native_dz * remaining_num;
                        let _ = sprites.tween_pos_by(to, dx, dy, dz, remaining_ms);
                    }
                    action.handle = to;
                }
            }
        }
        self.game_sprite_base_alpha
            .entry(slot)
            .or_insert_with(|| inherited_state.map_or(255, |(_, color, ..)| color.alpha() as i32));
        self.game_sprite_final_alpha_delta.entry(slot).or_insert(0);
        self.submit_game_sprite_alpha(sprites, slot, to);
        self.game_sprite_wrapper_visuals.insert(slot, next_visual);
        self.game_sprite_placements.insert(
            slot,
            GameSpritePlacement {
                arg_count: placement.arg_count,
                raw_x: placement.raw_x,
                raw_y: placement.raw_y,
                raw_z: placement.raw_z,
                width: decoded.width,
                height: decoded.height,
                default_x: decoded.offset_x,
                default_y: decoded.offset_y,
            },
        );
        self.game_sprite_native_scales.entry(slot).or_insert(100);
        let transition = *self
            .game_sprite_transitions
            .entry(slot)
            .or_insert_with(|| sprites.create_transition_handle());
        let _ = sprites.set_transition(
            transition,
            1,
            Some(to),
            from,
            transition_id as i32,
            self.pal_time_ms,
            duration_ms,
        );
        log::debug!(
            "[trace-sprite] sp_set_transition pc=0x{:08X} slot={slot} name={name:?} asset={:?} from={from:?} to={to:?} transition_id={transition_id} duration={duration_ms} live=to retire_from_at={}",
            self.pc,
            asset.name,
            self.pal_time_ms.wrapping_add(duration_ms)
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_sp_copy_image(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(slot) = args.first().copied() else {
            return ExtCallOutcome::Block;
        };
        let Some(sprites) = sprites else {
            return ExtCallOutcome::Value(1);
        };
        // Game.exe sub_424940 (`sp_copy_image`) does not copy the full rendered
        // scene into a new texture.  It cancels any active transition and moves
        // the current PAL sprite pointer into the wrapper's old-image lane.  A
        // separate `get_backbuffer` helper is the one that calls
        // PalSpriteBackBafferCopy.  Keeping a whole-scene raster snapshot here
        // baked text/overlay/transition stripes into later menu pages.
        if let Some(transition) = self.game_sprite_transitions.get(&slot).copied() {
            let _ = sprites.cancel_transition(transition);
        }
        if let Some(previous_source) = self.game_sprite_transition_sources.remove(&slot) {
            sprites.release(previous_source);
        }
        if let Some(handle) = self.game_sprites.remove(&slot) {
            self.game_sprite_transition_sources.insert(slot, handle);
        }
        log::debug!("[trace-sprite] sp_copy_image slot={slot}");
        ExtCallOutcome::Value(1)
    }

    fn apply_pending_named_sprite_animation(
        &mut self,
        sprites: &mut SpriteSystem,
        slot: i32,
        handle: SpriteHandle,
    ) {
        let Some(pending) = self.game_sprite_pending_named_animations.remove(&slot) else {
            return;
        };
        if self.apply_named_sprite_animation(sprites, handle, &pending.bytes) {
            log::debug!(
                "[trace-sprite] sp_set slot={slot} asset={:?} applied queued named animation",
                pending.asset_name
            );
        } else {
            log::warn!(
                "[trace-sprite] sp_set slot={slot} asset={:?} queued named animation unsupported",
                pending.asset_name
            );
        }
    }

    fn ext_sp_transition(&mut self, sprites: Option<&mut SpriteSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        let (Some(sprite_slot), Some(transition_id), Some(duration_ms)) = (
            args.first().copied(),
            args.get(1).copied(),
            args.get(2).copied(),
        ) else {
            return ExtCallOutcome::Block;
        };
        // koikake.exe runs the transition through the same per-frame skip gate
        // as effect_stop (0x411E0F): an active skip state collapses the fade
        // to a single frame instead of playing it out.
        let duration_ms = if self.dispatch_skip_state() != 0 {
            1
        } else {
            duration_ms.max(1)
        };
        if let Some(sprites) = sprites {
            let transition = *self
                .game_sprite_transitions
                .entry(sprite_slot)
                .or_insert_with(|| sprites.create_transition_handle());
            let to = self.game_sprites.get(&sprite_slot).copied();
            let from = self
                .game_sprite_transition_sources
                .get(&sprite_slot)
                .copied();
            if to.is_some() {
                let _ = sprites.set_transition(
                    transition,
                    1,
                    to,
                    from,
                    transition_id,
                    self.pal_time_ms,
                    duration_ms.max(1) as u32,
                );
            }
        }
        log::debug!(
            "[trace-sprite] sp_transition slot={sprite_slot} id={transition_id} duration={duration_ms}"
        );
        ExtCallOutcome::Value(1)
    }

    fn capture_save_snapshot(&self) -> RuntimeSaveSnapshot {
        RuntimeSaveSnapshot {
            ui: Some(self.capture_ui()),
            host_checkpoint: None,
            dynamic_strings: Some((self.dynamic_strings.clone(), self.dynamic_string_cursor)),
            version: PORTABLE_SAVE_VERSION,
            pc: self.pc,
            call_stack: self.call_stack.clone(),
            user_mem: bounded_i32_copy(&self.user_mem, DEFAULT_MEM_SIZE),
            system_mem: bounded_i32_copy(&self.system_mem, DEFAULT_MEM_SIZE),
            temp_mem: bounded_i32_copy(&self.temp_mem, DEFAULT_MEM_SIZE),
            mem_dat_words: bounded_i32_copy(&self.mem_dat_words, SAVE_MEMDAT_CAP),
            history_records: self.history_state.records.clone(),
            history_texts: self.history_state.texts.clone(),
            text_args: self.text_state.last_text_args,
            text_base: self.text_state.base,
            text_mode: self.text_state.mode,
            text_alpha: Some(self.text_state.alpha),
            text_visible: self.text_state.visible,
            vars: bounded_i32_copy(&self.vars, DEFAULT_VAR_COUNT),
            stack: self.stack.clone(),
            argument_stack: self.argument_stack.clone(),
            argument_base: self.argument_base,
            text_initialized: self.text_state.initialized,
            text_init_args: self.text_state.init_args,
            text_color: self.text_state.text_color,
            text_effect_color: self.text_state.text_effect_color,
            show_wait_mark: self.text_state.show_wait_mark,
            resume_wait_click: false,
            title_bytes: self.save_state.title_bytes.clone(),
            thumb_width: self.save_state.thumbnail_size[0],
            thumb_height: self.save_state.thumbnail_size[1],
            thumb_pixels: self.save_state.thumbnail_pixels.clone(),
            sprites: Vec::new(),
            buttons: Vec::new(),
            button_groups: Vec::new(),
            bgm_tracks: self.playing_bgm_tracks(),
            tips: self.capture_tips_snapshot(),
            bgv_tracks: self.saved_bgv_tracks(),
            actions: actions::SavedActions::default(),
            children: self.capture_children(None),
        }
    }

    /// BGM slots currently playing, flattened for the portable snapshot.
    fn playing_bgm_tracks(&self) -> Vec<SavedBgmTrack> {
        self.bgm_slots
            .iter()
            .filter(|(_, state)| state.playing && !state.name.is_empty())
            .map(|(&slot, state)| SavedBgmTrack {
                slot,
                name: state.name.chars().take(SAVE_NAME_CAP).collect(),
                looping: state.looping,
                loop_start: state.loop_samples.map(|(start, _)| start).unwrap_or(-1),
                loop_end: state.loop_samples.map(|(_, end)| end).unwrap_or(-1),
            })
            .collect()
    }

    fn capture_save_checkpoint(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        sprites: Option<&SpriteSystem>,
    ) {
        if self.temp_mem.len() > DEFAULT_MEM_SIZE {
            self.temp_mem.truncate(DEFAULT_MEM_SIZE);
            self.temp_mem.shrink_to_fit();
        }
        if let Some(sprites) = sprites {
            self.capture_thumbnail(sprites);
        }
        let (body, name) = self.adv_text_parts_for_render(assets, nls);
        let tips = self.capture_tips_snapshot_for_body(&body);
        let title = if name.is_empty() {
            body
        } else if body.is_empty() {
            name
        } else {
            format!("{name} {body}")
        };
        let (title, _) = parse_pal_text_directives(&title);
        if !title.is_empty() {
            self.save_state.title_bytes = title.into_bytes();
        }
        let mut snapshot = self.capture_resumable_scene(sprites);
        snapshot.tips = tips;
        self.save_state.resume_checkpoint = None;
        self.save_state.checkpoint = Some(snapshot);
        self.save_state.armed = true;
    }

    fn capture_resumable_scene(&self, sprites: Option<&SpriteSystem>) -> RuntimeSaveSnapshot {
        let mut snapshot = self.capture_save_snapshot();
        if let Some(sprites) = sprites {
            snapshot.actions = self.capture_actions(sprites);
            snapshot.children = self.capture_children(Some(sprites));
            snapshot.sprites = self.capture_sprite_records(sprites);
            snapshot.buttons = self.capture_button_records(sprites);
            snapshot.button_groups = self
                .button_groups
                .iter()
                .map(|(&group, entry)| SavedButtonGroup {
                    group,
                    normal_image: entry.normal_image,
                    hover_image: entry.hover_image,
                    onmouse_index: entry.onmouse_index,
                    offmouse_index: entry.offmouse_index,
                    push_index: entry.push_index,
                    slider_ended: entry.slider_ended,
                    raw_toggle: entry.raw_toggle,
                    locked: entry.locked,
                    lock_started_ms: entry.lock_started_ms,
                    lock_duration_ms: entry.lock_duration_ms,
                })
                .collect();
        }
        snapshot
    }

    fn resumable_save_snapshot(&self) -> Option<RuntimeSaveSnapshot> {
        self.save_state
            .resume_checkpoint
            .clone()
            .or_else(|| self.save_state.checkpoint.clone())
    }

    fn capture_sprite_records(&self, sprites: &SpriteSystem) -> Vec<SavedSprite> {
        let mut records = Vec::new();
        for (&slot, &handle) in &self.game_sprites {
            if let Some(record) = saved_sprite_from_handle(sprites, slot, handle) {
                records.push(record);
            }
            if records.len() >= SAVE_SPRITE_CAP {
                break;
            }
        }
        records
    }

    fn capture_button_records(&self, sprites: &SpriteSystem) -> Vec<SavedButton> {
        let mut records = Vec::new();
        for (&(group, index), entry) in &self.game_buttons {
            let Some(sprite) = sprites.get(entry.handle) else {
                continue;
            };
            let Some(surface) = sprites.surface(sprite.surface) else {
                continue;
            };
            let texture = surface.to_scene_texture();
            let expected = texture.width as usize * texture.height as usize * 4;
            if expected == 0 || expected > SAVE_SPRITE_BYTES_CAP || texture.pixels.len() < expected
            {
                continue;
            }
            records.push(SavedButton {
                base_position: entry.base_position,
                slider_position: entry.slider_position,
                registration_order: entry.registration_order,
                native_flags: entry.native_flags,
                hover_callbacks: entry.hover_callbacks,
                base_row: entry.toggle,
                group,
                index,
                visible: entry.visible,
                enabled: entry.enabled,
                alpha: entry.alpha,
                gosub_point: entry.gosub_point.map(|point| point as i32).unwrap_or(-1),
                x: sprite.position.x as i32,
                y: sprite.position.y as i32,
                z: sprite.position.z as i32,
                priority: sprite.base_priority,
                width: texture.width,
                height: texture.height,
                cell_width: sprite.cell_size.width,
                cell_height: sprite.cell_size.height,
                rect: [
                    sprite.source_rect.left,
                    sprite.source_rect.top,
                    sprite.source_rect.right,
                    sprite.source_rect.bottom,
                ],
                color: sprite.color.0,
                name: entry.name.chars().take(SAVE_NAME_CAP).collect(),
                rgba: texture.pixels[..expected].to_vec(),
            });
            if records.len() >= SAVE_SPRITE_CAP {
                break;
            }
        }
        records
    }

    /// Historical header-only fallback for builds whose complete native save
    /// layout is not audited. This does not reconstruct their native scene or
    /// memory blocks. Exact Koikake pairs use native_save instead.
    fn import_original_save(
        &self,
        root: &Path,
        slot: i32,
        assets: &CoreAssets,
    ) -> Option<RuntimeSaveSnapshot> {
        let path = find_loose_save_file(root, &original_save_filename(slot))?;
        let bytes = pal_asset::vfs::read_all(&path).ok()?;
        let (prefix, trailer) = decode_original_save(&bytes)?;
        if trailer.is_some() {
            // A portable trailer exists but failed to parse; do not guess.
            return None;
        }
        let resume_pc = prefix.resume_pc;
        // +0xC: the native tracker points past the current text extcall
        // (opcode word + category/index + dst slot), and resumes there.
        let resume_next = resume_pc as usize + 0xC;
        if resume_pc <= 0 || resume_next >= assets.script.bytes.len() {
            log::debug!("[trace-save] import slot={slot} rejected resume pc=0x{resume_pc:08X}");
            return None;
        }
        let text_value = read_original_text_value(&bytes).unwrap_or(0);
        let mut snapshot = self.capture_save_snapshot();
        snapshot.pc = resume_next as u32;
        snapshot.ui = None;
        snapshot.dynamic_strings = None;
        // This compatibility fallback does not decode native return addresses
        // or operand storage. Do not retain the outgoing menu's stacks.
        snapshot.call_stack = Vec::new();
        snapshot.stack = Vec::new();
        // Native scene records are unavailable for this profile. Drop the
        // outgoing menu rather than presenting it as the saved scene.
        snapshot.sprites = Vec::new();
        snapshot.buttons = Vec::new();
        snapshot.button_groups = Vec::new();
        snapshot.children = children::SavedChildren::default();
        // The native image does store a sound wrapper, but its layout is not
        // mapped yet; the tracks captured from the menu runtime (e.g. title
        // BGM) would be wrong for the restored scene, so drop them.
        snapshot.bgm_tracks = Vec::new();
        snapshot.bgv_tracks.clear();
        snapshot.history_records.clear();
        snapshot.history_texts.clear();
        snapshot.tips = SavedTipsState::default();
        if text_value != 0 {
            snapshot.text_args = [0, text_value, 0x0FFF_FFFF, 0x0FFF_FFFF];
            snapshot.text_visible = true;
            snapshot.show_wait_mark = true;
            // The image's ADV window alpha is part of the context this importer
            // does not map yet, and the menu the save was taken from is not the
            // scene being restored: show the window instead of inheriting the
            // menu's alpha.
            snapshot.text_alpha = Some(255);
        }
        snapshot.resume_wait_click = true;
        Some(snapshot)
    }

    fn restore_save_snapshot(&mut self, snapshot: RuntimeSaveSnapshot) {
        self.history_navigation.target = None;
        if let Some((strings, cursor)) = snapshot.dynamic_strings {
            self.dynamic_strings = strings;
            self.dynamic_string_cursor = cursor;
        }
        self.queue_restored_bgv(if snapshot.version >= 10 { snapshot.bgv_tracks.clone() } else { Vec::new() });
        self.restore_tips_snapshot(snapshot.tips);
        self.pc = snapshot.pc;
        self.call_stack = snapshot.call_stack;
        install_i32_words(&mut self.user_mem, &snapshot.user_mem, DEFAULT_MEM_SIZE);
        install_i32_words(&mut self.system_mem, &snapshot.system_mem, DEFAULT_MEM_SIZE);
        install_i32_words(&mut self.temp_mem, &snapshot.temp_mem, DEFAULT_MEM_SIZE);
        self.mem_dat_words = snapshot.mem_dat_words;
        self.history_state.records = snapshot.history_records;
        self.history_state.texts = snapshot.history_texts;
        self.history_state.texts.resize(self.history_state.records.len(), None);
        self.restore_ui(snapshot.ui.as_ref());
        self.history_state.records_generation += 1;
        self.history_state.layout_cache = None;
        self.history_state.hovered_record = None;
        self.text_state.last_text_args = snapshot.text_args;
        self.text_state.last_text_value = snapshot.text_args[1];
        self.text_state.base = snapshot.text_base;
        self.text_state.mode = snapshot.text_mode;
        // The script hides and re-shows the ADV window when a scene or
        // background changes (`text_window_set_alpha` around `text_set_base`),
        // so a save restored mid-scene has no call coming to re-establish what
        // the window looked like at save time. Saves written before version 7
        // and original-engine images carry no alpha: assume the window the
        // script left on screen is shown, instead of restoring it to alpha 0,
        // where the base panel paints nothing and the text sits on a naked CG.
        self.text_state.alpha = snapshot.text_alpha.unwrap_or(255);
        self.text_state.visible = snapshot.text_visible;
        self.text_state.reveal_enabled = false;
        self.text_state.dirty = true;
        if snapshot.version >= 2 {
            install_i32_words(&mut self.vars, &snapshot.vars, DEFAULT_VAR_COUNT);
            self.stack = snapshot.stack;
            self.argument_stack = snapshot.argument_stack;
            self.argument_base = snapshot.argument_base;
            self.text_state.initialized = snapshot.text_initialized;
            self.text_state.init_args = snapshot.text_init_args;
            self.text_state.text_color = snapshot.text_color;
            self.text_state.text_effect_color = snapshot.text_effect_color;
            self.text_state.show_wait_mark = snapshot.show_wait_mark;
        }
        // The load script can start a fade before invoking `load`. That fade
        // belongs to the outgoing scene and must not cover the restored one.
        self.effect_system.stop_selected(0x1d);
        self.wait_time_stack.clear();
        self.adv_wait_checkpoint_pending = false;
        self.modal_wait_suspensions.clear();
        if snapshot.version >= 5 {
            self.bgm_slots = snapshot
                .bgm_tracks
                .iter()
                .map(|track| {
                    (
                        track.slot,
                        BgmSlotState {
                            name: track.name.clone(),
                            native_flags: i32::from(track.looping),
                            looping: track.looping,
                            playing: true,
                            loop_samples: (track.loop_start >= 0 && track.loop_end >= 0)
                                .then_some((track.loop_start, track.loop_end)),
                        },
                    )
                })
                .collect();
            // Replay runs even with an empty track list so the outgoing
            // scene's BGM does not keep playing under the restored one.
            self.bgm_replay_pending = true;
        }
        self.status = RuntimeStatus::Running { pc: self.pc };
    }

    fn restore_checkpoint_scene(
        &mut self,
        snapshot: &RuntimeSaveSnapshot,
        sprites: &mut SpriteSystem,
    ) {
        self.release_scene_for_load(sprites);
        for record in &snapshot.sprites {
            let Some(handle) = sprites.create_shared_rgba_sprite(
                record.width,
                record.height,
                record.rgba.clone(),
                PalVec3::from_f32(record.x as f32, record.y as f32, record.z as f32),
                record.priority,
                format!("save-sprite:{}", record.slot),
            ) else {
                continue;
            };
            if let Some(scale_y) = record.scale_y_bits {
                let _ = sprites.set_scale_xy(handle, f32::from_bits(record.scale_bits), f32::from_bits(scale_y));
            } else {
                let _ = sprites.set_scale(handle, f32::from_bits(record.scale_bits));
            }
            let _ = sprites.set_color(handle, PalColor(record.color));
            let _ = sprites.view_ctrl(handle, record.visible);
            let _ = sprites.set_native_projection(handle, record.native_projection);
            if let Some(sprite) = sprites.get_mut(handle) {
                sprite.center_scale = record.center_scale;
            }
            let _ = sprites.set_rect(
                handle,
                Some(PalRect::new(
                    record.rect[0],
                    record.rect[1],
                    record.rect[2],
                    record.rect[3],
                )),
            );
            if let Some(sprite) = sprites.get_mut(handle) {
                sprite.offset = PalPoint2 {
                    x: record.offset_x,
                    y: record.offset_y,
                };
            }
            sprites.restore_option(handle, record.option_type, &record.option_backup);
            if record.transition_source { self.game_sprite_transition_sources.insert(record.slot,handle); }
            else { self.game_sprites.insert(record.slot, handle); }
        }
        self.button_groups.clear();
        for group in &snapshot.button_groups {
            self.button_groups.insert(
                group.group,
                GameButtonGroup {
                    normal_image: group.normal_image,
                    hover_image: group.hover_image,
                    onmouse_index: group.onmouse_index,
                    offmouse_index: group.offmouse_index,
                    push_index: group.push_index,
                    slider_ended: group.slider_ended,
                    raw_toggle: group.raw_toggle,
                    locked: group.locked,
                    lock_started_ms: group.lock_started_ms,
                    lock_duration_ms: group.lock_duration_ms,
                },
            );
        }
        self.next_button_registration = snapshot.buttons.iter().map(|button| button.registration_order).max().unwrap_or(0).wrapping_add(1);
        for record in &snapshot.buttons {
            let Some(handle) = sprites.create_rgba_sprite(
                record.width,
                record.height,
                record.rgba.clone(),
                PalVec3::from_f32(record.x as f32, record.y as f32, record.z as f32),
                record.priority,
                format!("save-button:{}:{}", record.group, record.index),
            ) else {
                continue;
            };
            // Button sheets stack one cell per state; restoring the full
            // texture without its cell window draws every state at once.
            let _ = sprites.set_offset_rect(handle, record.cell_width, record.cell_height);
            let _ = sprites.set_rect(
                handle,
                Some(PalRect::new(
                    record.rect[0],
                    record.rect[1],
                    record.rect[2],
                    record.rect[3],
                )),
            );
            let _ = sprites.set_color(handle, PalColor(record.color));
            let _ = sprites.view_ctrl(handle, record.visible);
            self.game_buttons.insert(
                (record.group, record.index),
                GameButtonEntry {
                    registration_order: record.registration_order,
                    native_flags: record.native_flags,
                    hover_callbacks: record.hover_callbacks,
                    handle,
                    name: record.name.clone(),
                    visible: record.visible,
                    enabled: record.enabled,
                    locked: self.button_groups.get(&record.group).is_some_and(|group| group.locked),
                    toggle: record.base_row,
                    alpha: record.alpha,
                    slider_offset: 0,
                    base_position: record.base_position,
                    slider_position: record.slider_position,
                    hit_rect: None,
                    gosub_point: (record.gosub_point > 0).then_some(record.gosub_point as u32),
                    anim_resource: None,
                    anim_play_flag: 0,
                },
            );
        }
        self.restore_actions(&snapshot.actions, sprites);
        self.restore_children(&snapshot.children, sprites);
        self.restore_ui(snapshot.ui.as_ref());
        self.text_state.dirty = true;
    }

    fn release_scene_for_load(&mut self, sprites: &mut SpriteSystem) {
        self.clear_history_icon_scene(sprites);
        self.button_children.clear();
        self.retire_button_sounds(-1);
        self.pending_button_reactions.clear();
        self.button_sound_hover.clear();
        self.button_callback_hover.clear();
        self.system_button_blocked_at = None;
        self.clear_button_capture(-1, -1);
        self.finish_full_movie(sprites);
        self.pending_registered_se_wait = None;
        self.action_state = ActionSubsystemState::default();
        self.action_audio_requests.clear();
        sprites.release_global_raster();
        self.pending_global_fx_release = false;
        self.clear_select_scene(Some(sprites));
        for transition in self.game_sprite_transitions.values().copied() {
            sprites.release_transition_handle(transition);
        }
        self.game_sprite_transitions.clear();
        for handle in self.game_sprite_transition_sources.values().copied() {
            sprites.release(handle);
        }
        self.game_sprite_transition_sources.clear();
        for handle in self.game_sprites.values().copied() {
            sprites.release(handle);
        }
        self.game_sprites.clear();
        self.game_sprite_pending_position.clear();
        self.game_sprite_active_alpha.clear();
        self.game_sprite_pending_alpha.clear();
        self.game_sprite_pending_named_animations.clear();
        self.game_sprite_animations.clear();
        self.game_sprite_placements.clear();
        self.game_sprite_native_scales.clear();
        self.game_sprite_wrapper_visuals.clear();
        self.game_sprite_child_lanes.clear();
        self.game_sprite_child_offsets.clear();
        self.game_sprite_render_lanes.clear();
        self.game_sprite_anim_params.clear();
        self.game_sprite_aspect_position_types.clear();
        self.game_sprite_vis_clip_slots.clear();
        self.game_face_slots.clear();
        self.game_sprite_base_alpha.clear();
        self.game_sprite_final_alpha_delta.clear();
        for state in self.game_msprites.values() {
            if let Some(handle) = state.handle {
                self.msprite_system.release(handle);
            }
        }
        self.game_msprites.clear();
        self.pending_msp_wait_slot = None;
        if let Some(handle) = self.backbuffer_sprite.take() {
            sprites.release(handle);
        }
        if let Some(handle) = self.text_state.sprite.take() {
            sprites.release(handle);
        }
        if let Some(handle) = self.text_state.name_sprite.take() {
            sprites.release(handle);
        }
        self.clear_save_drawings(sprites, -1);
        for entry in self.game_buttons.values() {
            sprites.release(entry.handle);
        }
        self.game_buttons.clear();
        self.button_push_queue.clear();
        // A load also drops any transient scene-skip mode; the script re-arms
        // skip or auto through its own handlers if the resumed scene wants
        // them.
        self.scene_skip_active = false;
        // system_buttons survive load: the script registers the target
        // scene's shortcut bindings before issuing load (koikake's CONTINUE /
        // DATA LOAD handlers bind the ADV slots 0..18 up front), and the
        // parked resume does not re-register them.
        for retired in self.retired_sprites.drain(..) {
            sprites.release(retired.handle);
        }
    }

    fn bgm_wrapper_result(assets: &CoreAssets, slot: i32) -> ExtCallOutcome {
        if assets.extended_softpal || assets.registered_softpal { ExtCallOutcome::Value(slot) }
        else { ExtCallOutcome::Continue }
    }

    fn bgm_fade_duration(&self, flags: i32, duration: i32) -> i32 {
        let duration = if self.dispatch_skip_state() != 0 { (duration as u32) >> 4 } else { duration as u32 };
        duration.wrapping_mul((flags & 4) as u32) as i32
    }

    fn load_bgm_native(
        &mut self, assets: &CoreAssets, slot: i32, name: &str, flags: i32,
        resources: Option<&mut ResourceManager>, audio: &mut AudioSystem,
    ) -> bool {
        if let Some(old) = self.game_audio.remove(&(4, slot)) { let _ = audio.release(old); }
        let skip_mask = if assets.registered_softpal || assets.extended_softpal { 0x30 } else { 0x10 };
        let suppress_load = self.dispatch_skip_state() & skip_mask != 0;
        let loaded = if suppress_load {
            if assets.registered_softpal {
                let raw = audio.load_set_reserved(0, 3);
                if raw != 0 {
                    self.game_audio.insert((4, slot), AudioHandle::from_raw(raw).unwrap());
                }
            }
            false
        } else {
            self.load_named_audio(4, slot, PalSoundGroup::GROUP3, name, flags, 100,
                false, None, resources, Some(audio))
        };
        // The EXE retains its script description even when PAL returns zero or
        // reserves an empty entry. PAL itself stores no deferred filename.
        self.bgm_slots.insert(slot, BgmSlotState { name: name.to_owned(), native_flags: flags,
            looping: flags & 1 != 0, playing: false, loop_samples: None });
        if assets.registered_softpal {
            let other = 1 - slot;
            if self.game_audio.get(&(4, slot)).is_some_and(|handle| self.game_audio.get(&(4, other)) == Some(handle)) {
                self.game_audio.remove(&(4, other)); self.bgm_slots.remove(&other);
            }
        }
        loaded
    }

    fn ext_bgm_play(
        &mut self, assets: &CoreAssets, nls: Nls,
        resource_manager: Option<&mut ResourceManager>, audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(7);
        if args.len() != 7 { return ExtCallOutcome::Block; }
        let slot = if args[0] == -1 { 0 } else { args[0] };
        if !(0..2).contains(&slot) { return ExtCallOutcome::Continue; }
        let Some(name) = self.resolve_resource_string(args[1], assets, nls) else {
            return Self::bgm_wrapper_result(assets, slot);
        };
        let Some(audio) = audio else {
            return Self::bgm_wrapper_result(assets, slot);
        };
        audio.set_pal_time(self.pal_time_ms);
        if args[0] == -1 {
            if assets.registered_softpal {
                for previous_slot in 0..2 {
                    let preload = self.bgm_slots.get(&previous_slot).is_some_and(|state| state.native_flags & 0x2000_0000 != 0);
                    if !preload && audio.sound_param_by_type(3, previous_slot as u32, 18).unwrap_or(0) == 0 {
                        self.game_audio.remove(&(4, previous_slot)); self.bgm_slots.remove(&previous_slot);
                    }
                }
            }
            let free = audio.free_channel_count(PalSoundGroup::GROUP3);
            if free == 0 {
                let _ = audio.release_group(PalSoundGroup::GROUP3);
                self.game_audio.remove(&(4, 0)); self.game_audio.remove(&(4, 1));
                self.bgm_slots.remove(&0); self.bgm_slots.remove(&1);
            } else if free == 1 && (!assets.registered_softpal || !self.game_audio.contains_key(&(4, 1))) {
                if let Some(previous) = self.game_audio.remove(&(4, 0)) { self.game_audio.insert((4, 1), previous); }
                if let Some(previous) = self.bgm_slots.remove(&0) { self.bgm_slots.insert(1, previous); }
            }
        }
        // A normal native Play loads into a free PAL bank entry without first
        // releasing the old script-slot handle. Auto slot -1 manages both banks.
        self.game_audio.remove(&(4, slot));
        self.bgm_slots.remove(&slot);
        if self.load_bgm_native(assets, slot, &name, args[2], resource_manager, audio) {
            let handle = self.game_audio[&(4, slot)];
            let range = crate::abi::PalSoundLoadRange32 { enabled: (args[2] & 0x10) as u32,
                initial_start_bytes: args[4] as u32, loop_start_bytes: args[5] as u32,
                loop_end_bytes: args[6] as u32 };
            if let Err(error) = audio.set_load_range(handle, range) {
                log::warn!("[trace-audio] BGM native range could not be applied: {error}");
            }
            let duration = self.bgm_fade_duration(args[2], args[3]);
            if let Err(error) = audio.play_native(handle, args[2] & 1, duration) {
                log::warn!("[trace-audio] BGM native PlayFade failed: {error}");
            }
            if let Some(state) = self.bgm_slots.get_mut(&slot) {
                state.playing = true;
                state.looping = args[2] & 1 != 0;
                state.loop_samples = audio.loop_samples(handle).ok();
            }
        }
        Self::bgm_wrapper_result(assets, slot)
    }

    fn ext_bgm_load(
        &mut self, assets: &CoreAssets, nls: Nls,
        resource_manager: Option<&mut ResourceManager>, audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() != 5 { return ExtCallOutcome::Block; }
        let slot = args[0];
        if !(0..2).contains(&slot) { return ExtCallOutcome::Continue; }
        let Some(name) = self.resolve_resource_string(args[1], assets, nls) else {
            return Self::bgm_wrapper_result(assets, slot);
        };
        let Some(audio) = audio else {
            return Self::bgm_wrapper_result(assets, slot);
        };
        if self.load_bgm_native(assets, slot, &name, 0x2000_0000, resource_manager, audio) {
            let handle = self.game_audio[&(4, slot)];
            let range = crate::abi::PalSoundLoadRange32 { enabled: u32::from(args[2] | args[3] | args[4] != 0),
                initial_start_bytes: args[2] as u32, loop_start_bytes: args[3] as u32,
                loop_end_bytes: args[4] as u32 };
            if let Err(error) = audio.set_load_range(handle, range) {
                log::warn!("[trace-audio] BGM preload range could not be applied: {error}");
            }
            if let Some(state) = self.bgm_slots.get_mut(&slot) { state.loop_samples = audio.loop_samples(handle).ok(); }
        }
        Self::bgm_wrapper_result(assets, slot)
    }

    fn ext_bgm_play_loaded(&mut self, assets: &CoreAssets, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() != 3 { return ExtCallOutcome::Block; }
        let slot = args[0];
        if !(0..2).contains(&slot) { return ExtCallOutcome::Continue; }
        let duration = self.bgm_fade_duration(args[1], args[2]);
        if let (Some(handle), Some(audio)) = (self.game_audio.get(&(4, slot)).copied(), audio) {
            audio.set_pal_time(self.pal_time_ms);
            let played = audio.play_native(handle, args[1] & 1, duration);
            if let Err(error) = &played {
                log::warn!("[trace-audio] BGM loaded PlayFade failed: {error}");
            }
            if let Some(state) = self.bgm_slots.get_mut(&slot) {
                state.playing = played.is_ok() && audio.channel_volume(handle).is_ok();
                state.looping = args[1] & 1 != 0; state.native_flags = args[1];
            }
        }
        Self::bgm_wrapper_result(assets, slot)
    }

    fn ext_bgm_stop(&mut self, assets: &CoreAssets, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() != 2 { return ExtCallOutcome::Block; }
        let duration = if self.dispatch_skip_state() != 0 { (args[1] as u32) >> 4 } else { args[1] as u32 };
        if let Some(audio) = audio {
            audio.set_pal_time(self.pal_time_ms);
            for slot in 0..2 {
                if args[0] != -1 && args[0] != slot { continue; }
                if let Some(&handle) = self.game_audio.get(&(4, slot)) {
                    if assets.registered_softpal { audio.set_reserved(3, handle.raw(), 0); }
                    let _ = audio.stop_fade(handle, duration);
                }
                self.bgm_slots.remove(&slot);
            }
        }
        ExtCallOutcome::Continue
    }

    fn ext_bgm_set_volume(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        self.bgm_volume_percent = clamp_percent(args.first().copied().unwrap_or(100));
        let volume = if self.bgm_muted {
            PalVolume::MIN
        } else {
            percent_to_volume(self.bgm_volume_percent)
        };
        if let Some(audio) = audio {
            let _ = audio.set_group_volume(PalSoundGroup::GROUP3, volume);
        }
        ExtCallOutcome::Value(1)
    }

    /// Game.exe `sub_40BFB0` (`bgm_set_auto_volume`) pops
    /// `(enable, volume_percent)`, stores both fields, and when auto volume is
    /// disabled restores the ordinary BGM group volume.  The portable engine
    /// does not yet implement full voice-ducking, but it must preserve the
    /// setting page state and avoid the shared fallback path.
    fn ext_bgm_set_auto_volume(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let enabled = args.first().copied().unwrap_or(0) != 0;
        let volume = clamp_percent(args.get(1).copied().unwrap_or(self.bgm_auto_volume_percent));
        self.bgm_auto_muted = !enabled;
        self.bgm_auto_volume_percent = volume;
        if !enabled {
            self.apply_bgm_group_volume(audio);
        }
        ExtCallOutcome::Value(1)
    }

    /// Game.exe `sub_40BD40` (`set_master_volume`) pops one script percent,
    /// stores PAL raw volume as `percent * 100`, and writes zero when the master
    /// mute latch is active.
    fn ext_set_master_volume(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        self.master_volume_percent = clamp_percent(args.first().copied().unwrap_or(100));
        self.apply_master_volume(audio);
        ExtCallOutcome::Value(1)
    }

    /// Game.exe `sub_40BC70` (`mute_master_volume`) pops a mute flag.  Non-zero
    /// means muted; zero restores the previously configured master volume.
    fn ext_mute_master_volume(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        self.master_muted = args.first().copied().unwrap_or(1) != 0;
        self.apply_master_volume(audio);
        ExtCallOutcome::Value(1)
    }

    /// Game.exe `sub_40C230` (`bgm_mute`) pops a mute flag and gates PAL sound
    /// group 3 without changing the configured BGM slider value.
    fn ext_bgm_mute(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        self.bgm_muted = args.first().copied().unwrap_or(1) != 0;
        self.apply_bgm_group_volume(audio);
        ExtCallOutcome::Value(1)
    }

    /// Game.exe `sub_40C060` (`mute_bgm_auto_volume`) pops one flag and stores
    /// whether automatic BGM volume ducking is enabled.  The actual ducking
    /// worker remains blocked, but the script-visible latch is no longer lost.
    fn ext_mute_bgm_auto_volume(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        self.bgm_auto_muted = args.first().copied().unwrap_or(1) != 0;
        ExtCallOutcome::Value(1)
    }

    fn apply_master_volume(&self, audio: Option<&mut AudioSystem>) {
        if let Some(audio) = audio {
            let volume = if self.master_muted {
                PalVolume::MIN
            } else {
                percent_to_volume(self.master_volume_percent)
            };
            let _ = audio.set_primary_volume(volume);
        }
    }

    /// Script `start`/`end` override the CSV when `end > start`. Otherwise the
    /// row in `BGM.CSV` supplies PCM sample loop points. Missing rows loop the
    /// whole file when the play flag requests looping.
    fn bgm_loop_samples_for(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
        assets: &CoreAssets,
        nls: Nls,
        name_value: i32,
        script_start: i32,
        script_end: i32,
    ) -> Option<(i64, i64)> {
        if script_end > script_start {
            return Some((i64::from(script_start), i64::from(script_end)));
        }
        let Some(name) = self.resolve_resource_string(name_value, assets, nls) else {
            return Some((0, 0));
        };
        let Some(resource_manager) = resource_manager else {
            return Some((0, 0));
        };
        self.ensure_bgm_loops(resource_manager);
        let row = self
            .bgm_loops
            .as_ref()
            .and_then(|table| table.get(&audio_lookup_key(&name)))
            .copied();
        Some(
            row.map(|row| (row.loop_start_samples, row.loop_end_samples))
                .unwrap_or((0, 0)),
        )
    }

    fn ensure_bgm_loops(&mut self, resource_manager: &mut ResourceManager) {
        if self.bgm_loops.is_some() {
            return;
        }
        let table = match resource_manager.open("BGM.CSV") {
            Ok(asset) => parse_bgm_csv(&asset.bytes),
            Err(err) => {
                log::debug!("[trace-audio] BGM.CSV unavailable: {err}");
                BTreeMap::new()
            }
        };
        log::debug!("[trace-audio] BGM.CSV rows={}", table.len());
        self.bgm_loops = Some(table);
    }

    fn apply_bgm_group_volume(&self, audio: Option<&mut AudioSystem>) {

        if let Some(audio) = audio {
            let volume = if self.bgm_muted {
                PalVolume::MIN
            } else {
                percent_to_volume(self.bgm_volume_percent)
            };
            let _ = audio.set_group_volume(PalSoundGroup::GROUP3, volume);
        }
    }

    /// Push the configured volume levels into the audio system.  Called once at
    /// boot after `load_portable_system_data`: the persisted settings otherwise
    /// only reach Kira when the script happens to call a volume extcall, so a
    /// muted or lowered configuration would still play at full volume.
    pub fn apply_persisted_audio_levels(&self, audio: &mut AudioSystem) {
        log::debug!(
            "[trace-audio] apply persisted levels: master={}%(muted={}) bgm={}%(muted={}) voice={}%(muted={}) se_slots={}",
            self.master_volume_percent,
            self.master_muted,
            self.bgm_volume_percent,
            self.bgm_muted,
            self.text_state.voice_volume,
            self.text_state.voice_muted,
            self.se_volume_percent.len(),
        );
        self.apply_master_volume(Some(audio));
        self.apply_bgm_group_volume(Some(audio));
        let voice_volume = if self.text_state.voice_muted {
            PalVolume::MIN
        } else {
            percent_to_volume(self.text_state.voice_volume)
        };
        let _ = audio.set_group_volume(PalSoundGroup::GROUP1, voice_volume);
        if !self.se_volume_percent.is_empty() {
            let _ = audio.set_group_volume(PalSoundGroup::GROUP4, self.effective_se_group_volume());
        }
    }

    fn ext_bgm_filename(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let value = args.first().copied().unwrap_or(0);
        let handle = self
            .resolve_resource_string(value, assets, nls)
            .map(|s| self.store_dynamic_string(s))
            .unwrap_or(0);
        ExtCallOutcome::Value(handle)
    }

    fn ext_se_play(
        &mut self,
        index: u16,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        if index == 0 {
            let arity = if assets.extended_softpal { 3 } else { 2 };
            let args = self.pop_ext_args(arity);
            if args.len() < arity {
                return ExtCallOutcome::Block;
            }
            let (slot, name_value) = if assets.extended_softpal {
                (args[1], args[0])
            } else {
                (args[0], args[1])
            };
            return self.audio_load_and_play(
                5,
                slot,
                PalSoundGroup::GROUP4,
                name_value,
                0,
                100,
                false,
                None,
                assets,
                nls,
                resource_manager,
                audio,
                0,
            );
        }
        if index == 1 && assets.extended_softpal {
            let args = self.pop_ext_args(3);
            if args.len() < 3 {
                return ExtCallOutcome::Block;
            }
            let slot = args[0];
            if let (Some(audio), Some(handle)) = (audio, self.game_audio.get(&(5, slot)).copied()) {
                let _ = audio.play(handle, args[2] != 0);
            }
            return ExtCallOutcome::Value(1);
        }
        let args = self.pop_ext_args(5);
        if args.len() < 5 {
            return ExtCallOutcome::Block;
        }
        let slot = if args[0] == -1 {
            self.next_free_audio_slot(5, 16)
        } else {
            args[0]
        };
        self.audio_load_and_play(
            5,
            slot,
            PalSoundGroup::GROUP4,
            args[1],
            args[3],
            args[4],
            true,
            None,
            assets,
            nls,
            resource_manager,
            audio,
            0,
        )
    }

    /// Load the `[sound] SOUND_UNIT_LISTFILE` voice registry once.  Each row is
    /// `"header","test_voice",registered` in unit-grid slot order; pseudo
    /// headers -1/-2/-3 are the vo99/vo98/vo97 catch-all buckets.
    fn ensure_voice_unit_headers(&mut self, resource_manager: &mut ResourceManager) {
        if self.voice_unit_headers.is_some() {
            return;
        }
        let list_name = self
            .system_ini(Some(resource_manager))
            .and_then(|ini| ini.get("sound"))
            .and_then(|section| section.get("sound_unit_listfile"))
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_else(|| "Sound.csv".to_owned());
        let headers = match resource_manager.open(&list_name) {
            Ok(asset) => match resource_manager.nls().decode(&asset.bytes) {
                Ok(text) => text
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with("//") {
                            return None;
                        }
                        let header = line.split(',').next()?.trim().trim_matches('"');
                        Some(header.to_owned())
                    })
                    .collect(),
                Err(_) => Vec::new(),
            },
            Err(_) => Vec::new(),
        };
        self.voice_unit_headers = Some(headers);
    }

    /// Map a voice resource name (`vo01_xxxxx`) to its unit-grid slot through
    /// the `Sound.csv` headers.  Unmatched `vo97`/`vo98`/`vo99` fall into the
    /// -3/-2/-1 catch-all rows; anything else yields `None`.
    fn voice_unit_slot_for_name(
        &mut self,
        resource_manager: &mut ResourceManager,
        name: &str,
    ) -> Option<i32> {
        self.ensure_voice_unit_headers(resource_manager);
        let headers = self.voice_unit_headers.as_deref()?;
        if headers.is_empty() {
            return None;
        }
        let lower = name.to_ascii_lowercase();
        let mut best: Option<(usize, usize)> = None;
        for (index, header) in headers.iter().enumerate() {
            let header = header.to_ascii_lowercase();
            let prefix = match header.parse::<i32>() {
                // Pseudo headers address the catch-all voice prefixes.
                Ok(-1) => "vo99".to_owned(),
                Ok(-2) => "vo98".to_owned(),
                Ok(-3) => "vo97".to_owned(),
                Ok(_) => continue,
                Err(_) => header,
            };
            if lower.starts_with(&prefix)
                && best.is_none_or(|(_, len)| prefix.len() > len)
            {
                best = Some((index, prefix.len()));
            }
        }
        best.map(|(index, _)| index as i32)
    }

    /// Effective per-channel voice percent for a unit slot: 0 when the
    /// character's voice is muted, otherwise its configured percent (default
    /// 100, neutral against the global group volume).
    fn voice_unit_channel_percent(&self, slot: i32) -> i32 {
        if self.voice_ex_enabled.get(&slot).copied() == Some(false) {
            return 0;
        }
        self.voice_ex_volume_percent
            .get(&slot)
            .copied()
            .unwrap_or(100)
            .clamp(0, 100)
    }

    /// Game.exe 0x442550 scans 16 registration slots, taking the first
    /// nonempty case-insensitive prefix. 0x442C10 bypasses it for flag 0x80.
    fn voice_autopan_match(&self, name: &str, flags: i32) -> Option<&VoiceAutopanEntry> {
        if !self.text_state.voice_autopan_enabled || flags & 0x80 != 0 {
            return None;
        }
        self.text_state.voice_autopan_entries.range(0..16).map(|(_, entry)| entry)
            .find(|entry| !entry.name.is_empty()
                && name.get(..entry.name.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(&entry.name)))
    }

    fn voice_autopan_value(&self, entry: &VoiceAutopanEntry, sprites: Option<&SpriteSystem>) -> Option<i32> {
        let center = if entry.mode != 0 {
            entry.target.clamp(0, 20) as f32 * 96.0
        } else if let Some(sprite) = self.game_sprites.get(&entry.target)
            .and_then(|handle| sprites.and_then(|sprites| sprites.get(*handle))) {
            sprite.position.x as i32 as f32 + (sprite.texture_size.width >> 1) as f32
        } else {
            let visual = self.game_sprite_wrapper_visuals.get(&entry.target)?;
            visual.native_x as i32 as f32 + (visual.width >> 1) as f32
        };
        let limit = self.text_state.voice_autopan_limit as f32;
        Some((center.clamp(0.0, 1920.0) / 1920.0 * (2.0 * limit) - limit) as i32)
    }

    /// Native AdvMessageProc (0x4457F0) updates all nine active voice slots,
    /// including while ADV execution is parked waiting for player input.
    fn update_voice_autopan(&mut self, sprites: Option<&SpriteSystem>, audio: Option<&mut AudioSystem>) {
        let Some(audio) = audio else { return; };
        if !self.text_state.voice_autopan_enabled { return; }
        self.text_state.voice_autopan_playing.retain(|slot, (handle, _)|
            self.game_audio.get(&(13, *slot)) == Some(handle));
        for (handle, entry) in self.text_state.voice_autopan_playing.values() {
            if let Some(pan) = self.voice_autopan_value(entry, sprites) {
                if audio.channel_pan(*handle).ok() != Some(pan.clamp(-10_000, 10_000)) {
                    let _ = audio.set_channel_pan(*handle, pan);
                }
            }
        }
    }

    fn ext_voice_play(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        mut resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() < 4 {
            return ExtCallOutcome::Block;
        }
        let slot = if args[0] == -1 {
            self.next_free_audio_slot(13, 8)
        } else {
            args[0]
        };
        // Game.exe sub_4447B0 passes the fourth argument to sub_442C10 as the
        // fade time used only by `PalSoundPlayFade` when `flags & 4` is set.
        // Voice loudness is controlled separately by voice_set_volume /
        // voice_mute through PAL sound group volume, so treating this field as
        // channel volume makes the common `voice_play(..., fade_ms=0)` path
        // fully silent.
        // Per-character volume/mute from the SOUND unit grid rides the PAL
        // channel volume so it composes with the group-wide voice volume.
        let unit_percent = self
            .resolve_resource_string(args[1], assets, nls)
            .zip(resource_manager.as_deref_mut())
            .and_then(|(name, rm)| {
                self.voice_unit_slot_for_name(rm, &name)
                    .map(|slot| self.voice_unit_channel_percent(slot))
            })
            .unwrap_or(100);
        self.audio_load_and_play(
            13,
            slot,
            PalSoundGroup::GROUP1,
            args[1],
            args[2],
            unit_percent,
            true,
            None,
            assets,
            nls,
            resource_manager,
            audio,
            args[3],
        )
    }

    /// Game.exe 0x443150 caches a BGV percentage and updates PAL group 2.
    fn ext_bgv_set_volume(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(&percent) = args.first() else {
            return ExtCallOutcome::Block;
        };
        self.text_state.bgv_volume = percent;
        self.apply_bgv_group_volume(audio);
        ExtCallOutcome::Value(1)
    }

    /// Game.exe 0x443080 retains the configured percentage while muted.
    fn ext_bgv_mute(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(&flag) = args.first() else {
            return ExtCallOutcome::Block;
        };
        self.text_state.bgv_muted = flag != 0;
        self.apply_bgv_group_volume(audio);
        ExtCallOutcome::Value(1)
    }

    fn apply_bgv_group_volume(&self, audio: Option<&mut AudioSystem>) {
        if let Some(audio) = audio {
            let volume = if self.text_state.bgv_muted {
                PalVolume::MIN
            } else {
                percent_to_volume(self.text_state.bgv_volume)
            };
            let _ = audio.set_group_volume(PalSoundGroup::GROUP2, volume);
        }
    }

    /// Game.exe `sub_4441E0` (`voice_mute`) pops one mute flag, stores the
    /// voice enable latch at ctx[164910] as `flag == 0`, and writes either the
    /// cached voice volume or zero to PAL sound group 1.  Some script tables
    /// reach this as `0x000D:0007`, while the generated opcode aliases also
    /// expose `0x000D:0018`; both dispatch here.
    fn ext_voice_mute(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        self.text_state.voice_muted = args.first().copied().unwrap_or(1) != 0;
        if let Some(audio) = audio {
            let volume = if self.text_state.voice_muted {
                PalVolume::MIN
            } else {
                percent_to_volume(self.text_state.voice_volume)
            };
            let _ = audio.set_group_volume(PalSoundGroup::GROUP1, volume);
        }
        log::debug!(
            "[trace-audio] voice_mute raw_flag={} muted={} cached_percent={}",
            args.first().copied().unwrap_or(1),
            self.text_state.voice_muted,
            self.text_state.voice_volume
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_audio_stop(
        &mut self,
        category: u16,
        group: PalSoundGroup,
        audio: Option<&mut AudioSystem>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let slot = args.first().copied().unwrap_or(-1);
        log::debug!("[trace-audio] stop category={category} slot={slot}");
        let Some(audio) = audio else {
            return ExtCallOutcome::Value(1);
        };
        if slot == -1 {
            let keys = self
                .game_audio
                .keys()
                .filter(|(cat, _)| *cat == category)
                .copied()
                .collect::<Vec<_>>();
            for key in keys {
                if let Some(handle) = self.game_audio.remove(&key) {
                    let _ = audio.release(handle);
                }
            }
            let _ = audio.release_group(group);
            if category == 4 {
                self.bgm_slots.clear();
            }
        } else if let Some(handle) = self.game_audio.remove(&(category, slot)) {
            let _ = audio.release(handle);
            if category == 4 {
                self.bgm_slots.remove(&slot);
            }
        }
        ExtCallOutcome::Value(1)
    }

    fn ext_se_wait(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let slot = args.first().copied().unwrap_or(-1);
        let handles = self
            .game_audio
            .iter()
            .filter_map(|((category, audio_slot), handle)| {
                (*category == 5 && (slot < 0 || *audio_slot == slot)).then_some(*handle)
            })
            .collect::<Vec<_>>();
        let Some(audio) = audio else {
            log::debug!("[trace-audio] se_wait slot={slot} no audio backend");
            return ExtCallOutcome::Value(1);
        };
        let any_playing = handles
            .iter()
            .copied()
            .any(|handle| audio.is_playing(handle).unwrap_or(false));
        if any_playing {
            log::debug!("[trace-audio] se_wait slot={slot} still playing");
            return ExtCallOutcome::Wait {
                value: 1,
                request: WaitRequest::Frame(1),
            };
        }
        log::debug!("[trace-audio] se_wait slot={slot} complete");
        ExtCallOutcome::Value(1)
    }

    fn effective_se_group_volume(&self) -> PalVolume {
        let mut best_percent = 0;
        for (&slot, &percent) in &self.se_volume_percent {
            let enabled = self.se_enabled.get(&slot).copied().unwrap_or(true);
            if enabled {
                best_percent = best_percent.max(clamp_percent(percent));
            }
        }
        percent_to_volume(best_percent)
    }

    /// Game.exe `sub_434D00` (`se_mute`) pops `(mute_flag, slot)`: `v9` is the
    /// top-of-stack mute flag and `v4` is the SE lane.  Native code stores
    /// `enabled = (mute_flag == 0)` at `ctx + 659648 + slot*4` and applies
    /// `cached_volume[slot] * enabled` to `dword_49C264[slot]`.  The portable
    /// backend has one SE group volume, so it keeps the same per-lane latches
    /// and projects them to group 4 with the loudest currently enabled lane.
    fn ext_se_mute(&mut self, audio: Option<&mut AudioSystem>) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        let raw_flag = args.first().copied().unwrap_or(1);
        let slot = args.get(1).copied().unwrap_or(0);
        let enabled = raw_flag == 0;
        self.se_enabled.insert(slot, enabled);
        self.se_muted.insert(slot, !enabled);
        let percent = self.se_volume_percent.get(&slot).copied().unwrap_or(100);
        if let Some(audio) = audio {
            let _ = audio.set_group_volume(PalSoundGroup::GROUP4, self.effective_se_group_volume());
        }
        log::debug!(
            "[trace-audio] se_mute raw_flag={raw_flag} slot={slot} enabled={enabled} cached_percent={percent} effective_group_raw={}",
            self.effective_se_group_volume().raw()
        );
        ExtCallOutcome::Value(1)
    }

    #[allow(clippy::too_many_arguments)]
    fn audio_load_and_play(
        &mut self,
        category: u16,
        slot: i32,
        group: PalSoundGroup,
        name_value: i32,
        flags: i32,
        volume_percent: i32,
        play: bool,
        loop_samples: Option<(i64, i64)>,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
        voice_fade_ms: i32,
    ) -> ExtCallOutcome {
        let Some(name) = self.resolve_resource_string(name_value, assets, nls) else {
            return ExtCallOutcome::Value(0);
        };
        if name.is_empty() {
            return ExtCallOutcome::Value(slot);
        }
        if is_resource_clear_sentinel(&name) {
            log::debug!(
                "[trace-audio] open category={category} slot={slot} sentinel=# action=noop"
            );
            return ExtCallOutcome::Value(slot);
        }
        // Title and menu scripts can request the current looping BGM again on
        // return. Keep its playback position when the requested track and loop
        // region are unchanged. Explicit stops clear the slot, and a finished
        // one-shot is allowed to start again.
        if category == 4 && play {
            if let (Some(state), Some(handle), Some(audio)) = (
                self.bgm_slots.get(&slot),
                self.game_audio.get(&(category, slot)),
                audio.as_ref(),
            ) {
                if state.playing
                    && state.name.eq_ignore_ascii_case(&name)
                    && state.looping == (flags & 1 != 0)
                    && state.loop_samples == loop_samples
                    && (!audio.is_enabled() || audio.is_playing(*handle).unwrap_or(false))
                {
                    log::debug!(
                        "[trace-audio] bgm reuse slot={slot} name={name:?} action=keep_playback"
                    );
                    return ExtCallOutcome::Value(slot);
                }
            }
        }
        if category == 13 {
            return if self.load_voice_audio(slot, group, &name, flags, volume_percent,
                play, voice_fade_ms, resource_manager, audio) {
                ExtCallOutcome::Value(slot)
            } else {
                ExtCallOutcome::Value(0)
            };
        }
        if self.load_named_audio(
            category,
            slot,
            group,
            &name,
            flags,
            volume_percent,
            play,
            loop_samples,
            resource_manager,
            audio,
        ) {
            ExtCallOutcome::Value(slot)
        } else {
            ExtCallOutcome::Value(0)
        }
    }

    /// Game.exe 0x442C10 gives a MIX up to eight independent voice lanes,
    /// replacing slots 0..7 regardless of the requested slot. Each member gets
    /// its own name-based volume and autopan; slot 8 remains independent.
    #[allow(clippy::too_many_arguments)]
    fn load_voice_audio(
        &mut self,
        slot: i32,
        group: PalSoundGroup,
        name: &str,
        flags: i32,
        volume_percent: i32,
        play: bool,
        fade_ms: i32,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> bool {
        let (Some(resources), Some(audio)) = (resource_manager, audio) else {
            return false;
        };
        // Native probes the playlist before opening the ordinary sound.
        let mix = if name.to_ascii_lowercase().ends_with(".mix") {
            resources.open(name).ok()
        } else {
            resources.open(&format!("{name}.mix"))
                .or_else(|_| resources.open(&format!("{name}.MIX"))).ok()
        };
        let Some(mix) = mix else {
            let percent = self.native_voice_channel_percent(name, flags)
                .unwrap_or(volume_percent);
            let loaded = self.load_named_audio(13, slot, group, name, flags,
                percent, false,
                None, Some(resources), Some(audio));
            if loaded && play {
                let handle = self.game_audio[&(13, slot)];
                if let Err(error) = audio.play_fade(handle, flags & 1 != 0,
                    if flags & 4 != 0 { fade_ms } else { 0 }) {
                    log::warn!("[trace-audio] voice play failed: {error}");
                }
            }
            return loaded;
        };
        let Some(members) = parse_mix_playlist(&mix.bytes) else {
            log::warn!("[trace-audio] invalid voice MIX {:?}", mix.name);
            return false;
        };
        self.release_adv_voice_slots(audio);
        let mut handles = Vec::new();
        for (lane, member) in members.iter().take(8).enumerate() {
            let percent = self.native_voice_channel_percent(member, flags).unwrap_or_else(|| {
                self.voice_unit_slot_for_name(resources, member)
                    .map(|unit| self.voice_unit_channel_percent(unit)).unwrap_or(100)
            });
            if self.load_named_audio(13, lane as i32, group, member, flags,
                percent, false, None, Some(resources), Some(audio)) {
                handles.push(self.game_audio[&(13, lane as i32)]);
            }
        }
        if play {
            for &handle in &handles {
                if let Err(error) = audio.play_fade(handle, flags & 1 != 0,
                    if flags & 4 != 0 { fade_ms } else { 0 }) {
                    log::warn!("[trace-audio] MIX member play failed: {error}");
                }
            }
        }
        !handles.is_empty()
    }

    fn release_adv_voice_slots(&mut self, audio: &mut AudioSystem) {
        for slot in 0..8 {
            if let Some(handle) = self.game_audio.remove(&(13, slot)) {
                let _ = audio.release(handle);
            }
            self.text_state.voice_autopan_playing.remove(&slot);
        }
    }

    /// Loads a resolved resource name into `(category, slot)` and optionally
    /// plays it. Returns false when the resource or audio backend failed;
    /// category-4 slots mirror the outcome into `bgm_slots` for save snapshots.
    #[allow(clippy::too_many_arguments)]
    fn load_named_audio(
        &mut self,
        category: u16,
        slot: i32,
        group: PalSoundGroup,
        name: &str,
        flags: i32,
        volume_percent: i32,
        play: bool,
        loop_samples: Option<(i64, i64)>,
        resource_manager: Option<&mut ResourceManager>,
        audio: Option<&mut AudioSystem>,
    ) -> bool {
        let (Some(resource_manager), Some(audio)) = (resource_manager, audio) else {
            return false;
        };
        if let Some(old) = self.game_audio.remove(&(category, slot)) {
            let _ = audio.release(old);
        }
        if category == 13 {
            self.text_state.voice_autopan_playing.remove(&slot);
        }
        if category == 4 {
            // The slot's previous track is gone even if the new load fails.
            self.bgm_slots.remove(&slot);
        }
        let extensions = if category == bgv::BGV_NAMESPACE { bgv::BGV_EXTENSIONS } else { AUDIO_EXTENSIONS };
        let asset = match open_resource_variant(resource_manager, name, extensions) {
            Ok(asset) => asset,
            Err(err) => {
                log::warn!("[trace-audio] open category={category} slot={slot} name={name:?} failed: {err}");
                return false;
            }
        };
        #[cfg(target_os = "vita")]
        let streaming = (group == PalSoundGroup::GROUP3 && asset.bytes.starts_with(b"OggS"))
            .then(|| audio.load_streaming_ogg(asset.name.clone(), &asset.bytes, group));
        #[cfg(not(target_os = "vita"))]
        let streaming: Option<anyhow::Result<AudioHandle>> = None;
        let loaded = if let Some(loaded) = streaming { loaded } else {
            // MIX playlists and non-Ogg sources retain their existing decoder.
            let decoded = decode_game_audio(&asset.bytes, &mut |member| {
                let opened = open_resource_variant(resource_manager, member, AUDIO_EXTENSIONS)?;
                Ok(opened.bytes)
            });
            let data = match decoded {
                Ok(data) => data,
                Err(err) => {
                    log::warn!(
                        "[trace-audio] decode category={category} slot={slot} asset={:?} failed: {err}",
                        asset.name
                    );
                    return false;
                }
            };
            let source_format = match crate::audio::source_pcm_format(&asset.bytes, &data) {
                Ok(format) => format,
                Err(error) => {
                    log::warn!("[trace-audio] source PCM metadata unavailable for {:?}: {error}", asset.name);
                    None
                }
            };
            let gain = if category == 5 && (0..16).contains(&slot) && group == PalSoundGroup::GROUP4 {
                self.registered_se_controls[slot as usize].load_gain()
            } else { 0.0 };
            let loaded = if gain > 1.0 {
                audio.load_static_data_ex(asset.name.clone(), data, group, gain)
            } else {
                audio.load_static_data(asset.name.clone(), data, group)
            };
            loaded.map(|handle| {
                if let Some(format) = source_format { let _ = audio.set_source_format(handle, format); }
                handle
            })
        };
        let handle = match loaded {
            Ok(handle) => handle,
            Err(err) => {
                log::warn!(
                    "[trace-audio] load category={category} slot={slot} asset={:?} failed: {err}",
                    asset.name
                );
                return false;
            }
        };
        if let Some((start, end)) = loop_samples {
            let _ = audio.set_loop_samples(handle, start, end);
        }
        let volume = PalVolume::from_raw(volume_percent.clamp(0, 100).saturating_mul(100));
        let _ = audio.set_channel_volume(handle, volume);
        if category == 13 {
            if let Some(entry) = self.voice_autopan_match(name, flags).cloned() {
                if let Some(pan) = self.voice_autopan_value(&entry, None) {
                    let _ = audio.set_channel_pan(handle, pan);
                }
                self.text_state.voice_autopan_playing.insert(slot, (handle, entry));
            }
        }
        let effective_volume = audio
            .effective_volume(handle)
            .map(|volume| volume.raw())
            .unwrap_or(0);
        if play {
            let looping = (flags & 1) != 0;
            if let Err(err) = audio.play(handle, looping) {
                log::warn!(
                    "[trace-audio] play category={category} slot={slot} asset={:?} failed: {err}",
                    asset.name
                );
            }
        }
        self.game_audio.insert((category, slot), handle);
        if category == 4 {
            self.bgm_slots.insert(
                slot,
                BgmSlotState {
                    name: name.to_owned(),
                    native_flags: flags,
                    looping: (flags & 1) != 0,
                    playing: play,
                    loop_samples,
                },
            );
        }
        log::debug!(
            "[trace-audio] category={category} slot={slot} asset={:?} group={:?} channel_raw={} effective_raw={} play={play} flags=0x{flags:08X}",
            asset.name,
            group,
            volume.raw(),
            effective_volume
        );
        true
    }

    /// Replays the BGM slots carried by a restored save snapshot (version 5).
    /// The outgoing scene's category-4 channels are released first so a title
    /// or menu track does not keep playing under the restored scene.  Runs at
    /// the top of the next frame after `load`, where the audio backend is
    /// reachable regardless of the restored wait state.
    fn replay_restored_bgm(
        &mut self,
        mut resource_manager: Option<&mut ResourceManager>,
        mut audio: Option<&mut AudioSystem>,
    ) {
        if !self.bgm_replay_pending {
            return;
        }
        self.bgm_replay_pending = false;
        let old_keys = self
            .game_audio
            .keys()
            .filter(|(category, _)| *category == 4)
            .copied()
            .collect::<Vec<_>>();
        for key in old_keys {
            if let Some(handle) = self.game_audio.remove(&key) {
                if let Some(audio) = audio.as_deref_mut() {
                    let _ = audio.release(handle);
                }
            }
        }
        let tracks = self
            .bgm_slots
            .iter()
            .filter(|(_, state)| state.playing)
            .map(|(&slot, state)| (slot, state.name.clone(), state.looping, state.loop_samples))
            .collect::<Vec<_>>();
        for (slot, name, looping, loop_samples) in tracks {
            let flags = i32::from(looping);
            let loaded = self.load_named_audio(
                4,
                slot,
                PalSoundGroup::GROUP3,
                &name,
                flags,
                100,
                true,
                loop_samples,
                resource_manager.as_deref_mut(),
                audio.as_deref_mut(),
            );
            if loaded {
                log::debug!("[trace-audio] restored bgm slot={slot} name={name:?}");
            } else {
                log::warn!("[trace-audio] restored bgm slot={slot} name={name:?} failed");
            }
        }
    }

    fn next_free_audio_slot(&self, category: u16, limit: i32) -> i32 {
        (0..limit)
            .find(|slot| !self.game_audio.contains_key(&(category, *slot)))
            .unwrap_or(0)
    }

    /// Game category 18 index 1 (`sub_41AB90`) pops one resolved string and
    /// calls ShellExecuteA("open", path).  The portable runtime must not launch
    /// host programs during tests/headless runs, but it preserves the exact
    /// argument contract and records the requested target.
    fn ext_app_exec(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.len() != 1 {
            return ExtCallOutcome::Block;
        }
        let path = self
            .resolve_script_string(args[0], assets, nls)
            .or_else(|| self.resolve_resource_string(args[0], assets, nls))
            .unwrap_or_default();
        log::debug!("[trace-system] app_exec path={path:?} portable_no_launch=true");
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 3 (`sub_41EBD0`) pops two string ids, resolves
    /// both through the Game string helper, lowercases the payloads, and returns
    /// `strcmp(lhs, rhs) != 0`.  Table-driven menu scripts branch on this while
    /// scanning parsed CSV/file rows, so returning a constant corrupts control
    /// flow and can make title/system pages fall through.
    fn ext_string_not_equal(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() != 2 {
            return ExtCallOutcome::Block;
        }
        let lhs = self
            .resolve_script_string(args[0], assets, nls)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let rhs = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let differs = lhs != rhs;
        log::debug!(
            "[trace-script] ext_0012_0003.string_not_equal lhs={lhs:?} rhs={rhs:?} -> {differs}"
        );
        ExtCallOutcome::Value(i32::from(differs))
    }

    /// Game category 18 index 4 (`sub_41A850`) appends a resolved string to a
    /// dynamic string buffer.  Native only accepts dynamic-string destinations;
    /// the portable VM preserves that rule and returns success.
    fn ext_string_append(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(2);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let dst = args[0];
        let suffix = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default();
        if is_dynamic_string_handle(dst) {
            let mut value = self
                .resolve_script_string(dst, assets, nls)
                .unwrap_or_default();
            value.push_str(&suffix);
            self.replace_dynamic_string(dst, value);
        }
        ExtCallOutcome::Value(1)
    }

    /// Newer VM 18:119 (0x4354C0): pop destination, source, byte offset,
    /// byte count. Count -1 copies the suffix; no VM destination is written.
    fn ext_string_substr_bytes(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() != 4 {
            return ExtCallOutcome::Block;
        }
        let (dst, source, start, count) = (args[0], args[1], args[2], args[3]);
        if !is_dynamic_string_handle(dst) || start < 0 || count < -1 {
            return ExtCallOutcome::Continue;
        }
        let source = self.resolve_script_string(source, assets, nls).unwrap_or_default();
        let Ok(bytes) = nls.encode(&source) else { return ExtCallOutcome::Continue; };
        let Some(suffix) = bytes.get(start as usize..) else { return ExtCallOutcome::Continue; };
        let length = if count == -1 { suffix.len() } else { count as usize };
        // Native buffers have a 2047-byte stride. Reject invalid encoding
        // boundaries and bound the copy instead of reproducing memory damage.
        let length = length.min(suffix.len()).min(2046);
        if let Ok(value) = nls.decode(&suffix[..length]) {
            self.replace_dynamic_string(dst, value);
        }
        ExtCallOutcome::Continue
    }

    /// Game category 18 index 7 (`sub_41A4B0`) copies a resolved string into a
    /// dynamic string buffer, truncated to len unless len is -1.
    fn ext_string_copy_len(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let dst = args[0];
        let mut value = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default();
        let len = args[2];
        if len >= 0 {
            value.truncate(len as usize);
        }
        if is_dynamic_string_handle(dst) {
            self.replace_dynamic_string(dst, value);
        }
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 5 (`sub_41A6B0`) is the native `strgetcf`
    /// helper: pop string, offset, length; when length is zero return the byte
    /// at offset, otherwise parse the digit-only substring as an integer.
    fn ext_str_get_char_or_int(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() != 3 {
            return ExtCallOutcome::Block;
        }
        let text = self
            .resolve_script_string(args[0], assets, nls)
            .or_else(|| self.resolve_resource_string(args[0], assets, nls))
            .unwrap_or_default();
        let offset = args[1].max(0) as usize;
        let len = args[2].max(0) as usize;
        let bytes = text.as_bytes();
        let value = if offset >= bytes.len() {
            0
        } else if len == 0 {
            i32::from(bytes[offset])
        } else {
            let end = offset.saturating_add(len).min(bytes.len());
            let slice = &bytes[offset..end];
            if slice.iter().all(u8::is_ascii_digit) {
                std::str::from_utf8(slice)
                    .ok()
                    .and_then(|slice| slice.parse::<i32>().ok())
                    .unwrap_or(0)
            } else {
                0
            }
        };
        log::debug!(
            "[trace-script] ext_0012_0005.strgetcf text={text:?} offset={offset} len={len} -> {value}"
        );
        ExtCallOutcome::Value(value)
    }

    /// Game category 18 index 8 (`sub_41A260`) resolves a PAL path and tests
    /// whether PalFileCreate can open it.  Check loose root paths and PAC
    /// resources for a cross-platform equivalent.
    fn ext_file_exist(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.len() != 1 {
            return ExtCallOutcome::Block;
        }
        let name = self
            .resolve_script_string(args[0], assets, nls)
            .or_else(|| self.resolve_resource_string(args[0], assets, nls))
            .unwrap_or_default();
        let exists = if name.is_empty() {
            false
        } else if let Some(manager) = resource_manager {
            let loose = manager.root().join(&name);
            loose.exists() || manager.open(&name).is_ok()
        } else {
            std::path::Path::new(&name).exists()
        };
        log::debug!("[trace-assets] file_exist name={name:?} -> {exists}");
        ExtCallOutcome::Value(i32::from(exists))
    }

    /// Game category 18 index 10 (`sub_419370`) checks a CD-ROM volume label,
    /// but native returns success in debug mode.  A physical-drive scan is not
    /// portable, so the VM uses the same permissive compatibility path.
    fn ext_check_disc(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.len() != 1 {
            return ExtCallOutcome::Block;
        }
        let label = self
            .resolve_script_string(args[0], assets, nls)
            .unwrap_or_else(|| args[0].to_string());
        log::debug!("[trace-system] check_disc label={label:?} -> true portable_debug_path");
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 18 (`sub_41A330`) replaces the first occurrence
    /// of one resolved string with another inside a dynamic string buffer.
    fn ext_string_replace(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() < 3 {
            return ExtCallOutcome::Block;
        }
        let dst = args[0];
        if !is_dynamic_string_handle(dst) {
            return ExtCallOutcome::Value(1);
        }
        let needle = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default();
        let replacement = self
            .resolve_script_string(args[2], assets, nls)
            .unwrap_or_default();
        let mut value = self
            .resolve_script_string(dst, assets, nls)
            .unwrap_or_default();
        if !needle.is_empty() {
            value = value.replacen(&needle, &replacement, 1);
            self.replace_dynamic_string(dst, value);
        }
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 40 (`sub_417E50`) clears an access/read flag by
    /// id or resource name.  The portable save/access flag table is not fully
    /// materialized yet, but the exact one-argument stack effect is required.
    fn ext_access_clear(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if let Some(entry) = args.first() {
            self.access_updates.remove(&entry.to_string());
        }
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 14 (`sub_417C20`) marks an access flag by string
    /// resource or numeric id.  Native mutates packed bits in PAL task data; the
    /// portable VM stores the resolved key so later save/history code can query
    /// the same logical state.
    fn ext_update_access(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.len() != 1 {
            return ExtCallOutcome::Block;
        }
        let key = self
            .resolve_script_string(args[0], assets, nls)
            .or_else(|| self.resolve_resource_string(args[0], assets, nls))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| args[0].to_string());
        self.access_updates.insert(key.clone(), 1);
        log::debug!("[trace-system] update_access key={key:?}");
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 15 (`sub_417BD0`) returns a PAL task-data value
    /// when the native system latch is active.  Existing title scripts expect a
    /// non-zero compatibility value during startup, matching the previous
    /// runtime behavior while removing the generic arg_probe placeholder.
    fn ext_system_task_value(&mut self) -> ExtCallOutcome {
        self.pop_ext_args(0);
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 17 (`sub_417AF0`) calls the native work-process
    /// value helper and sets the script dirty flag.  The exact PAL helper still
    /// needs deeper struct modeling; returning the supplied value preserves the
    /// wrapper's comparison semantics instead of a silent fallback.
    fn ext_work_process_value(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        if args.len() != 1 {
            return ExtCallOutcome::Block;
        }
        self.work_process_attached = true;
        let value = args[0];
        log::debug!("[trace-system] work_process_value arg={value} -> {value}");
        ExtCallOutcome::Value(value)
    }

    /// Game category 18 index 28 (`sub_417A50`) starts the native background
    /// work-process pump when no skip/system mode is active.  The handler pops
    /// no VM arguments, sets VM flags at +804088/+804084, and calls
    /// `PalAttachWorkProcess(sub_44A080, PalTaskGetTaskData(0)+824)`.
    ///
    /// PAL evidence: `PalAttachWorkProcess` posts a work message to the PAL
    /// thread (`PAL.sqlite` 0x1011CCA9 -> 0x10237F80).  The portable VM is
    /// single-threaded, so preserving the attach flag and dirty-script signal is
    /// the safe cross-platform equivalent.
    fn ext_attach_work_process(&mut self) -> ExtCallOutcome {
        self.pop_ext_args(0);
        self.work_process_attached = true;
        log::debug!("[trace-system] attach_work_process active=true");
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 29 (`sub_417A30`) clears the native work-process
    /// flag at VM offset +804088 and pops no VM arguments.
    fn ext_detach_work_process(&mut self) -> ExtCallOutcome {
        self.pop_ext_args(0);
        self.work_process_attached = false;
        log::debug!("[trace-system] attach_work_process active=false");
        ExtCallOutcome::Value(1)
    }

    fn ext_string_alloc(&mut self) -> ExtCallOutcome {
        // Game.exe 0x0041AAF0 (`VmExtcall_ArgGet` in the current IDB labels)
        // pops one unused stack value, takes ctx[192823] as a 16-slot rotating
        // dynamic-string index, clears the 0x7FF-byte native buffer, advances
        // the cursor, and writes `slot | 0x10000000` to the extcall
        // destination.  The portable runtime mirrors the observable handle and
        // empty-string side effect; later strcpy/strcat/file_string calls fill
        // the selected slot.
        let _ = self.pop_ext_args(1);
        let slot = self.dynamic_string_cursor % 16;
        self.dynamic_string_cursor = (slot + 1) % 16;
        if self.dynamic_strings.len() <= slot {
            self.dynamic_strings.resize(slot + 1, String::new());
        }
        self.dynamic_strings[slot].clear();
        let handle = 0x1000_0000u32.wrapping_add(slot as u32) as i32;
        log::debug!("[trace-script] ext_0012_0006.string_alloc slot={slot} -> {handle:#010X}");
        ExtCallOutcome::Value(handle)
    }

    fn ext_wsprint_compat(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        // Game.exe sub_419560 is a wsprintf-like dynamic string formatter.  It
        // pops destination, format string, and eight formatting arguments, then
        // writes the formatted byte string into the destination dynamic-string
        // buffer.  Leaving this as a zero-pop helper corrupts the VM stack and
        // makes the following strlenf/string-compare loops see empty strings.
        let args = self.pop_ext_args(10);
        if args.len() < 2 {
            return ExtCallOutcome::Block;
        }
        let dst = args[0];
        let format = self
            .resolve_script_string(args[1], assets, nls)
            .or_else(|| self.resolve_resource_string(args[1], assets, nls))
            .unwrap_or_default();
        let mut formatted = String::new();
        let mut chars = format.chars().peekable();
        let mut value_index = 2usize;
        while let Some(ch) = chars.next() {
            if ch != '%' {
                formatted.push(ch);
                continue;
            }
            // wsprintf-style conversion: %, optional zero-pad flag and width,
            // then the specifier (`sound_unit%02d`, `save_page%02d`, ...).
            let mut zero_pad = false;
            let mut width = 0usize;
            let mut spec = None;
            for next in chars.by_ref() {
                match next {
                    '0' if width == 0 && !zero_pad => zero_pad = true,
                    '1'..='9' => {
                        width = width
                            .saturating_mul(10)
                            .saturating_add(next.to_digit(10).unwrap_or(0) as usize);
                    }
                    _ => {
                        spec = Some(next);
                        break;
                    }
                }
            }
            let Some(spec) = spec else {
                formatted.push('%');
                break;
            };
            if spec == '%' {
                formatted.push('%');
                continue;
            }
            let value = args.get(value_index).copied().unwrap_or_default();
            value_index = value_index.saturating_add(1);
            // Numeric conversions accept dynamic-string arguments the way the
            // native formatter does: the buffer content is parsed (`Sound.csv`
            // hands the unit index over as a one-character string).
            let numeric = if is_dynamic_string_handle(value) {
                dynamic_string_index(value)
                    .and_then(|index| self.dynamic_strings.get(index))
                    .and_then(|text| text.trim().parse::<i32>().ok())
                    .unwrap_or(value)
            } else {
                value
            };
            match spec {
                's' | 'S' | 'f' | 'F' => {
                    let text = self
                        .resolve_script_string(value, assets, nls)
                        .or_else(|| self.resolve_resource_string(value, assets, nls))
                        .unwrap_or_else(|| value.to_string());
                    formatted.push_str(&text);
                }
                'd' | 'D' | 'i' | 'I' => {
                    if zero_pad {
                        formatted.push_str(&format!("{numeric:0width$}", width = width));
                    } else if width > 0 {
                        formatted.push_str(&format!("{numeric:width$}", width = width));
                    } else {
                        formatted.push_str(&numeric.to_string());
                    }
                }
                'x' => formatted.push_str(&format!("{numeric:x}")),
                'X' => formatted.push_str(&format!("{numeric:X}")),
                other => {
                    formatted.push('%');
                    if zero_pad {
                        formatted.push('0');
                    }
                    if width > 0 {
                        formatted.push_str(&width.to_string());
                    }
                    formatted.push(other);
                }
            }
        }
        if is_dynamic_string_handle(dst) {
            self.replace_dynamic_string(dst, formatted.clone());
        }
        log::debug!(
            "[trace-script] ext_0012_0009.wsprint dst={dst:#010X} format={format:?} -> {formatted:?}"
        );
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 12 returns the length of a dynamic string. Some
    /// Koikake callsites carry seven `0x0FFF_FFFF` filler operands below the
    /// string handle; leave the caller's argument-frame marker intact.
    fn ext_string_length(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let padded = self.stack.len() >= 8
            && self.stack[self.stack.len() - 8..self.stack.len() - 1]
                .iter()
                .all(|&value| value == 0x0FFF_FFFF);
        let args = self.pop_ext_args(if padded { 8 } else { 1 });
        let Some(value) = args.first().copied() else {
            return ExtCallOutcome::Value(0);
        };
        let len = self
            .resolve_script_string(value, assets, nls)
            .map(|text| text.len() as i32)
            .unwrap_or(0);
        log::debug!("[trace-script] ext_0012_000C.strlen value={value} -> {len}");
        ExtCallOutcome::Value(len)
    }

    /// Game category 18 index 13 (`sub_41B8D0`) stores a process checkpoint id
    /// and pushes the current script point into a native PalList with tag 3.
    /// The portable VM keeps script scheduling in Rust structures, so this is a
    /// visible bookkeeping/status operation rather than a separate native list.
    fn ext_process_checkpoint_set(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let checkpoint_id = args.first().copied().unwrap_or(0);
        log::debug!(
            "[trace-script] ext_0012_000D.process_checkpoint_set checkpoint_id={checkpoint_id}"
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_strlenf(&mut self, assets: &CoreAssets, nls: Nls) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(value) = args.first().copied() else {
            return ExtCallOutcome::Value(0);
        };
        let len = self
            .resolve_script_string(value, assets, nls)
            .map(|s| {
                nls.encode(&s)
                    .map(|bytes| bytes.len() as i32)
                    .unwrap_or_else(|_| s.len() as i32)
            })
            .unwrap_or(0);
        log::debug!("[trace-script] ext_0012_0015.strlenf value={value} -> {len}");
        ExtCallOutcome::Value(len)
    }

    fn ext_get_private_profile_int(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() != 4 {
            return ExtCallOutcome::Block;
        }
        let section = self
            .resolve_script_string(args[0], assets, nls)
            .unwrap_or_default();
        let key = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default();
        let default = args[2];
        let requested_filename = self
            .resolve_script_string(args[3], assets, nls)
            .unwrap_or_else(|| "SYSTEM.INI".to_owned());
        let filename = ini_filename_or_system(&requested_filename);

        let value = match self.system_ini(resource_manager) {
            Some(ini) => ini
                .get(&section.to_ascii_lowercase())
                .and_then(|section| section.get(&key.to_ascii_lowercase()))
                .and_then(|value| value.as_int())
                .map(|value| value as i32)
                .unwrap_or(default),
            None => default,
        };

        log::debug!(
            "[trace-script] ext_0012_0025.getprivateprofileint section={section:?} key={key:?} default={default} requested_filename={requested_filename:?} used_filename={filename:?} -> {value}"
        );
        ExtCallOutcome::Value(value)
    }

    fn ext_sz_buf(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(5);
        if args.len() != 5 {
            return ExtCallOutcome::Block;
        }
        let section = self
            .resolve_script_string(args[0], assets, nls)
            .unwrap_or_default();
        let key = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default();
        let default = self
            .resolve_script_string(args[2], assets, nls)
            .unwrap_or_default();
        let requested_filename = self
            .resolve_script_string(args[4], assets, nls)
            .unwrap_or_else(|| "SYSTEM.INI".to_owned());
        let filename = ini_filename_or_system(&requested_filename);
        let value = match self.system_ini(resource_manager) {
            Some(ini) => ini
                .get(&section.to_ascii_lowercase())
                .and_then(|section| section.get(&key.to_ascii_lowercase()))
                .and_then(|value| value.as_str().map(str::to_owned))
                .unwrap_or(default),
            None => default,
        };
        // args[3] is a caller-supplied destination dynamic-string buffer; the
        // SOUND menu passes one and then hands the same handle to open_file.
        // Native does not echo the handle into the task work bank — writing it
        // to temp_mem[base+128] clobbers the SOUND menu's unit loop counter.
        let dst = args[3];
        let handle = if is_dynamic_string_handle(dst) {
            self.replace_dynamic_string(dst, value.clone());
            dst
        } else {
            self.store_dynamic_string(value.clone())
        };
        log::debug!(
            "[trace-script] ext_0012_0024.sz_buf section={section:?} key={key:?} requested_filename={requested_filename:?} used_filename={filename:?} -> {value:?}"
        );
        ExtCallOutcome::Value(handle)
    }

    /// Game.exe `sub_416670` wraps `WritePrivateProfileStringA` for integer
    /// values.  The native pop order is `(section,key,value,filename)` from
    /// bottom to top after stack reversal; it returns the Win32 success value.
    /// Keep the write in the parsed SYSTEM.INI shadow so settings scripts can
    /// read their own updates without mutating the user's testcase directory.
    fn ext_write_private_profile_int(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() != 4 {
            return ExtCallOutcome::Block;
        }
        let section = self
            .resolve_script_string(args[0], assets, nls)
            .unwrap_or_default();
        let key = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default();
        let value = args[2];
        let requested_filename = self
            .resolve_script_string(args[3], assets, nls)
            .unwrap_or_else(|| "SYSTEM.INI".to_owned());
        let filename = ini_filename_or_system(&requested_filename).to_owned();
        self.write_system_ini_shadow(
            resource_manager,
            &section,
            &key,
            IniValue::Int(i64::from(value)),
        );
        log::debug!(
            "[trace-script] ext_0012_0026.writeprivateprofileint section={section:?} key={key:?} value={value} requested_filename={requested_filename:?} used_filename={filename:?}"
        );
        ExtCallOutcome::Value(1)
    }

    /// Game.exe `sub_416820` writes a quoted string through
    /// `WritePrivateProfileStringA`.  The portable runtime stores the decoded
    /// string value directly in the SYSTEM.INI shadow; `parse_ini_text` would
    /// strip the quotes on a later load, so keeping the unquoted payload matches
    /// subsequent `GetPrivateProfileString` behavior.
    fn ext_write_private_profile_string(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(4);
        if args.len() != 4 {
            return ExtCallOutcome::Block;
        }
        let section = self
            .resolve_script_string(args[0], assets, nls)
            .unwrap_or_default();
        let key = self
            .resolve_script_string(args[1], assets, nls)
            .unwrap_or_default();
        let value = self
            .resolve_script_string(args[2], assets, nls)
            .unwrap_or_default();
        let requested_filename = self
            .resolve_script_string(args[3], assets, nls)
            .unwrap_or_else(|| "SYSTEM.INI".to_owned());
        let filename = ini_filename_or_system(&requested_filename).to_owned();
        self.write_system_ini_shadow(
            resource_manager,
            &section,
            &key,
            IniValue::Str(value.clone()),
        );
        log::debug!(
            "[trace-script] ext_0012_0027.writeprivateprofilestring section={section:?} key={key:?} value={value:?} requested_filename={requested_filename:?} used_filename={filename:?}"
        );
        ExtCallOutcome::Value(1)
    }

    fn ext_open_file(
        &mut self,
        assets: &CoreAssets,
        nls: Nls,
        resource_manager: Option<&mut ResourceManager>,
    ) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let Some(name) = args.first().and_then(|arg| {
            self.resolve_resource_string(*arg, assets, nls)
                .or_else(|| self.resolve_script_string(*arg, assets, nls))
        }) else {
            return ExtCallOutcome::Value(0);
        };
        let Some(resource_manager) = resource_manager else {
            return ExtCallOutcome::Value(0);
        };
        match resource_manager.open(&name) {
            Ok(asset) => {
                let handle = self.insert_file_handle(RuntimeFile {
                    name: name.clone(),
                    parsed_table: parse_file_table(&asset.bytes, resource_manager.nls()).ok(),
                    bytes: asset.bytes,
                    cursor: 0,
                    table_cursor: 0,
                });
                log::debug!(
                    "[trace-assets] openfile name={name:?} source={} handle={handle}",
                    asset_source_label(&asset.source)
                );
                ExtCallOutcome::Value(handle)
            }
            Err(err) => {
                if is_known_non_asset_probe_name(&name) {
                    log::debug!("[trace-assets] openfile probe name={name:?} ignored: {err}");
                } else {
                    log::warn!("[trace-assets] openfile name={name:?} failed: {err}");
                }
                ExtCallOutcome::Value(0)
            }
        }
    }

    fn ext_close_file_not_handle(&mut self) -> ExtCallOutcome {
        // Game category 18 index 32 (`sub_4170C0`) pops the open_file handle,
        // removes it from the native PalList, frees the parsed table buffer,
        // and returns 1.  Treating this as zero-argument leaves the handle on
        // the VM stack and corrupts later menu/save table scans.
        let args = self.pop_ext_args(1);
        let handle = args.first().copied().unwrap_or(0);
        if handle > 0 {
            let slot = (handle - 1) as usize;
            if let Some(file_slot) = self.file_handles.get_mut(slot) {
                *file_slot = None;
            }
        }
        log::debug!("[trace-assets] close_file_not_handle handle={handle}");
        ExtCallOutcome::Value(1)
    }

    fn ext_read_file(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() != 3 {
            return ExtCallOutcome::Block;
        }
        let handle = args[0];
        let temp_offset = args[1];
        let count = args[2].max(0) as usize;
        let (name, entries, read_len) = {
            let Some(file) = self.file_handle_mut(handle) else {
                return ExtCallOutcome::Value(0);
            };
            if let Some(table) = file.parsed_table.as_ref() {
                let entry_start = file.table_cursor / 4;
                let available = table.entries.len().saturating_sub(entry_start);
                let read_len = available.min(count);
                let entries = table.entries[entry_start..entry_start + read_len].to_vec();
                file.table_cursor += read_len * 4;
                (file.name.clone(), entries, read_len)
            } else {
                let available = file.bytes.len().saturating_sub(file.cursor);
                let read_len = available.min(count);
                let start = file.cursor;
                let end = start + read_len;
                let entries = file.bytes[start..end]
                    .iter()
                    .map(|byte| i32::from(*byte))
                    .collect::<Vec<_>>();
                file.cursor = end;
                (file.name.clone(), entries, read_len)
            }
        };
        for (i, value) in entries.iter().enumerate() {
            self.write_temp_mem_absolute(temp_offset + i as i32, *value);
        }
        log::debug!(
            "[trace-assets] read_file handle={handle} name={:?} temp_offset={temp_offset} count={count} values={entries:?} -> {read_len}",
            name
        );
        ExtCallOutcome::Value(if read_len == count { 1 } else { 0 })
    }

    fn ext_set_file_pointer(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() != 3 {
            return ExtCallOutcome::Block;
        }
        let handle = args[0];
        let offset = args[1];
        let origin = args[2];
        let Some(file) = self.file_handle_mut(handle) else {
            return ExtCallOutcome::Value(0);
        };
        let base = match origin {
            0 => 0i64,
            1 => file.table_cursor as i64,
            2 => file
                .parsed_table
                .as_ref()
                .map_or(file.bytes.len(), |table| table.entries.len() * 4) as i64,
            _ => file.table_cursor as i64,
        };
        let next = base.saturating_add((offset as i64) * 4).max(0) as usize;
        let table_len = file
            .parsed_table
            .as_ref()
            .map_or(file.bytes.len(), |table| table.entries.len() * 4);
        file.table_cursor = next.min(table_len);
        file.cursor = file.table_cursor.min(file.bytes.len());
        log::debug!(
            "[trace-assets] set_file_pointer handle={handle} name={:?} offset={offset} origin={origin} table_cursor={}",
            file.name,
            file.table_cursor
        );
        ExtCallOutcome::Value(1)
    }

    /// Game category 18 index 34 (`sub_416EC0`) pops handle, encoded string
    /// entry, then destination dynamic-string slot.  The second popped value is
    /// masked with `0x7fffffff` and used as a string-table offset; the third
    /// names the destination dynamic string buffer. Keeping this order matters
    /// because the table-name helper runs immediately before sprite transition
    /// calls and a swapped entry/dst
    /// leaves resource names unresolved.
    fn ext_file_string(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(3);
        if args.len() != 3 {
            return ExtCallOutcome::Block;
        }
        let handle = args[0];
        let entry = args[1];
        let dst_slot = args[2];
        let Some(file) = self.file_handle_mut(handle) else {
            return ExtCallOutcome::Value(0);
        };
        let Some(table) = file.parsed_table.as_ref() else {
            return ExtCallOutcome::Value(0);
        };
        let offset = entry & 0x7FFF_FFFF;
        let value = table.strings.get(&offset).cloned().unwrap_or_default();
        let dyn_value = if is_dynamic_string_handle(dst_slot) {
            self.replace_dynamic_string(dst_slot, value.clone());
            dst_slot
        } else {
            self.store_dynamic_string(value.clone())
        };
        log::debug!(
            "[trace-script] ext_0012_0022.file_string handle={handle} entry=0x{entry:08X} offset={offset} -> {value:?}"
        );
        ExtCallOutcome::Value(dyn_value)
    }

    /// Game category 18 index 35 (`sub_4179B0`) pops one script point id,
    /// resolves it through the native point table when non-zero, and stores the
    /// result in the VM last-process field.  The portable runtime does not use
    /// the cached native address, but it must still pop exactly one argument.
    fn ext_set_last_process(&mut self) -> ExtCallOutcome {
        let args = self.pop_ext_args(1);
        let point_id = args.first().copied().unwrap_or(0);
        log::debug!("[trace-script] ext_0012_0023.set_last_process point_id={point_id}");
        ExtCallOutcome::Value(1)
    }

    fn pop_ext_args(&mut self, count: usize) -> Vec<i32> {
        let available = count.min(self.stack.len());
        let mut args = Vec::with_capacity(available);
        for _ in 0..available {
            if let Some(value) = self.stack.pop() {
                args.push(value);
            }
        }
        self.vm_trace(format_args!(
            "  ext_args requested={count} available={available} popped={args:?} stack_len={}",
            self.stack.len()
        ));
        args
    }

    fn resolve_script_string(&self, value: i32, assets: &CoreAssets, nls: Nls) -> Option<String> {
        if value == 0x0FFF_FFFF {
            return Some(String::new());
        }
        if let Some(index) = dynamic_string_index(value) {
            return self.dynamic_strings.get(index).cloned();
        }
        let offset = value.try_into().ok()?;
        read_text_record_string(
            &assets.text_dat.bytes,
            offset,
            nls,
            assets.text_dat_multilingual,
        )
        .map(|text| select_text_field(text, self.text_lang, assets.text_dat_multilingual))
        .or_else(|| read_c_string(&assets.text_dat.bytes, offset, nls))
        .or_else(|| read_c_string(&assets.file_dat.bytes, offset, nls))
    }

    fn resolve_resource_string(&self, value: i32, assets: &CoreAssets, nls: Nls) -> Option<String> {
        if value == 0x0FFF_FFFF {
            return Some(String::new());
        }
        if let Some(index) = dynamic_string_index(value) {
            return self.dynamic_strings.get(index).cloned();
        }
        let offset = value.try_into().ok()?;
        read_file_slot_string(&assets.file_dat.bytes, value, nls, assets.file_dat_layout)
            .or_else(|| read_padded_c_string(&assets.file_dat.bytes, offset, nls))
            .or_else(|| {
                read_text_record_string(&assets.file_dat.bytes, offset, nls, false)
                    .filter(|value| is_plausible_resource_name(value))
            })
            .or_else(|| {
                read_text_record_string(
                    &assets.text_dat.bytes,
                    offset,
                    nls,
                    assets.text_dat_multilingual,
                )
                .map(|text| select_text_field(text, self.text_lang, assets.text_dat_multilingual))
                .filter(|value| is_plausible_resource_name(value))
            })
            .or_else(|| {
                read_c_string(&assets.text_dat.bytes, offset, nls)
                    .filter(|value| is_plausible_resource_name(value))
            })
    }

    fn system_ini(&mut self, resource_manager: Option<&mut ResourceManager>) -> Option<&IniFile> {
        if self.system_ini.is_none() {
            let Some(resource_manager) = resource_manager else {
                return None;
            };
            match resource_manager.open("SYSTEM.INI") {
                Ok(asset) => {
                    log::debug!(
                        "[trace-assets] ini open name=\"SYSTEM.INI\" source={}",
                        asset_source_label(&asset.source)
                    );
                    match parse_system_ini(&asset.bytes, resource_manager.nls()) {
                        Ok(ini) => self.set_system_ini(ini),
                        Err(err) => {
                            log::warn!("SYSTEM.INI parse failed: {err}");
                            return None;
                        }
                    }
                }
                Err(err) => {
                    log::warn!("[trace-assets] SYSTEM.INI open failed: {err}");
                    return None;
                }
            }
        }
        self.system_ini.as_ref()
    }

    fn write_system_ini_shadow(
        &mut self,
        resource_manager: Option<&mut ResourceManager>,
        section: &str,
        key: &str,
        value: IniValue,
    ) {
        let _ = self.system_ini(resource_manager);
        let section_key = section.to_ascii_lowercase();
        let key_key = key.to_ascii_lowercase();
        self.system_ini
            .get_or_insert_with(Default::default)
            .entry(section_key)
            .or_default()
            .insert(key_key, value);
    }

    fn store_dynamic_string(&mut self, value: String) -> i32 {
        let index = self.dynamic_strings.len();
        self.dynamic_strings.push(value);
        0x1000_0000u32.wrapping_add(index as u32) as i32
    }

    fn replace_dynamic_string(&mut self, handle: i32, value: String) -> bool {
        let Some(index) = dynamic_string_index(handle) else {
            return false;
        };
        if index >= self.dynamic_strings.len() {
            self.dynamic_strings.resize(index + 1, String::new());
        }
        self.dynamic_strings[index] = value;
        true
    }

    fn write_temp_mem_relative(&mut self, offset: i32, value: i32) {
        let index = self.argument_base.wrapping_add(offset);
        if index < 0 {
            return;
        }
        if let Some(slot) = self.temp_mem.get_mut(index as usize) {
            *slot = value;
        }
    }

    fn write_temp_mem_absolute(&mut self, offset: i32, value: i32) {
        if offset < 0 {
            return;
        }
        let index = offset as usize;
        if index >= self.temp_mem.len() {
            self.temp_mem.resize(index + 1, 0);
        }
        self.temp_mem[index] = value;
    }

    fn insert_file_handle(&mut self, file: RuntimeFile) -> i32 {
        for (index, slot) in self.file_handles.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(file);
                return (index + 1) as i32;
            }
        }
        self.file_handles.push(Some(file));
        self.file_handles.len() as i32
    }

    fn file_handle_mut(&mut self, handle: i32) -> Option<&mut RuntimeFile> {
        if handle <= 0 {
            return None;
        }
        self.file_handles
            .get_mut(handle as usize - 1)
            .and_then(Option::as_mut)
    }

    fn encoded_button_key(&self, layer: u8, index: i32) -> Option<(i32, i32)> {
        // Tagged PAL sprite ids (0x010000NN / 0x020000NN) address Game.exe's
        // native UI sprite layers.  In this title they are used for the main
        // and title button tables: layer 1 covers the compact button strip,
        // while layer 2 reaches the voice/index-14 control.  Prefer ADV group 0
        // when it exists, then fall back to title group 1.
        match layer {
            1 | 2 => [(0, index), (1, index)]
                .into_iter()
                .find(|key| self.game_buttons.contains_key(key)),
            _ => None,
        }
    }

    fn apply_encoded_button_alpha_to(
        &mut self,
        sprites: &mut SpriteSystem,
        layer: u8,
        index: i32,
        alpha: u8,
        duration_ms: u32,
    ) -> bool {
        let Some(key) = self.encoded_button_key(layer, index) else {
            return false;
        };
        let Some(entry) = self.game_buttons.get_mut(&key) else {
            return false;
        };
        // Native encoded UI sprite ids (0x010000NN/0x020000NN) eventually
        // call PalSpriteSetColor on the backing button sprite.  Keep the PAL
        // button-table alpha cache in lockstep with the sprite target alpha;
        // otherwise visible fade-in buttons keep entry.alpha == 0 and the
        // portable hit-test rejects them as inactive.
        entry.alpha = alpha;
        let handle = entry.handle;
        if duration_ms > 0 {
            sprites.tween_alpha_to(handle, alpha, duration_ms)
        } else {
            sprites.set_alpha(handle, alpha)
        }
    }

    fn apply_encoded_button_alpha_delta(
        &mut self,
        sprites: &mut SpriteSystem,
        layer: u8,
        index: i32,
        alpha_delta: i32,
        duration_ms: u32,
    ) -> bool {
        let Some(key) = self.encoded_button_key(layer, index) else {
            return false;
        };
        let Some(entry) = self.game_buttons.get(&key) else {
            return false;
        };
        let current = sprites
            .get(entry.handle)
            .map_or(entry.alpha, |sprite| sprite.color.alpha()) as i32;
        let target = (current + alpha_delta).clamp(0, 255) as u8;
        self.apply_encoded_button_alpha_to(sprites, layer, index, target, duration_ms)
    }

    fn matching_button_handles(&self, group: i32, index: i32) -> Vec<SpriteHandle> {
        self.game_buttons
            .iter()
            .filter(|((button_group, button_index), _)| {
                (group < 0 || *button_group == group) && (index < 0 || *button_index == index)
            })
            .map(|(_, entry)| entry.handle)
            .collect()
    }

    fn matching_button_entries_mut(&mut self, group: i32, index: i32) -> Vec<&mut GameButtonEntry> {
        self.game_buttons
            .iter_mut()
            .filter(|((button_group, button_index), _)| {
                (group < 0 || *button_group == group) && (index < 0 || *button_index == index)
            })
            .map(|(_, entry)| entry)
            .collect()
    }

    /// Locate the slider track anchor for a knob button.  Script families lay
    /// sliders out as adjacent track/knob pairs, but the index offset varies
    /// (SOUND page uses knob = base + 10 on the main column and knob = base +
    /// 20 on the per-character column; the ADV bar uses knob = base + 1).
    /// Neighbouring knobs sit between the pair, so plain proximity picks the
    /// wrong button.  Instead use the knob's stored drag offset: the anchor is
    /// the neighbour sitting closest to where the knob would rest at offset 0.
    fn slider_anchor_position(
        &self,
        sprites: &SpriteSystem,
        group: i32,
        index: i32,
        axis: i32,
    ) -> Option<(i32, i32)> {
        let knob = self.game_buttons.get(&(group, index))?;
        let knob_pos = sprites.get(knob.handle)?.effective_position();
        let expected = if axis == 1 {
            (knob_pos.x, knob_pos.y - knob.slider_offset)
        } else {
            (knob_pos.x - knob.slider_offset, knob_pos.y)
        };
        let mut best: Option<(i64, (i32, i32))> = None;
        for delta in [20, 10, 2, 1] {
            if index < delta {
                continue;
            }
            let Some(entry) = self.game_buttons.get(&(group, index - delta)) else {
                continue;
            };
            let Some(sprite) = sprites.get(entry.handle) else {
                continue;
            };
            let pos = sprite.effective_position();
            let distance =
                (pos.x - expected.0).abs() as i64 + (pos.y - expected.1).abs() as i64;
            if best.map_or(true, |(best_distance, _)| distance < best_distance) {
                best = Some((distance, (pos.x, pos.y)));
            }
        }
        best.map(|(_, pos)| pos)
    }

    fn dispatch_button_push_compat(&mut self, group: i32, index: i32) {
        // Native `btn_set` stores a callback point in arg[3] for any button
        // group.  Title, save/load, and system tabs all depend on the PAL button
        // system injecting a gosub when that entry is clicked.
        let Some(point_id) = self
            .game_buttons
            .get(&(group, index))
            .and_then(|e| e.gosub_point)
        else {
            return;
        };
        self.pending_gosub_points.push_back(point_id);
        log::debug!(
            "[trace-button] queued gosub for group={group} index={index} -> point[{point_id}]"
        );
    }

    fn button_hit_at(
        &self,
        sprites: &SpriteSystem,
        mouse_x: i32,
        mouse_y: i32,
        group: i32,
    ) -> Option<(i32, i32)> {
        self.button_hit_at_with_lock(sprites, mouse_x, mouse_y, group, true)
    }

    fn button_hit_at_with_lock(
        &self, sprites: &SpriteSystem, mouse_x: i32, mouse_y: i32,
        group: i32, respect_lock: bool,
    ) -> Option<(i32, i32)> {
        if mouse_x < 0 || mouse_y < 0 { return None; }
        let mut best: Option<(i32, i32, i32, u64)> = None;
        for ((button_group, button_index), entry) in &self.game_buttons {
            if group >= 0 && *button_group != group {
                continue;
            }
            if !entry.visible || (respect_lock && entry.locked) {
                continue;
            }
            let Some(sprite) = sprites.get(entry.handle) else {
                continue;
            };
            if !sprite.visible {
                continue;
            }
            let hit = if let Some(rect) = entry.hit_rect {
                mouse_x >= rect[0] && mouse_x < rect[2]
                    && mouse_y >= rect[1] && mouse_y < rect[3]
            } else {
                sprites.hit(entry.handle, mouse_x, mouse_y,
                    self.effective_button_flags(entry.native_flags) & 0x10000) != 0
            };
            if !hit { continue; }
            // Game adapters invert native depth. PAL compares stored priority,
            // not the Z/action-adjusted priority used by the render list.
            let priority = sprite.base_priority;
            if best.is_none_or(|(_, _, best_priority, order)| priority > best_priority || (priority == best_priority && entry.registration_order >= order)) {
                best = Some((*button_group, *button_index, priority, entry.registration_order));
            }
        }
        best.and_then(|(group, index, _, _)| {
            self.game_buttons.get(&(group, index)).filter(|entry| entry.enabled).map(|_| (group, index))
        })
    }

    pub fn diagnostic_button_hit_enabled(
        &self,
        sprites: &SpriteSystem,
        mouse_x: i32,
        mouse_y: i32,
    ) -> Option<(i32, i32)> {
        self.button_hit_at(sprites, mouse_x, mouse_y, -1)
    }

    pub fn dump_text_diagnostic_state(
        &self,
        sprites: &SpriteSystem,
        scene: &FrameScene,
        status: &RuntimeStatus,
        input: &PalInputState,
        frame: u64,
    ) {
        let sprite_info = self.text_state.sprite.and_then(|handle| {
            sprites.get(handle).map(|sprite| {
                let texture_id = SceneTextureId(sprite.surface.0);
                let draw_exists = scene.commands.iter().any(|cmd| {
                    matches!(cmd, crate::scene::DrawCommand::Sprite(draw) if draw.texture_id == texture_id)
                });
                let pos = sprite.effective_position();
                (
                    handle.0,
                    sprite.visible,
                    sprite.color.alpha(),
                    [
                        pos.x,
                        pos.y,
                        pos.x.saturating_add(sprite.source_rect.width()),
                        pos.y.saturating_add(sprite.source_rect.height()),
                    ],
                    draw_exists,
                )
            })
        });
        let name_sprite_info = self.text_state.name_sprite.and_then(|handle| {
            sprites.get(handle).map(|sprite| {
                let texture_id = SceneTextureId(sprite.surface.0);
                let draw_exists = scene.commands.iter().any(|cmd| {
                    matches!(cmd, crate::scene::DrawCommand::Sprite(draw) if draw.texture_id == texture_id)
                });
                (handle.0, sprite.visible, sprite.color.alpha(), draw_exists)
            })
        });
        log::debug!(
            "[trace-text] baseline-state frame={frame} pc=0x{:08X} status={} initialized={} visible={} reveal_enabled={} reveal_remaining_ms={} reveal_complete={} last_text_value={} args={:?} sprite={:?} name_sprite={:?} text_panel_visible={} text_draw_command_exists={} input_key_push=0x{:08X} input_key_on=0x{:08X} input_mouse_push=0x{:02X} input_mouse_on=0x{:02X} ctrl_held={} auto_advance_enabled=false",
            self.pc,
            status,
            self.text_state.initialized,
            self.text_state.visible,
            self.text_state.reveal_enabled,
            self.text_reveal_remaining_ms(),
            self.text_reveal_remaining_ms() == 0,
            self.text_state.last_text_value,
            self.text_state.last_text_args,
            sprite_info,
            name_sprite_info,
            sprite_info.is_some_and(|(_, visible, alpha, _, _)| visible && alpha > 0),
            sprite_info.is_some_and(|(_, _, _, _, draw_exists)| draw_exists),
            input.raw_key_push(),
            input.raw_key_on(),
            input.raw_mouse_push(),
            input.raw_mouse_on(),
            input.fast_forward_held(),
        );
    }

    pub fn dump_button_states(
        &self,
        sprites: &SpriteSystem,
        frame: u64,
        mouse_x: i32,
        mouse_y: i32,
    ) {
        if self.game_buttons.is_empty() {
            return;
        }
        let hovered = self.button_hit_at(sprites, mouse_x, mouse_y, -1);
        log::debug!(
            "[trace-buttons] frame={frame} mouse=({mouse_x},{mouse_y}) hovered={hovered:?} count={}",
            self.game_buttons.len()
        );
        for ((group, index), entry) in &self.game_buttons {
            let sprite = sprites.get(entry.handle);
            let visible = sprite.map_or(false, |s| s.visible);
            let sprite_alpha = sprite.map_or(0u8, |s| (s.color.0 >> 24) as u8);
            let priority = sprite.map_or(0, |s| s.effective_priority());
            let rect = entry.hit_rect.unwrap_or_else(|| {
                sprite.map_or([0, 0, 0, 0], |s| {
                    let pos = s.effective_position();
                    [
                        pos.x,
                        pos.y,
                        pos.x.saturating_add(s.source_rect.width()),
                        pos.y.saturating_add(s.source_rect.height()),
                    ]
                })
            });
            let hit = hovered == Some((*group, *index));
            let why_no_hit = if hit {
                ""
            } else if !entry.enabled {
                " [disabled]"
            } else if entry.locked {
                " [locked]"
            } else if !visible {
                " [not_visible]"
            } else if sprite_alpha == 0 {
                " [alpha=0]"
            } else {
                " [out_of_rect]"
            };
            log::debug!(
                "[trace-buttons]   group={group} index={index} name={:?} \
                 rect=[{},{},{},{}] vis={} en={} lock={} alpha={}/{} tog={} prio={} handle={}{hit_str}",
                entry.name,
                rect[0], rect[1], rect[2], rect[3],
                visible, entry.enabled, entry.locked,
                entry.alpha, sprite_alpha,
                entry.toggle, priority, entry.handle.0,
                hit_str = if hit { " [HIT]" } else { why_no_hit },
            );
        }
    }

    fn pop_latched_button_push(&mut self, group: i32) -> Option<i32> {
        if group >= 0 {
            let queue = self.button_push_queue.get_mut(&group)?;
            let value = queue.pop_front();
            if queue.is_empty() {
                self.button_push_queue.remove(&group);
            }
            return value;
        }
        let first_group = self
            .button_push_queue
            .iter()
            .find_map(|(button_group, queue)| (!queue.is_empty()).then_some(*button_group))?;
        let queue = self.button_push_queue.get_mut(&first_group)?;
        let value = queue.pop_front();
        if queue.is_empty() {
            self.button_push_queue.remove(&first_group);
        }
        value
    }

    fn clear_button_capture(&mut self, group: i32, index: i32) {
        let matches = |key: (i32, i32)| {
            (group < 0 || group == key.0) && (index < 0 || index == key.1)
        };
        if self.pressed_button.is_some_and(matches) {
            self.pressed_button = None;
        }
        if self.button_release_capture.is_some_and(matches) {
            self.button_release_capture = None;
        }
        self.button_group_release_capture.retain(|&g, i| !matches((g, *i)));
    }

    fn forget_button_handles(&mut self, group: i32, index: i32) {
        self.clear_button_capture(group, index);
        let keys = self
            .game_buttons
            .keys()
            .copied()
            .filter(|(button_group, button_index)| {
                (group < 0 || *button_group == group) && (index < 0 || *button_index == index)
            })
            .collect::<Vec<_>>();
        for key in keys {
            self.game_buttons.remove(&key);
        }
        if group < 0 {
            self.button_push_queue.clear();
        } else {
            self.button_push_queue.remove(&group);
        }
    }

    fn logical_size(&self) -> (u32, u32) {
        ini_graphics_size(
            self.system_ini.as_ref(),
            FrameScene::PAL_DEFAULT_WIDTH,
            FrameScene::PAL_DEFAULT_HEIGHT,
        )
    }
}

fn dynamic_string_index(value: i32) -> Option<usize> {
    let raw = value as u32;
    if (raw & 0x1000_0000) == 0 {
        return None;
    }
    Some((raw & 0x0FFF_FFFF) as usize)
}

fn is_dynamic_string_handle(value: i32) -> bool {
    dynamic_string_index(value).is_some()
}

const IMAGE_EXTENSIONS: &[&str] = &["", ".PGD", ".pgd"];
const FONT_SHEET_EXTENSIONS: &[&str] = &["", ".TGA", ".tga", ".PGD", ".pgd"];
const FONT_DATA_EXTENSIONS: &[&str] = &["", ".DAT", ".dat"];
const MASK_IMAGE_EXTENSIONS: &[&str] = &["", ".TGA", ".tga", ".PGD", ".pgd"];
const ANIMATION_EXTENSIONS: &[&str] = &["", ".ANI", ".ani"];
const AUDIO_EXTENSIONS: &[&str] = &[
    "", ".OGG", ".ogg", ".WAV", ".wav", ".WMA", ".wma", ".MIX", ".mix",
];
// Game.exe 0x49C3D0 probes WMV, MPG, then AVI. The debug PAL_d.dll
// MP4 branch is incomplete and does not establish release-build support.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NamedSpriteAnimation {
    MoveBy {
        dx: i32,
        dy: i32,
        dz: i32,
        duration_ms: u32,
    },
    ScaleBy {
        delta_percent: i32,
        duration_ms: u32,
    },
    AlphaBy {
        delta: i32,
        duration_ms: u32,
    },
}

fn parse_named_sprite_animation(bytes: &[u8]) -> Option<NamedSpriteAnimation> {
    if bytes.len() >= 0x1a && bytes[0] == 0xfc && bytes[1] == 0x13 && bytes[2] == 0xfd {
        let property = bytes[3];
        return match property {
            0 => Some(NamedSpriteAnimation::MoveBy {
                dx: read_i32_le_at(bytes, 0x04)?,
                dy: read_i32_le_at(bytes, 0x08)?,
                dz: read_i32_le_at(bytes, 0x0c)?,
                duration_ms: read_duration_le_at(bytes, 0x10)?,
            }),
            2 => Some(NamedSpriteAnimation::ScaleBy {
                delta_percent: read_i32_le_at(bytes, 0x04)?,
                duration_ms: read_duration_le_at(bytes, 0x08)?,
            }),
            3 => Some(NamedSpriteAnimation::AlphaBy {
                delta: read_i32_le_at(bytes, 0x04)?,
                duration_ms: read_duration_le_at(bytes, 0x08)?,
            }),
            other => {
                log::debug!("[trace-sprite] unsupported named animation property={other}");
                None
            }
        };
    }

    if bytes.len() >= 0x10 && bytes[0] == 0xfd && bytes[1] == 3 {
        return Some(NamedSpriteAnimation::AlphaBy {
            delta: i32::from(i8::from_ne_bytes([bytes[2]])),
            duration_ms: read_duration_le_at(bytes, 0x06)?,
        });
    }

    None
}

fn read_i32_le_at(bytes: &[u8], offset: usize) -> Option<i32> {
    let raw = bytes.get(offset..offset + 4)?;
    Some(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_duration_le_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(read_i32_le_at(bytes, offset)?.max(1) as u32)
}

fn place_sprite(
    arg_count: usize,
    raw_x: i32,
    raw_y: i32,
    raw_z: i32,
    width: u32,
    height: u32,
    default_x: i32,
    default_y: i32,
    logical_width: u32,
    logical_height: u32,
) -> (i32, i32, i32) {
    if raw_x == 0xFFFF && raw_y == 0xFFFF {
        return (default_x, default_y, raw_z);
    }
    if arg_count >= 5 {
        // VmExtcall_SpSetEx @ 0x429350 multiplies script-space x/y by 1.5
        // when running the native 1920x1080 build.  The portable runtime uses
        // the configured logical stage instead of hard-coding 1920x1080:
        // script coordinates are authored against 1280x720, then scaled into
        // the active PAL logical coordinate space.  0xFFFF/0xFFFF is handled
        // above as the "use resource default position" sentinel.
        return (
            scale_script_x(raw_x, logical_width),
            scale_script_y(raw_y, logical_height),
            raw_z,
        );
    }
    let (x, y) = (
        logical_place_sprite_mode_x(raw_x, width, logical_width),
        logical_place_sprite_mode_y(raw_y, height, logical_height),
    );
    (x, y, 0)
}

fn native_place_sprite(
    arg_count: usize,
    raw_x: i32,
    raw_y: i32,
    raw_z: i32,
    width: u32,
    height: u32,
    default_x: i32,
    default_y: i32,
    logical_width: u32,
    logical_height: u32,
) -> (f32, f32, f32) {
    if raw_x == 0xFFFF && raw_y == 0xFFFF {
        // PGD default offsets are authored in the release's own stage space:
        // 1280x720 for the original release, 1920x1080 for the FHD releases
        // that re-author the same artwork at 1.5x. The wrapper stores native
        // 1920x1080 positions before projecting back to the configured logical
        // stage; convert these offsets just once, by that same stage scale.
        let to_native_x = 1920.0 / logical_width.max(1) as f32;
        let to_native_y = 1080.0 / logical_height.max(1) as f32;
        return (
            default_x as f32 * to_native_x,
            default_y as f32 * to_native_y,
            raw_z as f32,
        );
    }
    if arg_count >= 5 {
        let mut x = raw_x;
        let mut y = raw_y;
        if x <= 4096 {
            x = ((x as f32) * 1.5) as i32;
            y = ((y as f32) * 1.5) as i32;
        }
        return (x as f32, y as f32, raw_z as f32);
    }
    // The portable wrapper normalizes positions to 1920x1080 internally.
    // Native helpers use each build's stage and unscaled source dimensions:
    // Koikake 0x41ECB0 uses 1280x720, reference 0x423550 uses 1920x1080.
    (
        unproject_script_x(
            logical_place_sprite_mode_x(raw_x, width, logical_width),
            logical_width,
        ),
        unproject_script_y(
            logical_place_sprite_mode_y(raw_y, height, logical_height),
            logical_height,
        ),
        0.0,
    )
}

fn render_from_native_wrapper(
    visual: GameSpriteWrapperVisual,
    logical_width: u32,
    logical_height: u32,
) -> (f32, f32, f32, f32) {
    let project_x = logical_width.max(1) as f32 / 1920.0;
    let project_y = logical_height.max(1) as f32 / 1080.0;
    let x = visual.native_x * project_x;
    let y = visual.native_y * project_y;
    // Game.exe `sub_4494D0` passes the wrapper scale lane directly to
    // PAL.dll `PalSpriteSetScale`; PAL.dll `PalSpriteSetScale_0` stores that
    // bare float at PalSprite+0x70. The native 1920x1080 -> configured logical
    // projection belongs to position lanes only. Centering for scale != 1.0 is
    // performed by PAL's draw path (`sub_1028D740`), mirrored by
    // `PalSprite::draw_command_inner`.
    let scale = pal_scale_to_factor(visual.raw_scale);
    (x, y, visual.native_z, scale)
}

fn native_delta_for_visual(visual: GameSpriteWrapperVisual, x: i32, y: i32) -> (f32, f32) {
    if visual.project_native_draw {
        (x as f32, y as f32)
    } else {
        (native_delta_x(x), native_delta_y(y))
    }
}

fn scale_script_x(value: i32, logical_width: u32) -> i32 {
    if value > 4096 {
        return value;
    }
    ((value as f32) * (logical_width as f32 / 1280.0)) as i32
}

fn scale_script_y(value: i32, logical_height: u32) -> i32 {
    if value > 4096 {
        return value;
    }
    ((value as f32) * (logical_height as f32 / 720.0)) as i32
}

fn logical_place_sprite_mode_x(mode: i32, width: u32, logical_width: u32) -> i32 {
    let width = width as i32;
    let logical_width = logical_width as i32;
    if mode < 0 {
        -width
    } else if mode <= 20 {
        logical_width.saturating_mul(mode) / 20 - width / 2
    } else {
        logical_width
    }
}

fn logical_place_sprite_mode_y(mode: i32, height: u32, logical_height: u32) -> i32 {
    let height = height as i32;
    let logical_height = logical_height as i32;
    match mode {
        0 => 0,
        1 => logical_height - height / 2,
        2 => logical_height - height,
        other if other < 0 => -height,
        other => other - 2,
    }
}

fn unproject_script_x(value: i32, logical_width: u32) -> f32 {
    value as f32 * 1920.0 / logical_width.max(1) as f32
}

fn unproject_script_y(value: i32, logical_height: u32) -> f32 {
    value as f32 * 1080.0 / logical_height.max(1) as f32
}

fn native_delta_x(value: i32) -> f32 {
    ((value as f32) * 1.5) as i32 as f32
}

fn native_delta_y(value: i32) -> f32 {
    ((value as f32) * 1.5) as i32 as f32
}

fn open_resource_variant(
    resource_manager: &mut ResourceManager,
    name: &str,
    extensions: &[&str],
) -> pal_asset::Result<pal_asset::LoadedAsset> {
    let has_extension = name
        .rsplit(['/', '\\'])
        .next()
        .is_some_and(|leaf| leaf.contains('.'));
    if has_extension {
        return resource_manager.open(name);
    }
    let mut last_error = None;
    for ext in extensions {
        let candidate = format!("{name}{ext}");
        match resource_manager.open(&candidate) {
            Ok(asset) => return Ok(asset),
            Err(err) => last_error = Some(err),
        }
    }
    Err(
        last_error.unwrap_or_else(|| pal_asset::AssetError::AssetNotFound {
            name: name.to_owned(),
        }),
    )
}

fn decode_asset_image(
    resource_manager: &mut ResourceManager,
    asset: &LoadedAsset,
) -> anyhow::Result<DecodedImage> {
    let mut resolver = |name: &str| -> anyhow::Result<Vec<u8>> {
        Ok(open_resource_variant(resource_manager, name, IMAGE_EXTENSIONS)?.bytes)
    };
    decode_image_with_resolver(&asset.bytes, &mut resolver)
}

fn is_resource_clear_sentinel(name: &str) -> bool {
    name == "#"
}

fn is_named_animation_resource(name: &str) -> bool {
    name.rsplit(['/', '\\']).next().is_some_and(|leaf| {
        leaf.eq_ignore_ascii_case("ANI")
            || leaf.to_ascii_uppercase().starts_with("ANI_")
            || leaf.to_ascii_uppercase().ends_with(".ANI")
    })
}

fn is_fullscreen_solid_layer(name: &str) -> bool {
    name.eq_ignore_ascii_case("BK_BLACK")
        || name.eq_ignore_ascii_case("BK_WHITE")
}

fn expanded_solid_extent(base: u32, raw_offset: i32) -> u32 {
    if raw_offset < 0 {
        base.saturating_add(raw_offset.unsigned_abs().min(base))
    } else {
        base
    }
}

fn parse_solid_color_name_argb(name: &str) -> Option<(u8, u8, u8, u8)> {
    if name == "#" {
        return Some((0, 0, 0, 0));
    }
    if name.eq_ignore_ascii_case("BK_BLACK") {
        return Some((255, 0, 0, 0));
    }
    if name.eq_ignore_ascii_case("BK_WHITE") {
        return Some((255, 255, 255, 255));
    }
    let hex = name.strip_prefix('#')?;
    let normalized;
    let hex = match hex.len() {
        6 | 8 => hex,
        7 => {
            normalized = format!("0{hex}");
            normalized.as_str()
        }
        _ => return None,
    };
    let raw = u32::from_str_radix(hex, 16).ok()?;
    Some(match hex.len() {
        6 => (
            255,
            ((raw >> 16) & 0xFF) as u8,
            ((raw >> 8) & 0xFF) as u8,
            (raw & 0xFF) as u8,
        ),
        8 => (
            ((raw >> 24) & 0xFF) as u8,
            ((raw >> 16) & 0xFF) as u8,
            ((raw >> 8) & 0xFF) as u8,
            (raw & 0xFF) as u8,
        ),
        _ => return None,
    })
}

fn find_loose_save_file(root: &Path, filename: &str) -> Option<PathBuf> {
    let candidates = [
        root.join(filename),
        root.join("data").join(filename),
        root.join("存档").join(filename),
        root.join("補丁").join("存档").join(filename),
        root.join("patch").join("save").join(filename),
        pal_asset::vfs::save_directory(root).join(filename),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

fn write_save_lock_dword(path: &Path, value: i32) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;
    let mut header = [0_u8; 8];
    file.read_exact(&mut header)?;
    if &header == b"SENARSAV" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "portable-only save has no lock dword",
        ));
    }
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&value.to_le_bytes())
}

fn portable_save_dir(root: &Path) -> PathBuf {
    pal_asset::vfs::save_directory(root).join("sena_rs")
}

fn portable_save_path(root: &Path, slot: i32) -> PathBuf {
    let filename = if slot == -1 {
        "continue.sav".to_owned()
    } else {
        format!("save{slot:03}.sav")
    };
    portable_save_dir(root).join(filename)
}

fn portable_system_data_path(root: &Path) -> PathBuf {
    portable_save_dir(root).join("system.ini")
}

fn portable_system_mem_path(root: &Path) -> PathBuf {
    portable_save_dir(root).join("system_mem.bin")
}

fn portable_read_lines_path(root: &Path) -> PathBuf {
    portable_save_dir(root).join("read_lines.bin")
}

const PORTABLE_READ_LINES_MAGIC: &[u8; 8] = b"SENARLIN";
const PORTABLE_READ_LINES_VERSION: u32 = 1;

fn encode_portable_read_lines(ids: &BTreeSet<u32>) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(16 + ids.len() * 4);
    bytes.extend_from_slice(PORTABLE_READ_LINES_MAGIC);
    bytes.extend_from_slice(&PORTABLE_READ_LINES_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(ids.len() as u32).to_le_bytes());
    for id in ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    bytes
}

fn decode_portable_read_lines(bytes: &[u8]) -> std::io::Result<BTreeSet<u32>> {
    if bytes.len() < 16 || &bytes[..8] != PORTABLE_READ_LINES_MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bad SENARLIN header",
        ));
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    if version != PORTABLE_READ_LINES_VERSION {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unsupported SENARLIN version {version}"),
        ));
    }
    let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if bytes.len() < 16 + count * 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "truncated SENARLIN body",
        ));
    }
    let mut ids = BTreeSet::new();
    for chunk in bytes[16..16 + count * 4].chunks_exact(4) {
        ids.insert(u32::from_le_bytes(chunk.try_into().unwrap()));
    }
    Ok(ids)
}

const PORTABLE_SYSTEM_MEM_MAGIC: &[u8; 8] = b"SENARMEM";
const PORTABLE_SYSTEM_MEM_VERSION: u32 = 1;

fn encode_portable_system_mem(words: &[i32]) -> Vec<u8> {
    // Trim trailing zeros so the file stays small; zeros are the default state.
    let count = words.iter().rposition(|value| *value != 0).map_or(0, |i| i + 1);
    let mut bytes = Vec::with_capacity(12 + count * 4);
    bytes.extend_from_slice(PORTABLE_SYSTEM_MEM_MAGIC);
    bytes.extend_from_slice(&PORTABLE_SYSTEM_MEM_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(count as u32).to_le_bytes());
    for value in &words[..count] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn decode_portable_system_mem(bytes: &[u8]) -> std::io::Result<Vec<i32>> {
    if bytes.len() < 12 || &bytes[..8] != PORTABLE_SYSTEM_MEM_MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bad SENARMEM header",
        ));
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
    if version != PORTABLE_SYSTEM_MEM_VERSION {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unsupported SENARMEM version {version}"),
        ));
    }
    let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    if bytes.len() < 16 + count * 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "truncated SENARMEM body",
        ));
    }
    let mut words = Vec::with_capacity(count);
    for chunk in bytes[16..16 + count * 4].chunks_exact(4) {
        words.push(i32::from_le_bytes(chunk.try_into().unwrap()));
    }
    Ok(words)
}

fn encode_runtime_save_snapshot(snapshot: &RuntimeSaveSnapshot) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SENARSAV");
    write_u32(&mut bytes, PORTABLE_SAVE_VERSION)?;
    write_u32(&mut bytes, snapshot.pc)?;
    write_u32_vec(&mut bytes, &snapshot.call_stack)?;
    write_i32_vec(
        &mut bytes,
        &bounded_i32_copy(&snapshot.user_mem, DEFAULT_MEM_SIZE),
    )?;
    write_i32_vec(
        &mut bytes,
        &bounded_i32_copy(&snapshot.system_mem, DEFAULT_MEM_SIZE),
    )?;
    write_i32_vec(
        &mut bytes,
        &bounded_i32_copy(&snapshot.temp_mem, DEFAULT_MEM_SIZE),
    )?;
    write_i32_vec(
        &mut bytes,
        &bounded_i32_copy(&snapshot.mem_dat_words, SAVE_MEMDAT_CAP),
    )?;
    write_u32(&mut bytes, snapshot.history_records.len() as u32)?;
    for record in &snapshot.history_records {
        for value in record {
            write_i32(&mut bytes, *value)?;
        }
    }
    for value in snapshot.text_args {
        write_i32(&mut bytes, value)?;
    }
    write_i32(&mut bytes, snapshot.text_base)?;
    write_i32(&mut bytes, snapshot.text_mode)?;
    bytes.write_all(&[u8::from(snapshot.text_visible)])?;
    write_i32(&mut bytes, snapshot.argument_base)?;
    write_i32_vec(
        &mut bytes,
        &bounded_i32_copy(&snapshot.vars, DEFAULT_VAR_COUNT),
    )?;
    write_i32_vec(
        &mut bytes,
        &bounded_i32_copy(&snapshot.stack, DEFAULT_STACK_LIMIT),
    )?;
    write_i32_vec(
        &mut bytes,
        &bounded_i32_copy(&snapshot.argument_stack, DEFAULT_STACK_LIMIT),
    )?;
    bytes.write_all(&[u8::from(snapshot.text_initialized)])?;
    for value in snapshot.text_init_args {
        write_i32(&mut bytes, value)?;
    }
    write_u32(&mut bytes, snapshot.text_color)?;
    write_u32(&mut bytes, snapshot.text_effect_color)?;
    bytes.write_all(&[u8::from(snapshot.show_wait_mark)])?;
    write_bytes(&mut bytes, &snapshot.title_bytes, SAVE_NAME_CAP)?;
    write_i32(&mut bytes, snapshot.thumb_width)?;
    write_i32(&mut bytes, snapshot.thumb_height)?;
    write_bytes(&mut bytes, &snapshot.thumb_pixels, SAVE_SPRITE_BYTES_CAP)?;
    write_saved_sprites(&mut bytes, &snapshot.sprites)?;
    write_saved_buttons(&mut bytes, &snapshot.buttons)?;
    write_u32(
        &mut bytes,
        snapshot.button_groups.len().min(SAVE_SPRITE_CAP) as u32,
    )?;
    for group in snapshot.button_groups.iter().take(SAVE_SPRITE_CAP) {
        write_i32(&mut bytes, group.group)?;
        write_i32(&mut bytes, group.normal_image)?;
        write_i32(&mut bytes, group.hover_image)?;
        write_i32(&mut bytes, group.onmouse_index)?;
        write_i32(&mut bytes, group.offmouse_index)?;
        write_i32(&mut bytes, group.push_index)?;
        write_u32(&mut bytes, group.slider_ended as u32)?;
        write_u32(&mut bytes, group.locked as u32)?;
        write_u32(&mut bytes, group.lock_started_ms)?;
        write_u32(&mut bytes, group.lock_duration_ms)?;
        write_i32(&mut bytes, group.raw_toggle)?;
    }
    bytes.write_all(&[u8::from(snapshot.resume_wait_click)])?;
    write_u32(
        &mut bytes,
        snapshot.bgm_tracks.len().min(SAVE_BGM_TRACK_CAP) as u32,
    )?;
    for track in snapshot.bgm_tracks.iter().take(SAVE_BGM_TRACK_CAP) {
        write_i32(&mut bytes, track.slot)?;
        write_bytes(&mut bytes, track.name.as_bytes(), SAVE_NAME_CAP)?;
        bytes.write_all(&[u8::from(track.looping)])?;
        write_i64(&mut bytes, track.loop_start)?;
        write_i64(&mut bytes, track.loop_end)?;
    }
    // Version 7 appends the ADV window alpha.
    write_i32(&mut bytes, snapshot.text_alpha.unwrap_or(0))?;
    // Version 8 freezes history strings independently of mutable script buffers.
    history_query::write_history_texts(&mut bytes, snapshot)?;
    sprite_surface::write_sprite_options(&mut bytes, &snapshot.sprites)?;
    tips::write_tips_snapshot(&mut bytes, &snapshot.tips)?;
    bgv::write_saved_bgv(&mut bytes, &snapshot.bgv_tracks)?;
    actions::snapshot::write(&mut bytes, &snapshot.actions)?;
    children::write(&mut bytes, &snapshot.children)?;
    ui_snapshot::write(&mut bytes, snapshot.ui.as_ref())?;
    ui_snapshot::write_strings(&mut bytes, snapshot.dynamic_strings.as_ref())?;
    host_checkpoint::write(&mut bytes,snapshot.host_checkpoint.as_ref())?;
    Ok(bytes)
}

fn write_runtime_save_snapshot(
    root: &Path,
    slot: i32,
    snapshot: &RuntimeSaveSnapshot,
) -> std::io::Result<PathBuf> {
    let path = original_save_path(root, slot);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let (width, height) = (DEFAULT_THUMB_WIDTH, DEFAULT_THUMB_HEIGHT);
    let prefix = OriginalSavePrefix::empty(width, height);
    let snapshot_bytes = encode_runtime_save_snapshot(snapshot)?;
    std::fs::write(&path, encode_original_save(&prefix, &snapshot_bytes))?;
    Ok(path)
}

fn read_original_save_prefix(root: &Path, slot: i32) -> std::io::Result<OriginalSavePrefix> {
    let path = find_loose_save_file(root, &original_save_filename(slot))
        .unwrap_or_else(|| original_save_path(root, slot));
    let mut file = File::open(path)?;
    let mut header = [0_u8; HEADER_BEFORE_PIXELS];
    file.read_exact(&mut header)?;
    let prefix_len = original_prefix_len(&header).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "not an original save")
    })?;
    let mut bytes = vec![0_u8; prefix_len];
    bytes[..HEADER_BEFORE_PIXELS].copy_from_slice(&header);
    if prefix_len > HEADER_BEFORE_PIXELS {
        file.read_exact(&mut bytes[HEADER_BEFORE_PIXELS..])?;
    }
    decode_original_save(&bytes)
        .map(|(prefix, _)| prefix)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "not an original save"))
}

fn read_runtime_save_snapshot(root: &Path, slot: i32) -> std::io::Result<RuntimeSaveSnapshot> {
    let original = find_loose_save_file(root, &original_save_filename(slot))
        .unwrap_or_else(|| original_save_path(root, slot));
    let original_error = if original.is_file() {
        match read_snapshot_file(&original) {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => Some(error),
        }
    } else { None };
    let legacy = portable_save_path(root, slot);
    if legacy.is_file() {
        return read_snapshot_file(&legacy);
    }
    if let Some(error) = original_error { return Err(error); }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "save snapshot not found",
    ))
}

fn read_snapshot_file(path: &Path) -> std::io::Result<RuntimeSaveSnapshot> {
    let mut file = File::open(path)?;
    let mut magic = [0_u8; 8];
    file.read_exact(&mut magic)?;
    if &magic == b"SENARSAV" {
        file.seek(SeekFrom::Start(0))?;
        return decode_runtime_save_snapshot(&mut file);
    }
    file.seek(SeekFrom::Start(0))?;
    let mut header = [0_u8; HEADER_BEFORE_PIXELS];
    file.read_exact(&mut header)?;
    let prefix_len = original_prefix_len(&header).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "not an original save")
    })? as u64;
    file.seek(SeekFrom::Start(prefix_len))?;
    decode_runtime_save_snapshot(&mut file)
}

fn decode_runtime_save_snapshot(reader: &mut impl Read) -> std::io::Result<RuntimeSaveSnapshot> {
    let mut magic = [0_u8; 8];
    reader.read_exact(&mut magic)?;
    if &magic != b"SENARSAV" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "bad portable save magic",
        ));
    }
    let version = read_u32_from(reader)?;
    if !(1..=PORTABLE_SAVE_VERSION).contains(&version) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unsupported portable save version",
        ));
    }
    let pc = read_u32_from(reader)?;
    let call_stack = read_u32_vec_capped(reader, DEFAULT_STACK_LIMIT)?;
    let user_mem = read_i32_vec_capped(reader, DEFAULT_MEM_SIZE)?;
    let system_mem = read_i32_vec_capped(reader, DEFAULT_MEM_SIZE)?;
    let temp_mem = read_i32_vec_capped(reader, DEFAULT_MEM_SIZE)?;
    let mem_dat_words = read_i32_vec_capped(reader, SAVE_MEMDAT_CAP)?;
    let history_len = read_u32_from(reader)? as usize;
    if history_len > 10_000 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save history is too large",
        ));
    }
    let mut history_records = Vec::with_capacity(history_len);
    for _ in 0..history_len {
        let mut record = [0_i32; 9];
        for value in &mut record {
            *value = read_i32_from(reader)?;
        }
        history_records.push(record);
    }
    let mut text_args = [0_i32; 4];
    for value in &mut text_args {
        *value = read_i32_from(reader)?;
    }
    let text_base = read_i32_from(reader)?;
    let text_mode = read_i32_from(reader)?;
    let mut visible = [0_u8; 1];
    reader.read_exact(&mut visible)?;
    let mut snapshot = RuntimeSaveSnapshot {
        version,
        pc,
        call_stack,
        user_mem,
        system_mem,
        temp_mem,
        mem_dat_words,
        history_records,
        text_args,
        text_base,
        text_mode,
        text_visible: visible[0] != 0,
        ..RuntimeSaveSnapshot::default()
    };
    if version >= 2 {
        snapshot.argument_base = read_i32_from(reader)?;
        snapshot.vars = read_i32_vec_capped(reader, DEFAULT_VAR_COUNT)?;
        snapshot.stack = read_i32_vec_capped(reader, DEFAULT_STACK_LIMIT)?;
        snapshot.argument_stack = read_i32_vec_capped(reader, DEFAULT_STACK_LIMIT)?;
        let mut flag = [0_u8; 1];
        reader.read_exact(&mut flag)?;
        snapshot.text_initialized = flag[0] != 0;
        for value in &mut snapshot.text_init_args {
            *value = read_i32_from(reader)?;
        }
        snapshot.text_color = read_u32_from(reader)?;
        snapshot.text_effect_color = read_u32_from(reader)?;
        reader.read_exact(&mut flag)?;
        snapshot.show_wait_mark = flag[0] != 0;
        snapshot.title_bytes = read_bytes_capped(reader, SAVE_NAME_CAP)?;
        snapshot.thumb_width = read_i32_from(reader)?;
        snapshot.thumb_height = read_i32_from(reader)?;
        snapshot.thumb_pixels = read_bytes_capped(reader, SAVE_SPRITE_BYTES_CAP)?;
        snapshot.sprites = read_saved_sprites(reader, version)?;
        snapshot.buttons = read_saved_buttons(reader, version)?;
        let group_len = read_u32_from(reader)? as usize;
        if group_len > SAVE_SPRITE_CAP {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "save button groups are too large",
            ));
        }
        for _ in 0..group_len {
            snapshot.button_groups.push(SavedButtonGroup {
                group: read_i32_from(reader)?,
                normal_image: read_i32_from(reader)?,
                hover_image: read_i32_from(reader)?,
                onmouse_index: read_i32_from(reader)?,
                offmouse_index: if version >= 12 { read_i32_from(reader)? } else { 0 },
                push_index: if version >= 12 { read_i32_from(reader)? } else { 0 },
                slider_ended: version >= 12 && read_u32_from(reader)? != 0,
                locked: version >= 12 && read_u32_from(reader)? != 0,
                lock_started_ms: if version >= 12 { read_u32_from(reader)? } else { 0 },
                lock_duration_ms: if version >= 12 { read_u32_from(reader)? } else { 0 },
                raw_toggle: if version >= 14 { read_i32_from(reader)? } else { 0 },
            });
        }
        if version >= 3 {
            reader.read_exact(&mut flag)?;
            snapshot.resume_wait_click = flag[0] != 0;
        }
        if version >= 5 {
            let bgm_len = read_u32_from(reader)? as usize;
            if bgm_len > SAVE_BGM_TRACK_CAP {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "save bgm track list is too large",
                ));
            }
            for _ in 0..bgm_len {
                let slot = read_i32_from(reader)?;
                let name_bytes = read_bytes_capped(reader, SAVE_NAME_CAP)?;
                let name = String::from_utf8_lossy(&name_bytes).into_owned();
                reader.read_exact(&mut flag)?;
                let looping = flag[0] != 0;
                let loop_start = read_i64_from(reader)?;
                let loop_end = read_i64_from(reader)?;
                snapshot.bgm_tracks.push(SavedBgmTrack {
                    slot,
                    name,
                    looping,
                    loop_start,
                    loop_end,
                });
            }
        }
        if version >= 7 {
            snapshot.text_alpha = Some(read_i32_from(reader)?);
        }
        if version >= 8 {
            snapshot.history_texts = history_query::read_history_texts(reader, history_len)?;
        }
        if version >= 9 {
            sprite_surface::read_sprite_options(reader, &mut snapshot.sprites)?;
        }
        if version >= 10 {
            snapshot.tips = tips::read_tips_snapshot(reader)?;
            snapshot.bgv_tracks = bgv::read_saved_bgv(reader)?;
        }
        if version >= 11 {
            snapshot.actions = actions::snapshot::read(reader, version)?;
        }
        if version >= 13 {
            snapshot.children = children::read(reader)?;
        }
        if version >= 15 {
            snapshot.ui = ui_snapshot::read(reader)?;
            snapshot.dynamic_strings = ui_snapshot::read_strings(reader)?;
        }
    }
    if version>=16 { snapshot.host_checkpoint=host_checkpoint::read(reader)?; }
    Ok(snapshot)
}

fn write_i32(out: &mut Vec<u8>, value: i32) -> std::io::Result<()> {
    out.write_all(&value.to_le_bytes())
}

fn write_u32(out: &mut Vec<u8>, value: u32) -> std::io::Result<()> {
    out.write_all(&value.to_le_bytes())
}

fn write_i64(out: &mut Vec<u8>, value: i64) -> std::io::Result<()> {
    out.write_all(&value.to_le_bytes())
}

fn write_i32_vec(out: &mut Vec<u8>, values: &[i32]) -> std::io::Result<()> {
    write_u32(out, values.len() as u32)?;
    for value in values {
        write_i32(out, *value)?;
    }
    Ok(())
}

fn write_u32_vec(out: &mut Vec<u8>, values: &[u32]) -> std::io::Result<()> {
    write_u32(out, values.len() as u32)?;
    for value in values {
        write_u32(out, *value)?;
    }
    Ok(())
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8], cap: usize) -> std::io::Result<()> {
    let bytes = if bytes.len() > cap {
        &bytes[..cap]
    } else {
        bytes
    };
    write_u32(out, bytes.len() as u32)?;
    out.write_all(bytes)
}

fn write_saved_sprites(out: &mut Vec<u8>, sprites: &[SavedSprite]) -> std::io::Result<()> {
    let sprites = sprites
        .iter()
        .filter(|sprite| sprite.rgba.len() <= CHECKPOINT_SPRITE_BYTES_CAP)
        .take(SAVE_SPRITE_CAP);
    let sprites: Vec<&SavedSprite> = sprites.collect();
    write_u32(out, sprites.len() as u32)?;
    let mut shared = std::collections::HashMap::new();
    for sprite in sprites {
        write_i32(out, sprite.slot)?;
        write_i32(out, sprite.x)?;
        write_i32(out, sprite.y)?;
        write_i32(out, sprite.z)?;
        write_i32(out, sprite.offset_x)?;
        write_i32(out, sprite.offset_y)?;
        write_i32(out, sprite.priority)?;
        write_u32(out, sprite.scale_bits)?;
        write_u32(out, sprite.color)?;
        out.write_all(&[u8::from(sprite.visible)])?;
        for value in sprite.rect {
            write_i32(out, value)?;
        }
        write_u32(out, sprite.width)?;
        write_u32(out, sprite.height)?;
        let (project_x, project_y) = sprite.native_projection.unwrap_or((-1.0, -1.0));
        write_u32(out, project_x.to_bits())?;
        write_u32(out, project_y.to_bits())?;
        out.write_all(&[u8::from(sprite.center_scale)])?;
        let pointer = Arc::as_ptr(&sprite.rgba) as usize;
        let next = shared.len() as u32;
        let id = *shared.entry(pointer).or_insert(next);
        write_u32(out, id)?;
        if id == next { write_bytes(out, &sprite.rgba, CHECKPOINT_SPRITE_BYTES_CAP)?; }
        write_u32(out, u32::from(sprite.scale_y_bits.is_some()))?;
        write_u32(out, sprite.scale_y_bits.unwrap_or(0))?;
        write_u32(out,u32::from(sprite.transition_source))?;
    }
    Ok(())
}

fn write_saved_buttons(out: &mut Vec<u8>, buttons: &[SavedButton]) -> std::io::Result<()> {
    let buttons: Vec<&SavedButton> = buttons
        .iter()
        .filter(|button| button.rgba.len() <= SAVE_SPRITE_BYTES_CAP)
        .take(SAVE_SPRITE_CAP)
        .collect();
    write_u32(out, buttons.len() as u32)?;
    for button in buttons {
        write_i32(out, button.group)?;
        write_i32(out, button.index)?;
        out.write_all(&[u8::from(button.visible), button.enabled as u8, button.alpha])?;
        write_i32(out, button.gosub_point)?;
        write_i32(out, button.x)?;
        write_i32(out, button.y)?;
        write_i32(out, button.z)?;
        write_i32(out, button.priority)?;
        write_u32(out, button.width)?;
        write_u32(out, button.height)?;
        write_u32(out, button.cell_width)?;
        write_u32(out, button.cell_height)?;
        for value in button.rect {
            write_i32(out, value)?;
        }
        write_u32(out, button.color)?;
        write_bytes(out, button.name.as_bytes(), SAVE_NAME_CAP)?;
        write_bytes(out, &button.rgba, SAVE_SPRITE_BYTES_CAP)?;
        out.write_all(&button.registration_order.to_le_bytes())?;
        write_u32(out, button.native_flags)?;
        write_i32(out, button.base_row)?;
        for callback in button.hover_callbacks { write_u32(out, callback.unwrap_or(0))?; }
        for word in button.base_position.into_iter().chain(button.slider_position) { write_i32(out, word)?; }
    }
    Ok(())
}

fn read_i32_from(reader: &mut impl Read) -> std::io::Result<i32> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_u32_from(reader: &mut impl Read) -> std::io::Result<u32> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_i64_from(reader: &mut impl Read) -> std::io::Result<i64> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_le_bytes(bytes))
}

fn read_i32_vec_capped(reader: &mut impl Read, cap: usize) -> std::io::Result<Vec<i32>> {
    let len = read_u32_from(reader)? as usize;
    if len > cap {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save vector is too large",
        ));
    }
    let mut values = Vec::with_capacity(len);
    for _ in 0..len {
        values.push(read_i32_from(reader)?);
    }
    Ok(values)
}

fn read_u32_vec_capped(reader: &mut impl Read, cap: usize) -> std::io::Result<Vec<u32>> {
    let len = read_u32_from(reader)? as usize;
    if len > cap {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save vector is too large",
        ));
    }
    let mut values = Vec::with_capacity(len);
    for _ in 0..len {
        values.push(read_u32_from(reader)?);
    }
    Ok(values)
}

fn read_bytes_capped(reader: &mut impl Read, cap: usize) -> std::io::Result<Vec<u8>> {
    let len = read_u32_from(reader)? as usize;
    if len > cap {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save blob is too large",
        ));
    }
    let mut bytes = vec![0_u8; len];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn validate_saved_rgba(width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4));
    if !matches!(expected, Some(len) if len != 0 && len <= CHECKPOINT_SPRITE_BYTES_CAP && len == rgba.len()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save texture dimensions do not match its RGBA data",
        ));
    }
    Ok(())
}

fn read_saved_sprites(reader: &mut impl Read, version: u32) -> std::io::Result<Vec<SavedSprite>> {
    let len = read_u32_from(reader)? as usize;
    if len > SAVE_SPRITE_CAP {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save sprites are too large",
        ));
    }
    let mut sprites = Vec::with_capacity(len);
    let mut shared: Vec<Arc<Vec<u8>>> = Vec::new();
    for _ in 0..len {
        let slot = read_i32_from(reader)?;
        let x = read_i32_from(reader)?;
        let y = read_i32_from(reader)?;
        let z = read_i32_from(reader)?;
        let offset_x = read_i32_from(reader)?;
        let offset_y = read_i32_from(reader)?;
        let priority = read_i32_from(reader)?;
        let scale_bits = read_u32_from(reader)?;
        let color = read_u32_from(reader)?;
        let mut flags = [0_u8; 1];
        reader.read_exact(&mut flags)?;
        let mut rect = [0_i32; 4];
        for value in &mut rect {
            *value = read_i32_from(reader)?;
        }
        let width = read_u32_from(reader)?;
        let height = read_u32_from(reader)?;
        let (native_projection, center_scale) = if version >= 6 {
            let project_x = f32::from_bits(read_u32_from(reader)?);
            let project_y = f32::from_bits(read_u32_from(reader)?);
            let mut center = [0_u8; 1];
            reader.read_exact(&mut center)?;
            let projection = (project_x >= 0.0 && project_y >= 0.0).then_some((project_x, project_y));
            (projection, center[0] != 0)
        } else {
            (None, false)
        };
        let rgba = if version >= 16 {
            let id = read_u32_from(reader)? as usize;
            if id > shared.len() {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid shared save texture"));
            }
            if id == shared.len() {
                shared.push(Arc::new(read_bytes_capped(reader, CHECKPOINT_SPRITE_BYTES_CAP)?));
            }
            shared[id].clone()
        } else { Arc::new(read_bytes_capped(reader, SAVE_SPRITE_BYTES_CAP)?) };
        validate_saved_rgba(width, height, &rgba)?;
        let scale_y_bits = if version >= 12 {
            let present = read_u32_from(reader)? != 0;
            let bits = read_u32_from(reader)?;
            present.then_some(bits)
        } else {
            None
        };
        let transition_source=version>=16 && read_u32_from(reader)?!=0;
        sprites.push(SavedSprite {
            slot,
            transition_source,
            x,
            y,
            z,
            offset_x,
            offset_y,
            priority,
            scale_bits,
            scale_y_bits,
            color,
            visible: flags[0] != 0,
            rect,
            width,
            height,
            native_projection,
            center_scale,
            rgba,
            option_type: 0,
            option_backup: Vec::new(),
        });
    }
    Ok(sprites)
}

fn read_saved_buttons(reader: &mut impl Read, version: u32) -> std::io::Result<Vec<SavedButton>> {
    let len = read_u32_from(reader)? as usize;
    if len > SAVE_SPRITE_CAP {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "save buttons are too large",
        ));
    }
    let mut buttons = Vec::with_capacity(len);
    for _ in 0..len {
        let group = read_i32_from(reader)?;
        let index = read_i32_from(reader)?;
        let mut flags = [0_u8; 3];
        reader.read_exact(&mut flags)?;
        let gosub_point = read_i32_from(reader)?;
        let x = read_i32_from(reader)?;
        let y = read_i32_from(reader)?;
        let z = read_i32_from(reader)?;
        let priority = read_i32_from(reader)?;
        let width = read_u32_from(reader)?;
        let height = read_u32_from(reader)?;
        // Version 4 records the cell window; older images only carry the full
        // sheet, so fall back to a single whole-texture cell.
        let (cell_width, cell_height, rect, color) = if version >= 4 {
            let cell_width = read_u32_from(reader)?;
            let cell_height = read_u32_from(reader)?;
            let mut rect = [0_i32; 4];
            for value in &mut rect {
                *value = read_i32_from(reader)?;
            }
            let color = read_u32_from(reader)?;
            (cell_width, cell_height, rect, color)
        } else {
            (
                width.max(1),
                height.max(1),
                [0, 0, width as i32, height as i32],
                0xFFFF_FFFF,
            )
        };
        let name = String::from_utf8_lossy(&read_bytes_capped(reader, SAVE_NAME_CAP)?).into_owned();
        let rgba = read_bytes_capped(reader, SAVE_SPRITE_BYTES_CAP)?;
        validate_saved_rgba(width, height, &rgba)?;
        let registration_order = if version >= 12 { read_i64_from(reader)? as u64 } else { buttons.len() as u64 };
        let native_flags = if version >= 12 { read_u32_from(reader)? } else { 1 };
        let base_row = if version >= 12 { read_i32_from(reader)? } else { 0 };
        let mut hover_callbacks = [None; 2];
        if version >= 12 {
            for callback in &mut hover_callbacks {
                let value = read_u32_from(reader)?;
                *callback = (value != 0).then_some(value);
            }
        }
        let (base_position, slider_position) = if version >= 14 {
            ([read_i32_from(reader)?, read_i32_from(reader)?], [read_i32_from(reader)?, read_i32_from(reader)?])
        } else { ([x, y], [0; 2]) };
        buttons.push(SavedButton {
            base_position,
            slider_position,
            registration_order,
            native_flags,
            hover_callbacks,
            base_row,
            group,
            index,
            visible: flags[0] != 0,
            enabled: flags[1] != 0,
            alpha: flags[2],
            gosub_point,
            x,
            y,
            z,
            priority,
            width,
            height,
            cell_width,
            cell_height,
            rect,
            color,
            name,
            rgba,
        });
    }
    Ok(buttons)
}

fn bounded_i32_copy(values: &[i32], cap: usize) -> Vec<i32> {
    values.iter().take(cap).copied().collect()
}

fn install_i32_words(dst: &mut Vec<i32>, src: &[i32], min_len: usize) {
    if dst.len() < min_len {
        dst.resize(min_len, 0);
    }
    for slot in dst.iter_mut() {
        *slot = 0;
    }
    for (index, value) in src.iter().enumerate().take(dst.len()) {
        dst[index] = *value;
    }
}

fn saved_sprite_from_handle(
    sprites: &SpriteSystem,
    slot: i32,
    handle: SpriteHandle,
) -> Option<SavedSprite> {
    saved_sprite_with_limit(sprites, slot, handle, SAVE_SPRITE_BYTES_CAP)
}

fn saved_sprite_with_limit(sprites: &SpriteSystem, slot: i32, handle: SpriteHandle,
    limit: usize) -> Option<SavedSprite> {
    let sprite = sprites.get(handle)?;
    let surface = sprites.surface(sprite.surface)?;
    let texture = surface.to_scene_texture();
    let expected = texture.width as usize * texture.height as usize * 4;
    if expected == 0 || expected > limit || texture.pixels.len() < expected {
        return None;
    }
    Some(SavedSprite {
        slot,
        transition_source: false,
        x: sprite.position.x as i32,
        y: sprite.position.y as i32,
        z: sprite.position.z as i32,
        offset_x: sprite.offset.x,
        offset_y: sprite.offset.y,
        priority: sprite.base_priority,
        scale_bits: sprite.scale.to_bits(),
        scale_y_bits: sprite.scale_y.map(f32::to_bits),
        color: sprite.color.0,
        visible: sprite.visible,
        rect: [
            sprite.source_rect.left,
            sprite.source_rect.top,
            sprite.source_rect.right,
            sprite.source_rect.bottom,
        ],
        width: texture.width,
        height: texture.height,
        native_projection: sprite.native_projection,
        center_scale: sprite.center_scale,
        rgba: texture.pixels.clone(),
        option_type: sprite.option_type,
        option_backup: sprites.option_backup(sprite.handle).unwrap_or_default().to_vec(),
    })
}

fn save_file_modified(resource_manager: Option<&ResourceManager>, slot: i32) -> Option<SystemTime> {
    let manager = resource_manager?;
    let path = find_loose_save_file(manager.root(), &original_save_filename(slot))?;
    std::fs::metadata(path).ok()?.modified().ok()
}

fn decode_save_title(bytes: &[u8], nls: Nls) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    let bytes = &bytes[..end];
    if bytes.is_empty() {
        return String::new();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    nls.decode(bytes)
        .unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned())
}

struct LocalDateTime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    weekday: u32,
}

fn local_date_time(time: SystemTime) -> LocalDateTime {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    #[cfg(unix)]
    if let Some(local) = unix_local_date_time(secs) {
        return local;
    }
    utc_date_time(secs)
}

fn utc_date_time(secs: i64) -> LocalDateTime {
    let days = secs.div_euclid(86_400);
    let tod = secs.rem_euclid(86_400) as u32;
    let weekday = (days + 4).rem_euclid(7) as u32;
    let mut year = 1970_i32;
    let mut day = days;
    loop {
        let year_days = if is_leap(year) { 366 } else { 365 };
        if day < year_days {
            break;
        }
        day -= year_days;
        year += 1;
    }
    let months = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1_u32;
    for days_in_month in months {
        if day < days_in_month {
            break;
        }
        day -= days_in_month;
        month += 1;
    }
    LocalDateTime {
        year,
        month,
        day: day as u32 + 1,
        hour: tod / 3_600,
        minute: (tod / 60) % 60,
        second: tod % 60,
        weekday,
    }
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(unix)]
fn unix_local_date_time(secs: i64) -> Option<LocalDateTime> {
    #[repr(C)]
    struct LibcTm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
        tm_gmtoff: i64,
        tm_zone: *const i8,
    }
    extern "C" {
        fn localtime_r(timer: *const i64, result: *mut LibcTm) -> *mut LibcTm;
    }
    unsafe {
        let mut tm = std::mem::zeroed::<LibcTm>();
        if localtime_r(&secs, &mut tm).is_null() {
            return None;
        }
        Some(LocalDateTime {
            year: tm.tm_year + 1900,
            month: tm.tm_mon as u32 + 1,
            day: tm.tm_mday as u32,
            hour: tm.tm_hour as u32,
            minute: tm.tm_min as u32,
            second: tm.tm_sec as u32,
            weekday: tm.tm_wday as u32,
        })
    }
}

fn format_save_time(modified: SystemTime, format_mode: i32) -> String {
    let local = local_date_time(modified);
    let hour = local.hour;
    let minute = local.minute;
    let second = local.second;
    match format_mode {
        1 => format!("{hour:02}{minute:02}{second:02}"),
        2 => format!("{hour:02}:{minute:02}"),
        3 => format!("{hour:02}{minute:02}"),
        _ => format!("{hour:02}:{minute:02}:{second:02}"),
    }
}

fn format_save_day(modified: SystemTime, format_mode: i32) -> String {
    let local = local_date_time(modified);
    let date = format!(
        "{:02}/{:02}/{:02}",
        local.year.rem_euclid(100),
        local.month,
        local.day
    );
    if format_mode == 0 {
        return date;
    }
    let week = ["日", "月", "火", "水", "木", "金", "土"][local.weekday as usize % 7];
    format!("{date} ({week})")
}

fn parse_pal_text_directives(text: &str) -> (String, Option<u16>) {
    let directive = if let Some(rest) = text.strip_prefix('<') {
        if let Some(end) = rest.find('>') {
            Some(&rest[..end])
        } else {
            None
        }
    } else {
        None
    };
    let mut size = None;
    if let Some(directive) = directive {
        if let Some(value) = directive.strip_prefix('s').and_then(|value| value.parse::<u16>().ok()) {
            size = Some(value.max(1));
        }
        for part in directive.split(|ch| ch == ';' || ch == ',' || ch == ' ') {
            let Some(raw_size) = part.strip_prefix("size=") else {
                continue;
            };
            if let Ok(value) = raw_size.parse::<u16>() {
                size = Some(value.max(1));
            }
        }
    }
    (strip_pal_inline_tags(text), size)
}

fn strip_pal_inline_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut tag = String::new();
            let mut closed = false;
            while let Some(next) = chars.next() {
                if next == '>' {
                    closed = true;
                    break;
                }
                tag.push(next);
            }
            // Newer native text renderer 0x46AB70 recognizes the literal
            // little-endian DWORD '<br>' and resets x while advancing y.
            if closed && tag == "br" {
                out.push('\n');
                continue;
            }
            if closed && is_pal_text_tag(&tag) {
                continue;
            }
            out.push('<');
            out.push_str(&tag);
            if closed {
                out.push('>');
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn wrap_text_lines(font: &PalFontSystem, text: &str, max_width: u32) -> Vec<String> {
    let max_width = max_width.max(1);
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut line = String::new();
        for (unit, _) in font.layout_units(paragraph) {
            let mut candidate = line.clone();
            candidate.push_str(unit);
            let (candidate_width, _) = font.measure(&candidate);
            if !line.is_empty() && candidate_width > max_width {
                lines.push(std::mem::take(&mut line));
                line.push_str(unit);
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn measure_wrapped_text(
    font: &PalFontSystem,
    text: &str,
    max_width: u32,
) -> (u32, u32, Vec<String>) {
    let lines = wrap_text_lines(font, text, max_width);
    let line_gap = (u32::from(font.font_size()).max(12) / 4).max(4);
    let mut width = 1_u32;
    let mut height = 0_u32;
    for (index, line) in lines.iter().enumerate() {
        let (line_width, line_height) = font.measure(line);
        width = width.max(line_width.max(1));
        if index > 0 {
            height = height.saturating_add(line_gap);
        }
        height = height.saturating_add(line_height.max(1));
    }
    (width, height.max(1), lines)
}

/// Rasterize every wrapped line once into a single text block and record the
/// per-line geometry the smooth reveal clips against.
fn rasterize_text_block_with_layout(
    font: &PalFontSystem,
    lines: &[String],
    width: u32,
    height: u32,
) -> (u32, u32, Vec<u8>, Vec<AdvTextLineLayout>) {
    let line_gap = (u32::from(font.font_size()).max(12) / 4).max(4);
    let width = width.max(1);
    let height = height.max(1);
    let mut rgba = vec![0_u8; (width * height * 4) as usize];
    let mut y = 0_u32;
    let mut char_start = 0_usize;
    let mut layouts = Vec::with_capacity(lines.len());
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            y = y.saturating_add(line_gap);
        }
        let line_y = y;
        let units = font.layout_units(line);
        let char_count = units.iter().filter(|(_, visible)| *visible).count();
        let (_, layout_height) = font.measure(line);
        let (line_width, line_height, line_rgba) = if line.is_empty() {
            (0, 0, Vec::new())
        } else {
            font.rasterize(line)
        };
        for sy in 0..line_height {
            for sx in 0..line_width {
                let si = ((sy * line_width + sx) * 4) as usize;
                let Some(src) = line_rgba.get(si..si + 4) else {
                    continue;
                };
                if src[3] == 0 {
                    continue;
                }
                let dx = sx;
                let dy = line_y + sy;
                if dx >= width || dy >= height {
                    continue;
                }
                let di = ((dy * width + dx) * 4) as usize;
                alpha_blend_rgba(&mut rgba[di..di + 4], src);
            }
        }
        let mut char_x = Vec::with_capacity(char_count + 1);
        char_x.push(0);
        let mut prefix_len = 0;
        for (unit, visible) in units {
            prefix_len += unit.len();
            if visible {
                char_x.push(font.measure(&line[..prefix_len]).0);
            }
        }
        let advance = layout_height.max(line_height).max(1);
        layouts.push(AdvTextLineLayout {
            y: line_y,
            height: advance,
            char_start,
            char_count,
            char_x,
        });
        char_start += char_count;
        y = y.saturating_add(advance);
    }
    (width, height, rgba, layouts)
}

fn is_pal_text_tag(tag: &str) -> bool {
    let normalized = tag.trim().trim_start_matches('/').to_ascii_lowercase();
    normalized.is_empty()
        || normalized.strip_prefix('d').is_some_and(|value| value.chars().all(|ch| ch.is_ascii_digit()))
        || normalized.starts_with("s")
        || normalized.starts_with("size=")
        || normalized.starts_with("color")
        || normalized.starts_with("font")
        || normalized.starts_with("ruby")
        || normalized.starts_with("r=")
}

/// ADV window panel without any body text: either the script's base image or
/// the fallback dark window rectangle.
fn adv_text_base_panel(
    panel_text_width: u32,
    panel_text_height: u32,
    min_width: u32,
    min_height: u32,
    base_image: Option<DecodedImage>,
    window_alpha: i32,
) -> (u32, u32, Vec<u8>, u32, u32) {
    let pad_x = 24_u32;
    let pad_y = 18_u32;
    if let Some(base) = base_image {
        let mut rgba = base.rgba;
        let alpha = window_alpha.clamp(0, 255) as u8;
        if alpha < 255 {
            for px in rgba.chunks_exact_mut(4) {
                px[3] = ((u16::from(px[3]) * u16::from(alpha) + 127) / 255) as u8;
            }
        }
        (base.width, base.height, rgba, pad_x, pad_y)
    } else {
        let width = panel_text_width.saturating_add(pad_x * 2).max(min_width);
        let height = panel_text_height.saturating_add(pad_y * 2).max(min_height);
        let mut rgba = vec![0_u8; (width * height * 4) as usize];
        let alpha = if window_alpha > 0 {
            window_alpha.clamp(0, 255) as u8
        } else {
            184
        };
        for px in rgba.chunks_exact_mut(4) {
            px[0] = 16;
            px[1] = 16;
            px[2] = 20;
            px[3] = alpha;
        }
        (width, height, rgba, pad_x, pad_y)
    }
}

/// Composite one presented ADV text frame from the cached panel and text
/// block, clipping body glyphs at the smooth reveal limit and optionally
/// blitting the wait mark. Returns the surface plus its sprite position.
fn compose_adv_text_frame(
    cache: &AdvTextPanelCache,
    reveal_limit: Option<(usize, u32)>,
    animation_progress: Option<u32>,
    wait_mark: Option<(u32, u32, Vec<u8>, u32, u32)>,
) -> (u32, u32, Vec<u8>, i32, i32) {
    let mut rgba = cache.panel_rgba.clone();
    let width = cache.panel_width;
    let height = cache.panel_height;
    for (index, line) in cache.lines.iter().enumerate() {
        let x_limit = match reveal_limit {
            None => u32::MAX,
            Some((limit_line, limit_x)) => {
                if index < limit_line {
                    u32::MAX
                } else if index == limit_line {
                    limit_x
                } else {
                    0
                }
            }
        };
        let x_limit = x_limit.min(cache.text_width);
        if x_limit == 0 {
            continue;
        }
        for sy in 0..line.height {
            let dy = cache
                .text_origin_y
                .saturating_add(line.y)
                .saturating_add(sy);
            if dy >= height {
                break;
            }
            for sx in 0..x_limit {
                let dx = cache.text_origin_x + sx;
                if dx >= width {
                    break;
                }
                let si = (((line.y + sy) * cache.text_width + sx) * 4) as usize;
                let Some(src) = cache.text_rgba.get(si..si + 4) else {
                    continue;
                };
                if src[3] == 0 {
                    continue;
                }
                let di = ((dy * width + dx) * 4) as usize;
                let mut pixel: [u8; 4] = src.try_into().unwrap();
                if let (Some(map), Some(progress)) = (&cache.thresholds, animation_progress) {
                    pixel[3] = map.alpha(si / 4, pixel[3], progress);
                }
                alpha_blend_rgba(&mut rgba[di..di + 4], &pixel);
            }
        }
    }
    if let Some((mark_w, mark_h, mark_rgba, mark_x, mark_y)) = wait_mark {
        blit_rgba(
            &mut rgba,
            width,
            height,
            &mark_rgba,
            mark_w,
            mark_h,
            cache.text_origin_x.saturating_add(mark_x),
            cache
                .text_origin_y
                .saturating_add(mark_y)
                .saturating_add(4),
        );
    }
    (width, height, rgba, cache.sprite_x, cache.sprite_y)
}

fn alpha_blend_rgba(dst: &mut [u8], src: &[u8]) {
    let sa = src[3] as u32;
    let da = dst[3] as u32;
    let out_a = sa + (da * (255 - sa) + 127) / 255;
    if out_a == 0 {
        dst.fill(0);
        return;
    }
    for channel in 0..3 {
        let sc = src[channel] as u32;
        let dc = dst[channel] as u32;
        let premul = sc * sa + (dc * da * (255 - sa) + 127) / 255;
        dst[channel] = ((premul + out_a / 2) / out_a).min(255) as u8;
    }
    dst[3] = out_a.min(255) as u8;
}

fn ini_first_int(ini: &IniFile, key: &str) -> Option<i64> {
    let key = key.to_ascii_lowercase();
    ini.values()
        .find_map(|section| section.get(&key).and_then(|value| value.as_int()))
}

fn ini_first_str(ini: &IniFile, key: &str) -> Option<String> {
    let key = key.to_ascii_lowercase();
    ini.values().find_map(|section| {
        section
            .get(&key)
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

fn argb_to_rgba_bytes(color: u32) -> [u8; 4] {
    [
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8,
        ((color >> 24) & 0xFF) as u8,
    ]
}

fn wait_mark_frame(sheet: &DecodedImage, frame: u32, color: [u8; 4]) -> (u32, u32, Vec<u8>) {
    let cell_h = sheet.height.max(1);
    let frames = if cell_h > 0 && sheet.width.is_multiple_of(cell_h) {
        (sheet.width / cell_h).max(1)
    } else {
        1
    };
    let cell_w = (sheet.width / frames).max(1);
    let frame = frame % frames;
    let mut rgba = vec![0u8; cell_w as usize * cell_h as usize * 4];
    for y in 0..cell_h.min(sheet.height) {
        for x in 0..cell_w {
            let sx = frame * cell_w + x;
            if sx >= sheet.width {
                continue;
            }
            let src_index = (y as usize * sheet.width as usize + sx as usize) * 4;
            let Some(src) = sheet.rgba.get(src_index..src_index + 4) else {
                continue;
            };
            let coverage = src[0].max(src[1]).max(src[2]);
            if coverage == 0 {
                continue;
            }
            let dst_index = (y as usize * cell_w as usize + x as usize) * 4;
            rgba[dst_index] = color[0];
            rgba[dst_index + 1] = color[1];
            rgba[dst_index + 2] = color[2];
            rgba[dst_index + 3] = ((u16::from(coverage) * u16::from(color[3])) / 255) as u8;
        }
    }
    (cell_w, cell_h, rgba)
}

fn fallback_wait_mark(color: [u8; 4]) -> (u32, u32, Vec<u8>) {
    let size = 14u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        let half = y / 2;
        for x in (size / 2 - half)..(size / 2 + half) {
            let index = (y as usize * size as usize + x as usize) * 4;
            rgba[index..index + 4].copy_from_slice(&color);
        }
    }
    (size, size, rgba)
}

fn blit_rgba(
    dst: &mut [u8],
    dst_width: u32,
    dst_height: u32,
    src: &[u8],
    src_width: u32,
    src_height: u32,
    dst_x: u32,
    dst_y: u32,
) {
    for sy in 0..src_height {
        for sx in 0..src_width {
            let dx = dst_x + sx;
            let dy = dst_y + sy;
            if dx >= dst_width || dy >= dst_height {
                continue;
            }
            let si = ((sy * src_width + sx) * 4) as usize;
            let di = ((dy * dst_width + dx) * 4) as usize;
            let Some(src_px) = src.get(si..si + 4) else {
                continue;
            };
            if src_px[3] == 0 {
                continue;
            }
            alpha_blend_rgba(&mut dst[di..di + 4], src_px);
        }
    }
}

fn read_text_record_string(bytes: &[u8], offset: usize, nls: Nls, multilingual: bool) -> Option<String> {
    let start = offset.checked_add(4)?;
    if multilingual {
        read_utf16_c_string(bytes, start)
    } else {
        read_c_string(bytes, start, nls)
    }
}

/// Read the NUL-terminated UTF-16LE string at `offset`.
///
/// Used by the FHD text layout, whose records are UTF-16LE; the original
/// single-byte layout would stop at the high byte of the first character.
fn read_utf16_c_string(bytes: &[u8], offset: usize) -> Option<String> {
    if offset >= bytes.len() {
        return None;
    }
    let mut units = Vec::new();
    let mut position = offset;
    while position + 2 <= bytes.len() {
        let unit = u16::from_le_bytes([bytes[position], bytes[position + 1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
        position += 2;
    }
    Some(String::from_utf16_lossy(&units))
}

/// Pick this language's field out of a multi-language text record.
///
/// The FHD pool joins one field per language with U+203B. Records without the
/// separator (UI format strings) are returned unchanged, and a table that only
/// holds one language never reaches the split, so the original releases render
/// byte-for-byte as before.
fn select_text_field(text: String, lang: TextLang, multilingual: bool) -> String {
    if !multilingual || !text.contains(TEXT_FIELD_SEPARATOR) {
        return text;
    }
    match text.split(TEXT_FIELD_SEPARATOR).nth(lang.field_index()) {
        Some(field) => field.to_owned(),
        None => text,
    }
}

/// Text face style a PAL font type names on a release that ships its own faces.
///
/// The FHD releases offer 思源宋体 and 思源黑体 on their SYSTEM screen, store the
/// picked row as the PAL font type the boot `set_language` applies, and tag the
/// style in the shipped file names (`SourceHanSerif-Medium`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceStyle {
    Serif,
    Sans,
}

impl FaceStyle {
    /// Style a PAL font type selects: the 宋体 row stores 0 and the 黑体 row 1.
    /// The original releases' types (including the bitmap type 4) name no face.
    fn from_font_type(font_type: u16) -> Option<Self> {
        match font_type {
            0 => Some(Self::Serif),
            1 => Some(Self::Sans),
            _ => None,
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::Serif => "serif",
            Self::Sans => "sans",
        }
    }
}

/// TrueType faces the release ships as loose files, in path order.
///
/// The caller's `--font-file` is the whole set; otherwise this is the game's own
/// `fonts` directory, which the original installs do not have.
fn game_font_faces(root: Option<&Path>, font_file: Option<&Path>) -> Vec<PathBuf> {
    if let Some(path) = font_file {
        return vec![path.to_path_buf()];
    }
    let Some(root) = root else {
        return Vec::new();
    };
    #[cfg(any(feature = "nfs", feature = "smb"))]
    use game_network::fs as font_fs;
    #[cfg(not(any(feature = "nfs", feature = "smb")))]
    use std::fs as font_fs;
    let mut candidates: Vec<PathBuf> = font_fs::read_dir(root.join("fonts"))
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    pal_asset::vfs::is_file(path)
                        && path
                            .extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case("ttf"))
                })
                .collect()
        })
        .unwrap_or_default();
    candidates.sort();
    candidates
}

/// Pick the face of `style` for `lang`, falling back to the language's face.
///
/// A release that ships both styles tags both the script and the style in the
/// file name, so the language's own tagged face wins and a style it does not
/// ship (or a single-face install) keeps the face `pick_face` selects.
fn pick_face_for(
    candidates: &[PathBuf],
    lang: TextLang,
    style: Option<FaceStyle>,
) -> Option<PathBuf> {
    let Some(style) = style else {
        return pick_face(candidates.to_vec(), lang);
    };
    let tagged = |path: &Path, tag: &str| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.to_ascii_lowercase().contains(tag))
    };
    let style = style.tag();
    let language = match lang {
        TextLang::Cn | TextLang::Tc => Some("cn"),
        TextLang::Jp => None,
    };
    if let Some(language) = language {
        if let Some(face) = candidates
            .iter()
            .find(|path| tagged(path, language) && tagged(path, style))
        {
            return Some(face.clone());
        }
    }
    if let Some(face) = candidates.iter().find(|path| tagged(path, style)) {
        return Some(face.clone());
    }
    pick_face(candidates.to_vec(), lang)
}

/// Pick the face tagged for `lang`, falling back to the first one by path.
///
/// A release that carries several scripts tags them in the file name — this one
/// ships `SourceHanSansCN-Medium` (Chinese) next to `SourceHanSans-Medium`
/// (Japanese) — and its replaced `PAL.dll` picks its text face by the same tag.
/// Traditional Chinese has no face of its own and uses the Chinese one. A
/// single-face install has nothing to choose between.
fn pick_face(candidates: Vec<PathBuf>, lang: TextLang) -> Option<PathBuf> {
    if matches!(lang, TextLang::Cn | TextLang::Tc) {
        let tagged = candidates.iter().find(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| stem.to_ascii_lowercase().contains("cn"))
        });
        if let Some(tagged) = tagged {
            return Some(tagged.clone());
        }
    }
    candidates.into_iter().next()
}

/// Text-pool record id (the 4-byte header read_text_record_string skips).
/// koikake.exe 0x42CBD0 uses it as the bit number in the read-text bitmap.
fn text_pool_record_id(text_value: i32, assets: &CoreAssets) -> Option<u32> {
    if text_value == 0x0FFF_FFFF || dynamic_string_index(text_value).is_some() {
        return None;
    }
    let offset: usize = text_value.try_into().ok()?;
    let bytes = assets.text_dat.bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_c_string(bytes: &[u8], offset: usize, nls: Nls) -> Option<String> {
    if offset >= bytes.len() {
        return None;
    }
    let end = bytes[offset..]
        .iter()
        .position(|b| *b == 0)
        .map(|rel| offset + rel)
        .unwrap_or(bytes.len());
    if end == offset {
        return Some(String::new());
    }
    nls.decode(&bytes[offset..end]).ok()
}

fn read_padded_c_string(bytes: &[u8], mut offset: usize, nls: Nls) -> Option<String> {
    let start = offset;
    while offset < bytes.len() && bytes[offset] == 0 && offset.saturating_sub(start) < 4 {
        offset += 1;
    }
    read_c_string(bytes, offset, nls).filter(|value| is_plausible_resource_name(value))
}

fn read_file_slot_string(
    bytes: &[u8],
    value: i32,
    nls: Nls,
    layout: FileDatLayout,
) -> Option<String> {
    let index: usize = value.try_into().ok()?;
    let offset = layout.base.checked_add(index.checked_mul(layout.stride)?)?;
    let text = if layout.utf16 {
        read_utf16_c_string(bytes, offset)?
    } else {
        read_c_string(bytes, offset, nls)?
    };
    is_plausible_resource_name(&text).then_some(text)
}

fn is_plausible_resource_name(value: &str) -> bool {
    if value.is_empty() || value.starts_with('$') || value.contains('*') || value.contains("__3I") {
        return false;
    }
    value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'#' | b'.'))
}

fn looks_like_media_or_resource_name(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.ends_with(".pgd")
        || lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".bmp")
        || lower.ends_with(".ogg")
        || lower.ends_with(".wav")
        || lower.ends_with(".wmv")
        || (value.is_ascii() && is_plausible_resource_name(value))
}

fn is_known_non_asset_probe_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    matches!(upper.as_str(), "ACK_R" | "_HIDE" | "ISC_VOLUMELABEL_NAME")
}

fn asset_source_label(source: &AssetSource) -> String {
    match source {
        AssetSource::Loose { path } => format!("loose:{}", path.display()),
        AssetSource::Pac { pac_path, .. } => format!("pac:{}", pac_path.display()),
    }
}

fn ini_filename_or_system(requested: &str) -> &str {
    if requested.to_ascii_lowercase().ends_with(".ini") {
        requested
    } else {
        "SYSTEM.INI"
    }
}

fn parse_file_table(bytes: &[u8], nls: Nls) -> anyhow::Result<ParsedFileTable> {
    let text = decode_file_table_text(bytes, nls);
    let compact = compact_csv_text(&text);
    let tokens = split_csv_tokens(&compact);
    let mut entries = Vec::with_capacity(tokens.len());
    let mut strings = BTreeMap::new();
    let mut string_offset = 0i32;
    for token in tokens {
        if token.is_empty() {
            continue;
        }
        if let Some(stripped) = token.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            let value = stripped.replace("\"\"", "\"");
            entries.push(0x8000_0000u32.wrapping_add(string_offset as u32) as i32);
            strings.insert(string_offset, value.clone());
            string_offset = string_offset
                .saturating_add(2)
                .saturating_add(value.as_bytes().len().min(i32::MAX as usize) as i32);
        } else {
            entries.push(parse_table_int(&token));
        }
    }
    Ok(ParsedFileTable { entries, strings })
}

fn decode_file_table_text(bytes: &[u8], nls: Nls) -> String {
    if let Ok(text) = nls.decode(bytes) {
        return text;
    }

    for fallback in [Nls::ShiftJis, Nls::Gbk, Nls::Utf8] {
        if fallback == nls {
            continue;
        }
        if let Ok(text) = fallback.decode(bytes) {
            log::debug!(
                "[trace-assets] file table decoded with fallback nls={} after {} failed",
                fallback.name(),
                nls.name()
            );
            return text;
        }
    }

    let (text, _encoding_used, had_errors) = nls.encoding().decode(bytes);
    if had_errors {
        log::debug!(
            "[trace-assets] file table decoded lossily with nls={}",
            nls.name()
        );
    }
    text.into_owned()
}

fn compact_csv_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_quote = false;
    while let Some(ch) = chars.next() {
        if !in_quote && ch == '/' && chars.peek() == Some(&'/') {
            while let Some(next) = chars.next() {
                if next == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }
        if ch == '"' {
            in_quote = !in_quote;
            out.push(ch);
            continue;
        }
        if !in_quote && ch.is_whitespace() {
            if ch == '\n' {
                out.push('\n');
            }
            continue;
        }
        out.push(ch);
    }
    out
}

fn split_csv_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    let mut in_quote = false;
    while let Some(ch) = chars.next() {
        if ch == '"' {
            current.push(ch);
            if in_quote && chars.peek() == Some(&'"') {
                current.push(chars.next().expect("peeked quote"));
            } else {
                in_quote = !in_quote;
            }
            continue;
        }
        if !in_quote && (ch == ',' || ch == '\n') {
            tokens.push(current.trim().to_owned());
            current.clear();
            continue;
        }
        current.push(ch);
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_owned());
    }
    tokens
}

fn parse_table_int(token: &str) -> i32 {
    if let Some(hex) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        i32::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        token.parse::<i32>().unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeTick {
    pub executed: usize,
    pub status: RuntimeStatus,
    /// A blocking wait task requested by the VM this tick. Engine creates the task
    /// and assigns the handle back to the runtime via set_wait_handle().
    pub wait_request: Option<WaitRequest>,
    /// Per-frame events: skipped extcalls, wait requests, unsupported ops.
    pub frame_events: Vec<FrameEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeStatus {
    NotBooted,
    Running {
        pc: u32,
    },
    /// Waiting on a 1-frame task created in TaskSystem (opcode 252).
    WaitFrame {
        pc: u32,
    },
    /// Waiting on an input-push task created in TaskSystem (opcode 253).
    WaitClick {
        pc: u32,
    },
    Halted {
        pc: u32,
    },
    UnsupportedCommand {
        pc: u32,
        opcode: u16,
        name: Option<String>,
    },
    UnsupportedExtCall {
        pc: u32,
        category: u16,
        index: u16,
        name: Option<String>,
        dst_slot: u32,
    },
    Faulted {
        pc: u32,
        message: String,
    },
}

impl RuntimeStatus {
    pub fn pc(&self) -> Option<u32> {
        match self {
            Self::NotBooted => None,
            Self::Running { pc }
            | Self::WaitFrame { pc }
            | Self::WaitClick { pc }
            | Self::Halted { pc }
            | Self::UnsupportedCommand { pc, .. }
            | Self::UnsupportedExtCall { pc, .. }
            | Self::Faulted { pc, .. } => Some(*pc),
        }
    }

    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            Self::WaitFrame { .. }
                | Self::WaitClick { .. }
                | Self::Halted { .. }
                | Self::UnsupportedCommand { .. }
                | Self::UnsupportedExtCall { .. }
                | Self::Faulted { .. }
        )
    }
}

impl fmt::Display for RuntimeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotBooted => write!(f, "not booted"),
            Self::Running { pc } => write!(f, "running pc=0x{pc:08X}"),
            Self::WaitFrame { pc } => write!(f, "wait frame (task) pc=0x{pc:08X}"),
            Self::WaitClick { pc } => write!(f, "wait click (task) pc=0x{pc:08X}"),
            Self::Halted { pc } => write!(f, "halted pc=0x{pc:08X}"),
            Self::UnsupportedCommand { pc, opcode, name } => match name {
                Some(name) => write!(f, "unsupported command pc=0x{pc:08X} opcode={}({})", opcode, name),
                None => write!(f, "unsupported command pc=0x{pc:08X} opcode={}", opcode),
            },
            Self::UnsupportedExtCall { pc, category, index, name, dst_slot } => match name {
                Some(name) => write!(
                    f,
                    "unsupported extcall pc=0x{pc:08X} ext_{category:04X}_{index:04X}.{} dst_slot={}",
                    name, dst_slot
                ),
                None => write!(f, "unsupported extcall pc=0x{pc:08X} ext_{category:04X}_{index:04X} dst_slot={}", dst_slot),
            },
            Self::Faulted { pc, message } => write!(f, "faulted pc=0x{pc:08X}: {message}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    ScriptParse {
        source: String,
    },
    ReadOutOfBounds {
        pc: u32,
        len: usize,
    },
    InvalidInstructionWord {
        pc: u32,
        word: u32,
    },
    PointResolve {
        point_id: u32,
        source: String,
    },
    DivideByZero {
        pc: u32,
    },
    ReturnStackUnderflow {
        pc: u32,
    },
    StackUnderflow {
        pc: u32,
    },
    StackOverflow {
        pc: u32,
        limit: usize,
    },
    ArgumentStackUnderflow {
        pc: u32,
    },
    NegativeCount {
        pc: u32,
        count: i32,
    },
    VariableOutOfRange {
        slot: usize,
    },
    StackSlotOutOfRange {
        slot: usize,
        depth: usize,
    },
    ArgumentSlotOutOfRange {
        slot: usize,
        depth: usize,
    },
    MemDatOutOfRange {
        offset: usize,
        len: usize,
    },
    MemIndexOutOfRange {
        index: usize,
        len: usize,
        kind: &'static str,
    },
    UnsupportedOperandRead {
        raw: u32,
        kind: String,
    },
    UnsupportedOperandWrite {
        raw: u32,
        kind: String,
    },
    UnsupportedInternalOpcode {
        pc: u32,
        opcode: u16,
    },
    ArithmeticOverflow {
        pc: u32,
    },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScriptParse { source } => write!(f, "failed to parse Script.src: {source}"),
            Self::ReadOutOfBounds { pc, len } => {
                write!(
                    f,
                    "script read out of bounds at pc=0x{pc:08X}, len=0x{len:X}"
                )
            }
            Self::InvalidInstructionWord { pc, word } => {
                write!(f, "invalid instruction word at pc=0x{pc:08X}: 0x{word:08X}")
            }
            Self::PointResolve { point_id, source } => {
                write!(f, "failed to resolve point id {}: {}", point_id, source)
            }
            Self::DivideByZero { pc } => write!(f, "script divide by zero at pc=0x{pc:08X}"),
            Self::ReturnStackUnderflow { pc } => {
                write!(f, "script return stack underflow at pc=0x{pc:08X}")
            }
            Self::StackUnderflow { pc } => write!(f, "script stack underflow at pc=0x{pc:08X}"),
            Self::StackOverflow { pc, limit } => {
                write!(f, "script stack overflow at pc=0x{pc:08X}, limit={limit}")
            }
            Self::ArgumentStackUnderflow { pc } => {
                write!(f, "script argument stack underflow at pc=0x{pc:08X}")
            }
            Self::NegativeCount { pc, count } => {
                write!(f, "negative count {} at pc=0x{pc:08X}", count)
            }
            Self::VariableOutOfRange { slot } => {
                write!(f, "variable slot {} is out of range", slot)
            }
            Self::StackSlotOutOfRange { slot, depth } => {
                write!(f, "stack slot {} is out of range for depth {}", slot, depth)
            }
            Self::ArgumentSlotOutOfRange { slot, depth } => {
                write!(
                    f,
                    "argument stack reverse slot {} is out of range for depth {}",
                    slot, depth
                )
            }
            Self::MemDatOutOfRange { offset, len } => {
                write!(
                    f,
                    "Mem.dat read out of range at offset=0x{offset:X}, len=0x{len:X}"
                )
            }
            Self::MemIndexOutOfRange { index, len, kind } => {
                write!(f, "{kind} index {index} out of range (len={len})")
            }
            Self::UnsupportedOperandRead { raw, kind } => {
                write!(f, "unsupported operand read kind={} raw=0x{raw:08X}", kind)
            }
            Self::UnsupportedOperandWrite { raw, kind } => {
                write!(f, "unsupported operand write kind={} raw=0x{raw:08X}", kind)
            }
            Self::UnsupportedInternalOpcode { pc, opcode } => {
                write!(
                    f,
                    "internal opcode {} is unsupported at pc=0x{pc:08X}",
                    opcode
                )
            }
            Self::ArithmeticOverflow { pc } => {
                write!(f, "arithmetic overflow near pc=0x{pc:08X}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

fn bool_to_i32(value: bool) -> i32 {
    if value {
        1
    } else {
        0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StepResult {
    Continue,
    Blocked,
    BlockedWithWait(WaitRequest),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioConfig;

    #[test]
    fn pal_cell_indices_survive_vm_extcall_dispatch() {
        let (assets, points) = empty_core_assets(false);
        let mut vm = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let handle = sprites
            .create_rgba_sprite(
                64, 32, vec![255; 64 * 32 * 4], PalVec3::default(), 0, "ABI fixture",
            )
            .unwrap();
        sprites.set_offset_rect(handle, 16, 16);
        vm.game_sprites.insert(0, handle);
        vm.stack = vec![991, -1, 65_536, 0];
        let outcome = vm.dispatch_extcall(
            3, 15, &[], &assets, &points, None, Some(&mut sprites), None, None, None,
        );
        assert!(matches!(outcome, ExtCallOutcome::Value(1)));
        assert_eq!(vm.stack, vec![991]);
        assert_eq!(
            sprites.get(handle).unwrap().source_rect,
            PalRect::new(48, 16, 64, 32)
        );
    }

    #[test]
    fn chinese_text_language_selects_the_face_tagged_for_it() {
        let faces = |names: &[&str]| names.iter().map(PathBuf::from).collect::<Vec<_>>();
        let shipped = faces(&[
            "fonts/SourceHanSans-Medium.ttf",
            "fonts/SourceHanSansCN-Medium.ttf",
            "fonts/SourceHanSerif-Medium.ttf",
        ]);
        // Traditional Chinese has no face of its own, so it shares the Chinese one.
        for lang in [TextLang::Cn, TextLang::Tc] {
            assert_eq!(
                pick_face(shipped.clone(), lang),
                Some(PathBuf::from("fonts/SourceHanSansCN-Medium.ttf"))
            );
        }
        assert_eq!(
            pick_face(shipped, TextLang::Jp),
            Some(PathBuf::from("fonts/SourceHanSans-Medium.ttf"))
        );
    }

    #[test]
    fn a_fonts_directory_without_a_tagged_face_keeps_the_first_one() {
        let only = vec![PathBuf::from("fonts/SomeSans.ttf")];
        for lang in [TextLang::Jp, TextLang::Cn, TextLang::Tc] {
            assert_eq!(
                pick_face(only.clone(), lang),
                Some(PathBuf::from("fonts/SomeSans.ttf"))
            );
        }
        assert_eq!(pick_face(Vec::new(), TextLang::Cn), None);
    }

    #[test]
    fn font_style_selects_the_release_face_of_that_style() {
        let shipped = [
            PathBuf::from("fonts/SourceHanSans-Medium.ttf"),
            PathBuf::from("fonts/SourceHanSansCN-Medium.ttf"),
            PathBuf::from("fonts/SourceHanSerif-Medium.ttf"),
        ];
        let picked = |lang, style| pick_face_for(&shipped, lang, Some(style));
        // Each language keeps its own script: the Chinese 黑体 row reaches the
        // Chinese sans face, the Japanese one the Japanese sans face.
        assert_eq!(picked(TextLang::Cn, FaceStyle::Sans), Some(shipped[1].clone()));
        assert_eq!(picked(TextLang::Tc, FaceStyle::Sans), Some(shipped[1].clone()));
        assert_eq!(picked(TextLang::Jp, FaceStyle::Sans), Some(shipped[0].clone()));
        // Both languages share the one serif face.
        for lang in [TextLang::Jp, TextLang::Cn, TextLang::Tc] {
            assert_eq!(picked(lang, FaceStyle::Serif), Some(shipped[2].clone()));
        }
        // No style requested keeps the language's face, as does a style the
        // release does not ship.
        assert_eq!(
            pick_face_for(&shipped, TextLang::Cn, None),
            Some(shipped[1].clone())
        );
        let sans_only = [PathBuf::from("fonts/SourceHanSansCN-Medium.ttf")];
        assert_eq!(
            pick_face_for(&sans_only, TextLang::Cn, Some(FaceStyle::Serif)),
            Some(sans_only[0].clone())
        );
    }

    #[test]
    fn font_type_names_the_face_style_of_a_localized_release() {
        assert_eq!(FaceStyle::from_font_type(0), Some(FaceStyle::Serif));
        assert_eq!(FaceStyle::from_font_type(1), Some(FaceStyle::Sans));
        // The bitmap type of the original releases and the remaining types
        // name no shipped face.
        for font_type in [2, 3, 4, 17] {
            assert_eq!(FaceStyle::from_font_type(font_type), None);
        }
    }

    /// The SYSTEM screen's 文本字体 row stores a PAL font type; a release that
    /// ships those faces switches the text face when that type is applied.
    #[test]
    fn font_type_switch_selects_the_release_face_of_that_style() {
        let root = std::env::temp_dir().join("sena_rs_face_style_test");
        let fonts = root.join("fonts");
        std::fs::create_dir_all(&fonts).unwrap();
        let shipped = [
            "SourceHanSans-Medium.ttf",
            "SourceHanSansCN-Medium.ttf",
            "SourceHanSerif-Medium.ttf",
        ];
        for name in shipped {
            std::fs::write(fonts.join(name), include_bytes!("default.ttf")).unwrap();
        }
        let (assets, points) = empty_core_assets(true);
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.text_lang = TextLang::Cn;
        runtime.font_face_candidates = game_font_faces(Some(&root), None);
        assert_eq!(runtime.font_face_candidates.len(), 3);
        // Boot picked the Chinese face; the row then stores 宋体 (type 0).
        runtime.loaded_face_path = Some(fonts.join(shipped[1]));
        runtime.style_faces_enabled = true;
        runtime.stack.push(0);
        let outcome =
            runtime.dispatch_extcall(9, 17, &[], &assets, &points, None, None, None, None, None);
        assert!(matches!(outcome, ExtCallOutcome::Value(1)));
        assert!(runtime.stack.is_empty(), "the font type argument is consumed");
        assert_eq!(runtime.font_state.font_type(), 0);
        assert_eq!(runtime.loaded_face_path, Some(fonts.join(shipped[2])));
        assert!(runtime.text_state.dirty, "the parked line re-typesets");
        // 黑体 (type 1) goes back to the Chinese sans face.
        runtime.stack.push(1);
        runtime.dispatch_extcall(9, 17, &[], &assets, &points, None, None, None, None, None);
        assert_eq!(runtime.loaded_face_path, Some(fonts.join(shipped[1])));
        std::fs::remove_dir_all(&root).ok();
    }

    /// The original releases have one face at most, so a font type leaves it
    /// alone even where a caller named one.
    #[test]
    fn font_type_leaves_a_single_face_release_alone() {
        let root = std::env::temp_dir().join("sena_rs_face_style_single_test");
        let fonts = root.join("fonts");
        std::fs::create_dir_all(&fonts).unwrap();
        let face = fonts.join("SomeSansCN.ttf");
        std::fs::write(&face, include_bytes!("default.ttf")).unwrap();
        let (assets, points) = empty_core_assets(false);
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.text_lang = TextLang::Cn;
        runtime.font_face_candidates = vec![face.clone()];
        runtime.loaded_face_path = Some(face.clone());
        runtime.stack.push(0);
        let dirty = runtime.text_state.dirty;
        runtime.dispatch_extcall(9, 17, &[], &assets, &points, None, None, None, None, None);
        assert_eq!(runtime.loaded_face_path, Some(face));
        assert_eq!(runtime.text_state.dirty, dirty);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn script_priority_cursor_layers_popup_over_existing_buttons() {
        let mut sprites = SpriteSystem::new();
        let menu_button = sprites
            .create_rgba_sprite(
                2,
                2,
                vec![255; 16],
                PalVec3::new(0, 0, 0),
                button_render_priority(127, -134),
                "settings button",
            )
            .unwrap();
        let frame = sprites
            .create_rgba_sprite(
                2,
                2,
                vec![255; 16],
                PalVec3::new(0, 0, 0),
                game_sprite_priority(127, -268),
                "popup frame",
            )
            .unwrap();
        let confirm_button = sprites
            .create_rgba_sprite(
                2,
                2,
                vec![255; 16],
                PalVec3::new(0, 0, 0),
                button_render_priority(0, -268),
                "confirm button",
            )
            .unwrap();
        let order = sprites
            .commands()
            .into_iter()
            .filter_map(|command| match command {
                crate::scene::DrawCommand::Sprite(draw) => Some(draw.texture_id.0),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            order,
            [
                sprites.get(menu_button).unwrap().surface.0,
                sprites.get(frame).unwrap().surface.0,
                sprites.get(confirm_button).unwrap().surface.0
            ]
        );
    }

    #[test]
    fn adv_text_layer_stays_under_buttons_and_over_scene_sprites() {
        for cursor in [0, 134, 5000] {
            let text = adv_text_render_priority(cursor);
            assert!(
                text > game_sprite_priority(80, cursor),
                "text window must cover low-slot scene sprites (cursor={cursor})"
            );
            assert!(
                text < button_render_priority(0, cursor),
                "button layer must cover the text window (cursor={cursor})"
            );
        }
        // Negative popup lanes bring their own content forward; the text
        // window must not follow them up or a mid-reveal recompose would
        // paint it over the popup.
        assert_eq!(adv_text_render_priority(-268), adv_text_render_priority(0));
    }

    #[test]
    fn history_text_follows_popup_lane_between_sprites_and_buttons() {
        // Koikake opens the log screen at cursor -134: LOG_BASE lands on
        // sprite slot 64 and the screen buttons start at index 20.
        let cursor = -134;
        let history = history_text_render_priority(cursor);
        assert!(
            history > game_sprite_priority(64, cursor),
            "backlog text must cover the log screen background"
        );
        assert!(
            history < button_render_priority(20, cursor),
            "log screen buttons must cover the backlog text"
        );
        // Outside a popup lane the backlog stays just above the text window.
        assert_eq!(
            history_text_render_priority(0),
            adv_text_render_priority(0) + 1
        );
    }

    #[test]
    fn inline_quick_buttons_follow_toolbar_alpha() {
        let mut runtime = ScriptRuntime::boot(0, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let text = sprites
            .create_rgba_sprite(2, 2, vec![255; 16], PalVec3::new(0, 0, 0), 90, "adv:text")
            .unwrap();
        runtime.text_state.initialized = true;
        runtime.text_state.visible = true;
        runtime.text_state.sprite = Some(text);

        let entry = |handle, name: &str, alpha| GameButtonEntry {
            registration_order: 0,
            native_flags: 1,
            hover_callbacks: [None; 2],
            handle,
            name: name.to_owned(),
            visible: true,
            enabled: true,
            locked: false,
            toggle: 0,
            alpha,
            slider_offset: 0,
            base_position: [0; 2],
            slider_position: [0; 2],
            hit_rect: None,
            gosub_point: None,
            anim_resource: None,
            anim_play_flag: 0,
        };
        let peer = sprites
            .create_rgba_sprite(
                2,
                2,
                vec![255; 16],
                PalVec3::new(0, 0, 0),
                BUTTON_RENDER_PRIORITY,
                "save",
            )
            .unwrap();
        let quick = sprites
            .create_rgba_sprite(
                2,
                2,
                vec![255; 16],
                PalVec3::new(0, 0, 0),
                BUTTON_RENDER_PRIORITY,
                "quick save",
            )
            .unwrap();
        sprites.set_alpha(quick, 0);
        runtime
            .game_buttons
            .insert((0, 5), entry(peer, "MAIN_BTN_SAVE", 255));
        runtime
            .game_buttons
            .insert((0, 7), entry(quick, "MAIN_BTN_QSAVE", 0));

        runtime.sync_adv_button_chrome_visibility(&mut sprites);
        assert!(sprites.get(quick).unwrap().visible);
        assert_eq!(sprites.get(quick).unwrap().color.alpha(), 255);

        sprites.set_alpha(peer, 0);
        runtime.game_buttons.get_mut(&(0, 5)).unwrap().alpha = 0;
        runtime.sync_adv_button_chrome_visibility(&mut sprites);
        assert!(!sprites.get(quick).unwrap().visible);
        assert_eq!(sprites.get(quick).unwrap().color.alpha(), 0);

        sprites.set_alpha(peer, 128);
        runtime.game_buttons.get_mut(&(0, 5)).unwrap().alpha = 128;
        runtime.sync_adv_button_chrome_visibility(&mut sprites);
        assert!(sprites.get(quick).unwrap().visible);
        assert_eq!(sprites.get(quick).unwrap().color.alpha(), 128);
    }

    #[test]
    fn pgd_default_offsets_use_native_stage_coordinates() {
        let (x, y, z) =
            native_place_sprite(5, 0xFFFF, 0xFFFF, 70, 464, 144, 408, 288, 1280, 720);
        assert_eq!((x, y, z), (612.0, 432.0, 70.0));
        // The FHD releases author the same artwork and its default offsets in
        // their own 1920x1080 stage, so there the offsets are already native.
        let (x, y, z) =
            native_place_sprite(5, 0xFFFF, 0xFFFF, 70, 634, 208, 627, 370, 1920, 1080);
        assert_eq!((x, y, z), (627.0, 370.0, 70.0));
    }

    #[test]
    fn button_position_query_writes_requested_temporary_slots() {
        let mut runtime = ScriptRuntime::boot(0, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let mut desc = SpriteDesc::new(SceneTextureId(1), 200, 120);
        desc.position = PalVec3::new(448, 376, 0);
        let handle = sprites.create(desc);
        runtime.game_buttons.insert(
            (7, 0),
            GameButtonEntry {
                registration_order: 0,
            native_flags: 1,
            hover_callbacks: [None; 2],
                handle,
                name: "button".to_owned(),
                visible: true,
                enabled: true,
                locked: false,
                toggle: 0,
                alpha: 255,
                slider_offset: 0,
            base_position: [0; 2],
            slider_position: [0; 2],
                hit_rect: None,
                gosub_point: None,
                anim_resource: None,
                anim_play_flag: 0,
            },
        );
        runtime.argument_base = 64;
        runtime.write_temp_mem_absolute(0, i32::MIN + 1683);
        runtime.stack = vec![99, 1, 0, 0, 7];
        assert!(matches!(
            runtime.ext_btn_get_pos(Some(&sprites)),
            ExtCallOutcome::Value(1)
        ));
        assert_eq!(runtime.temp_mem[0], 448);
        assert_eq!(runtime.temp_mem[1], 376);
        assert_eq!(runtime.stack, [99]);
    }

    #[test]
    fn slider_set_maps_value_percent_to_knob_offset() {
        let mut runtime = ScriptRuntime::boot(0, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let entry = |handle, name: &str| GameButtonEntry {
            registration_order: 0,
            native_flags: 1,
            hover_callbacks: [None; 2],
            handle,
            name: name.to_owned(),
            visible: true,
            enabled: true,
            locked: false,
            toggle: 0,
            alpha: 255,
            slider_offset: 0,
            base_position: [0; 2],
            slider_position: [0; 2],
            hit_rect: None,
            gosub_point: None,
            anim_resource: None,
            anim_play_flag: 0,
        };
        let mut base_desc = SpriteDesc::new(SceneTextureId(1), 240, 32);
        base_desc.position = PalVec3::new(72, 153, 0);
        let base_handle = sprites.create(base_desc);
        let mut knob_desc = SpriteDesc::new(SceneTextureId(2), 19, 16);
        knob_desc.position = PalVec3::new(72, 161, 0);
        let knob_handle = sprites.create(knob_desc);
        runtime
            .game_buttons
            .insert((6, 0), entry(base_handle, "SOUND_SLIDE_BASE_UNIT"));
        runtime
            .game_buttons
            .insert((6, 20), entry(knob_handle, "SOUND_SLIDE_ICON_UNIT"));
        // Push order: value, axis, travel, index, group (group pops first).
        runtime.stack = vec![75, 0, 208, 20, 6];
        assert!(matches!(
            runtime.ext_btn_slider_set(Some(&mut sprites)),
            ExtCallOutcome::Value(1)
        ));
        assert_eq!(runtime.game_buttons[&(6, 20)].slider_offset, 156);
        assert_eq!(sprites.get(knob_handle).unwrap().position.x, 228.0);

        // Values above 100% pin the knob at the travel end (BGM passes 250).
        runtime.stack = vec![250, 0, 208, 20, 6];
        assert!(matches!(
            runtime.ext_btn_slider_set(Some(&mut sprites)),
            ExtCallOutcome::Value(1)
        ));
        assert_eq!(runtime.game_buttons[&(6, 20)].slider_offset, 208);
        assert_eq!(sprites.get(knob_handle).unwrap().position.x, 280.0);
    }

    #[test]
    fn slider_on_check_reads_the_group_end_latch() {
        let mut runtime = ScriptRuntime::boot(0, ScriptRuntimeConfig::default());
        runtime.stack = vec![20, 6];
        assert!(matches!(
            runtime.ext_btn_on_check(None, None),
            ExtCallOutcome::Value(0)
        ));
        runtime.button_groups.get_mut(&6).unwrap().slider_ended = true;
        runtime.stack = vec![20, 6];
        assert!(matches!(
            runtime.ext_btn_on_check(None, None),
            ExtCallOutcome::Value(-1)
        ));
    }

    #[test]
    fn sp_get_width_resolves_packed_button_reference() {
        let mut runtime = ScriptRuntime::boot(0, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let desc = SpriteDesc::new(SceneTextureId(1), 240, 32);
        let handle = sprites.create(desc);
        runtime.game_buttons.insert(
            (6, 0),
            GameButtonEntry {
                registration_order: 0,
            native_flags: 1,
            hover_callbacks: [None; 2],
                handle,
                name: "SOUND_SLIDE_BASE_UNIT".to_owned(),
                visible: true,
                enabled: true,
                locked: false,
                toggle: 0,
                alpha: 255,
                slider_offset: 0,
            base_position: [0; 2],
            slider_position: [0; 2],
                hit_rect: None,
                gosub_point: None,
                anim_resource: None,
                anim_play_flag: 0,
            },
        );
        runtime.stack = vec![0x0260_0000];
        assert!(matches!(
            runtime.ext_sp_get_dimension(Some(&mut sprites), true),
            ExtCallOutcome::Value(240)
        ));
        runtime.stack = vec![0x0260_0000];
        assert!(matches!(
            runtime.ext_sp_get_dimension(Some(&mut sprites), false),
            ExtCallOutcome::Value(32)
        ));
    }

    #[test]
    fn extended_pal_clock_advances_without_consuming_script_arguments() {
        let empty_asset = |name: &str| LoadedAsset {
            name: name.to_owned(),
            bytes: Vec::new(),
            source: AssetSource::Loose {
                path: PathBuf::from(name),
            },
        };
        let points = PointTable::parse(&[]).unwrap();
        let assets = CoreAssets {
            script: empty_asset("Script.src"),
            file_dat: empty_asset("File.dat"),
            text_dat: empty_asset("Text.dat"),
            mem_dat: empty_asset("Mem.dat"),
            point_dat: empty_asset("Point.dat"),
            graphic_dat: None,
            text_dat_multilingual: false,
            file_dat_layout: FileDatLayout::default(),
            script_check_value: 0,
            script_entry_pc: 12,
            extended_softpal: false,
            native_build: Default::default(),
            registered_softpal: false,
            point_table: points.clone(),
            graphic_index: None,
        };
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.stack.push(42);
        runtime.set_pal_time(1_000);
        let first =
            runtime.dispatch_extcall(18, 121, &[], &assets, &points, None, None, None, None, None);
        runtime.set_pal_time(1_088);
        let second =
            runtime.dispatch_extcall(18, 121, &[], &assets, &points, None, None, None, None, None);
        assert!(matches!(first, ExtCallOutcome::Value(1_000)));
        assert!(matches!(second, ExtCallOutcome::Value(1_088)));
        assert_eq!(runtime.stack, [42]);
    }

    #[test]
    fn bgv_volume_dispatch_preserves_cached_level_through_mute() {
        let (assets, points) = empty_core_assets(false);
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        let mut audio = AudioSystem::new(crate::audio::AudioConfig { enabled: false }).unwrap();
        runtime.stack.push(0x123456);
        for (category, index, argument, expected_gain, expected_cached) in [
            (13, 21, 37, 3700, 37),
            (13, 20, 1, 0, 37),
            (13, 21, 61, 0, 61),
            (13, 20, 0, 6100, 61),
            (2, 69, 23, 2300, 23),
            (2, 68, 1, 0, 23),
            (2, 68, 0, 2300, 23),
        ] {
            runtime.stack.push(argument);
            assert_eq!(runtime.dispatch_extcall(category, index, &[], &assets, &points,
                None, None, None, Some(&mut audio), None), ExtCallOutcome::Value(1));
            assert_eq!(runtime.stack, vec![0x123456]);
            assert_eq!(audio.group_volume(PalSoundGroup::GROUP2).raw(), expected_gain);
            assert_eq!(audio.group_volume(PalSoundGroup::GROUP1), PalVolume::MAX);
            assert_eq!(runtime.dispatch_extcall(13, 22, &[], &assets, &points,
                None, None, None, Some(&mut audio), None), ExtCallOutcome::Value(expected_cached));
            assert_eq!(runtime.stack, vec![0x123456]);
        }
    }

    #[test]
    fn primary_waits_consume_native_operands_and_resume_after_four_bytes() {
        for (opcode, stack, expected) in [
            (252u32, vec![777, 0, 250], Some(WaitRequest::Time(250))),
            (252, vec![777, 0, -1], Some(WaitRequest::Frame(-1))),
            (252, vec![777, 1, -1], Some(WaitRequest::Click)),
            (253, vec![777, 120], Some(WaitRequest::ClickOrTime(120))),
            (253, vec![777, -1], None),
        ] {
            let (mut assets, points) = empty_core_assets(false);
            assets.script.bytes = [b"Sv20".as_slice(), &0u32.to_le_bytes(), &12u32.to_le_bytes(), &(0x10000 | opcode).to_le_bytes()].concat();
            let script = assets.script_image().unwrap();
            let mut vm = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
            vm.stack = stack;
            let result = vm.step(&script, &points, &[], &assets, None, None, None, None, None).unwrap();
            match expected {
                Some(request) => assert!(matches!(result, StepResult::BlockedWithWait(actual) if actual == request)),
                None => assert!(matches!(result, StepResult::Continue)),
            }
            assert_eq!(vm.pc, 16);
            assert_eq!(vm.stack, [777]);
        }
    }

    #[test]
    fn direct_wait_sync_dispatch_has_no_destination_and_uses_four_byte_continuation() {
        let (mut assets, _) = empty_core_assets(false);
        assets.script.bytes = [b"Sv20".as_slice(), &0u32.to_le_bytes(), &12u32.to_le_bytes(),
            &(0x10000u32 | 254).to_le_bytes(), &(0x10000u32 | 257).to_le_bytes(),
            &(0x10000u32 | 21).to_le_bytes()].concat();
        let config = ScriptRuntimeConfig::default();
        let mut vm = ScriptRuntime::boot(12, config.clone());
        vm.vars[0] = 777;
        vm.set_pal_time(1234);
        let tick = vm.run_frame(&assets, &config).unwrap();
        assert_eq!(tick.wait_request, Some(WaitRequest::Frame(1)));
        assert_eq!(vm.wait_sync_begin_ms, 1234);
        assert_eq!(vm.vars[0], 777);
        assert_eq!(vm.pc, 20);
        assert_eq!(vm.extcall_instruction_size, 4);
        vm.resolve_pending_wait();
        vm.run_frame(&assets, &config).unwrap();
        assert!(matches!(vm.status, RuntimeStatus::Halted { pc: 24 }));
    }

    #[test]
    fn global_raster_call_and_stop_preserve_foreground_ui() {
        let (assets, points) = empty_core_assets(false);
        let mut vm = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let background: Vec<u8> = (0..32*32).flat_map(|i| [(i%32*8) as u8, (i/32*8) as u8, 0, 255]).collect();
        sprites.create_rgba_sprite(32, 32, background, PalVec3::default(), -5000, "background").unwrap();
        let ui = sprites.create_rgba_sprite(8, 8, [255,0,0,255].repeat(64), PalVec3 { x: 12.0, y: 12.0, z: 0.0 }, -100, "ui").unwrap();
        vm.stack = vec![777, 100, 20, 40, 10];
        assert!(matches!(vm.dispatch_extcall(16, 5, &[], &assets, &points, None, Some(&mut sprites), None, None, None),
            ExtCallOutcome::Wait { request: WaitRequest::Frame(1), .. }));
        sprites.tick_fx_effects(0);
        sprites.tick_fx_effects(75);
        let mut scene = FrameScene::boot().with_logical_size(32, 32);
        scene.commands = sprites.commands();
        scene.textures = sprites.textures().collect();
        scene.raster_stage = sprites.global_raster_stage();
        assert_eq!(scene.raster_stage.unwrap().priority, game_sprite_priority(10, 0));
        let actual = crate::scene::rasterize_scene_rgba(&scene);
        scene.raster_stage = None;
        let plain = crate::scene::rasterize_scene_rgba(&scene);
        assert_ne!(actual, plain);
        assert_eq!(&actual[(16*32+16)*4..(16*32+16)*4+4], &[255,0,0,255]);
        assert!(sprites.get(ui).unwrap().fx_effect.is_none());
        vm.stack.extend([25, 0x10]);
        vm.dispatch_extcall(16, 0, &[], &assets, &points, None, Some(&mut sprites), None, None, None);
        sprites.tick_fx_effects(99);
        assert!(sprites.global_raster_stage().is_some());
        sprites.tick_fx_effects(100);
        assert!(sprites.global_raster_stage().is_none());
        vm.finish_global_fx_stop(&mut sprites);
        assert!(sprites.start_global_raster(0, Default::default(), 0));
        vm.release_scene_for_load(&mut sprites);
        assert!(sprites.start_global_raster(0, Default::default(), 0));
        assert_eq!(vm.stack, [777]);
    }

    #[test]
    fn raster_extcalls_reach_scene_and_wait_for_the_stop_phase() {
        let (assets, points) = empty_core_assets(false);
        let mut vm = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let handle = sprites.create_rgba_sprite(4, 4, vec![255; 64], PalVec3::default(), 0, "fx-test").unwrap();
        vm.game_sprites.insert(3, handle);
        vm.stack = vec![777, 200, 20, 40, 3];
        let outcome = vm.dispatch_extcall(22, 7, &[], &assets, &points, None, Some(&mut sprites), None, None, None);
        assert!(matches!(outcome, ExtCallOutcome::Value(1)));
        assert_eq!(vm.stack, [777]);
        sprites.tick_fx_effects(100);
        let crate::scene::DrawCommand::Sprite(draw) = &sprites.commands()[0] else { panic!() };
        assert!((draw.raster_fx.unwrap().undulate_x - 0.2).abs() < 0.00001);
        vm.stack.extend([50, 3]);
        vm.dispatch_fx_ext(5, Some(&mut sprites));
        assert_eq!(sprites.sprite_fx_effect_state(handle), 2);
        vm.stack.push(3);
        assert!(matches!(vm.dispatch_fx_ext(6, Some(&mut sprites)), ExtCallOutcome::Wait { request: WaitRequest::SpriteFx(h), .. } if h == handle));
        let mut tasks = TaskSystem::new();
        let wait = tasks.create_wait_sprite_fx(handle).unwrap();
        let input = PalInputState::new();
        sprites.tick_fx_effects(149);
        tasks.process(&mut sprites, &input, false, false);
        assert!(tasks.is_alive(wait));
        sprites.tick_fx_effects(150);
        tasks.process(&mut sprites, &input, false, false);
        tasks.process(&mut sprites, &input, false, false);
        assert!(!tasks.is_alive(wait));
        assert_eq!(sprites.sprite_fx_effect_state(handle), 0);
        assert_eq!(vm.stack, [777]);
    }

    /// Minimal asset set for extcalls that only consult runtime state.
    pub(super) fn empty_core_assets(multilingual: bool) -> (CoreAssets, PointTable) {
        let empty_asset = |name: &str| LoadedAsset {
            name: name.to_owned(),
            bytes: Vec::new(),
            source: AssetSource::Loose {
                path: PathBuf::from(name),
            },
        };
        let points = PointTable::parse(&[]).unwrap();
        let assets = CoreAssets {
            script: empty_asset("Script.src"),
            file_dat: empty_asset("File.dat"),
            text_dat: empty_asset("Text.dat"),
            mem_dat: empty_asset("Mem.dat"),
            point_dat: empty_asset("Point.dat"),
            graphic_dat: None,
            text_dat_multilingual: multilingual,
            file_dat_layout: FileDatLayout::default(),
            script_check_value: 0,
            script_entry_pc: 12,
            extended_softpal: false,
            native_build: Default::default(),
            registered_softpal: false,
            point_table: points.clone(),
            graphic_index: None,
        };
        (assets, points)
    }

    #[test]
    fn voice_mix_uses_independent_native_lanes_and_replaces_the_whole_adv_group() {
        let root = std::env::temp_dir().join(format!("sena_voice_mix_{}_{}",
            std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&root).unwrap();
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF"); wav.extend_from_slice(&40u32.to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt "); wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&8000u32.to_le_bytes()); wav.extend_from_slice(&16000u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes()); wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data"); wav.extend_from_slice(&4u32.to_le_bytes());
        wav.extend_from_slice(&[0, 0, 0, 0]);
        let mut mix = vec![0u8; 64 * 9];
        for lane in 0..9 {
            let name = format!("speaker{lane}_line");
            for (i, unit) in name.encode_utf16().enumerate() {
                mix[lane * 64 + i * 2..lane * 64 + i * 2 + 2].copy_from_slice(&unit.to_le_bytes());
            }
            std::fs::write(root.join(format!("{name}.wav")), &wav).unwrap();
        }
        std::fs::write(root.join("chorus.mix"), mix).unwrap();
        // MIX lookup must win over a colliding ordinary clip.
        std::fs::write(root.join("chorus.wav"), &wav).unwrap();
        let mut resources = ResourceManager::new(&root, Nls::ShiftJis);
        let mut audio = AudioSystem::new(AudioConfig { enabled: false }).unwrap();
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.voice_unit_headers = Some(vec!["speaker0".into(), "speaker1".into()]);
        runtime.voice_ex_volume_percent.insert(0, 25);
        runtime.voice_ex_enabled.insert(1, false);
        runtime.text_state.voice_autopan_enabled = true;
        runtime.text_state.voice_autopan_limit = 3000;
        for (slot, target) in [(0, 0), (1, 20)] {
            runtime.text_state.voice_autopan_entries.insert(slot, VoiceAutopanEntry {
                target, mode: 1, name_value: 0, name: format!("speaker{slot}"),
            });
        }
        assert!(runtime.load_named_audio(13, 8, PalSoundGroup::GROUP1, "speaker8_line",
            0, 100, false, None, Some(&mut resources), Some(&mut audio)));
        let independent = runtime.game_audio[&(13, 8)];
        assert!(runtime.load_voice_audio(7, PalSoundGroup::GROUP1, "chorus", 0, 100,
            true, 0, Some(&mut resources), Some(&mut audio)));
        assert_eq!(runtime.game_audio.len(), 9, "eight MIX lanes plus independent slot 8");
        assert_eq!(runtime.game_audio[&(13, 8)], independent);
        assert_eq!(audio.loaded_channel_count(PalSoundGroup::GROUP1), 9);
        let first = runtime.game_audio[&(13, 0)];
        let second = runtime.game_audio[&(13, 1)];
        assert_ne!(first, second);
        assert_eq!(audio.channel_volume(first).unwrap().raw(), 2500);
        assert_eq!(audio.channel_volume(second).unwrap().raw(), 0);
        assert_eq!(audio.channel_pan(first).unwrap(), -3000);
        assert_eq!(audio.channel_pan(second).unwrap(), 3000);
        assert!(runtime.load_voice_audio(0, PalSoundGroup::GROUP1, "chorus", 0xC0, 100,
            false, 0, Some(&mut resources), Some(&mut audio)));
        for slot in 0..8 {
            let handle = runtime.game_audio[&(13, slot)];
            assert_eq!(audio.channel_volume(handle).unwrap().raw(), 10000);
            assert_eq!(audio.channel_pan(handle).unwrap(), 0);
        }
        // A following ordinary ADV line releases every member of the chorus.
        let (assets, _) = empty_core_assets(false);
        let voice = runtime.store_dynamic_string("speaker0_line.wav".into());
        runtime.try_play_text_voice(voice, &assets, Nls::ShiftJis,
            Some(&mut resources), Some(&mut audio));
        assert_eq!(runtime.game_audio.len(), 2);
        assert!(runtime.game_audio.contains_key(&(13, 0)));
        assert_eq!(runtime.game_audio[&(13, 8)], independent);
        assert_eq!(audio.loaded_channel_count(PalSoundGroup::GROUP1), 2);
        // voice_stop(-1) uses the same map and releases the complete group.
        runtime.stack = vec![0, -1];
        runtime.ext_audio_stop(13, PalSoundGroup::GROUP1, Some(&mut audio));
        assert!(runtime.game_audio.is_empty());
        assert_eq!(audio.loaded_channel_count(PalSoundGroup::GROUP1), 0);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn voice_autopan_initialize_consumes_limit_without_disabling() {
        let (assets, points) = empty_core_assets(false);
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.text_state.voice_autopan_enabled = true;
        runtime.stack.extend([991, 30]);
        runtime.dispatch_extcall(13, 14, &[], &assets, &points, None, None, None, None, None);
        assert_eq!(runtime.stack, [991]);
        assert_eq!(runtime.text_state.voice_autopan_limit, 3000);
        assert!(runtime.text_state.voice_autopan_enabled);
    }

    #[test]
    fn voice_autopan_uses_first_prefix_and_native_twenty_step_range() {
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.text_state.voice_autopan_enabled = true;
        runtime.text_state.voice_autopan_limit = 3000;
        for (slot, name, target) in [(1, "VO", 5), (2, "VO01", 20)] {
            runtime.text_state.voice_autopan_entries.insert(slot, VoiceAutopanEntry {
                name: name.into(), target, mode: 1, name_value: 0,
            });
        }
        let entry = runtime.voice_autopan_match("vo01_clip", 0).unwrap();
        assert_eq!(entry.target, 5);
        assert_eq!(runtime.voice_autopan_value(entry, None), Some(-1500));
        assert!(runtime.voice_autopan_match("vo01_clip", 0x80).is_none());
        for (target, expected) in [(-1, -3000), (0, -3000), (10, 0), (20, 3000), (21, 3000)] {
            let entry = VoiceAutopanEntry { target, mode: 1, ..Default::default() };
            assert_eq!(runtime.voice_autopan_value(&entry, None), Some(expected));
        }
    }

    #[test]
    fn voice_autopan_tracks_a_moving_sprite_and_drops_replaced_channels() {
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.text_state.voice_autopan_enabled = true;
        runtime.text_state.voice_autopan_limit = 3000;
        let mut sprites = SpriteSystem::new();
        let mut desc = SpriteDesc::new(SceneTextureId(1), 200, 400);
        desc.position.x = 860.0;
        let sprite = sprites.create(desc);
        runtime.game_sprites.insert(7, sprite);
        let mut audio = AudioSystem::new(crate::audio::AudioConfig { enabled: false }).unwrap();
        let data = kira::sound::static_sound::StaticSoundData {
            sample_rate: 44100,
            frames: vec![kira::Frame::new(0.0, 0.0); 16].into(),
            settings: Default::default(),
            slice: None,
        };
        let handle = audio.load_static_data("voice", data, PalSoundGroup::GROUP1).unwrap();
        runtime.game_audio.insert((13, 0), handle);
        runtime.text_state.voice_autopan_playing.insert(0, (handle, VoiceAutopanEntry {
            target: 7, mode: 0, ..Default::default()
        }));
        runtime.update_voice_autopan(Some(&sprites), Some(&mut audio));
        assert_eq!(audio.channel_pan(handle).unwrap(), 0);
        sprites.get_mut(sprite).unwrap().position.x = 1820.0;
        runtime.update_voice_autopan(Some(&sprites), Some(&mut audio));
        assert_eq!(audio.channel_pan(handle).unwrap(), 3000);
        runtime.game_audio.remove(&(13, 0));
        runtime.update_voice_autopan(Some(&sprites), Some(&mut audio));
        assert!(runtime.text_state.voice_autopan_playing.is_empty());
    }

    #[test]
    fn language_switch_extcall_selects_the_pressed_field() {
        let (assets, points) = empty_core_assets(true);
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.set_text_lang(TextLang::Cn);
        // The SYSTEM language row passes the picked text field: 2 = traditional.
        runtime.stack.push(2);
        let outcome =
            runtime.dispatch_extcall(9, 60, &[], &assets, &points, None, None, None, None, None);
        assert!(matches!(outcome, ExtCallOutcome::Value(1)));
        assert!(runtime.stack.is_empty(), "the field argument is consumed");
        assert_eq!(runtime.text_lang, TextLang::Tc);
        assert!(runtime.text_state.dirty, "the parked line re-typesets");
    }

    #[test]
    fn unset_language_field_keeps_the_configured_language() {
        let (assets, points) = empty_core_assets(true);
        let mut runtime = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        runtime.set_text_lang(TextLang::Cn);
        // A fresh install stores 0 in the script's setting and the boot path
        // calls the extcall with it; the host's configured language stands.
        runtime.stack.push(0);
        let outcome =
            runtime.dispatch_extcall(9, 60, &[], &assets, &points, None, None, None, None, None);
        assert!(matches!(outcome, ExtCallOutcome::Value(1)));
        assert!(runtime.stack.is_empty());
        assert_eq!(runtime.text_lang, TextLang::Cn);
    }

    #[test]
    fn text_language_field_index_round_trips() {
        for lang in [TextLang::Jp, TextLang::Cn, TextLang::Tc] {
            assert_eq!(
                TextLang::from_field_index(lang.field_index() as i32),
                Some(lang)
            );
        }
        assert_eq!(TextLang::from_field_index(-1), None);
        assert_eq!(TextLang::from_field_index(3), None);
    }

    #[test]
    fn native_helper_calls_preserve_stack_and_capture_release_savepoints() {
        let (assets, points) = empty_core_assets(false);
        let mut vm = ScriptRuntime::boot(0x6B4EC, ScriptRuntimeConfig::default());
        vm.stack = vec![99, 0, 6548];
        assert_eq!(vm.dispatch_extcall(15, 2, &[], &assets, &points,
            None, None, None, None, None), ExtCallOutcome::Value(1));
        assert_eq!(vm.stack, vec![99]);
        assert!(vm.save_state.armed);
        assert_eq!(vm.save_state.checkpoint.as_ref().unwrap().pc, 0x6B4EC);
        vm.save_state.locked = true;
        vm.pc = 0x70000;
        vm.stack.extend([1, 6548]);
        vm.dispatch_extcall(15, 2, &[], &assets, &points, None, None, None, None, None);
        assert_eq!(vm.save_state.checkpoint.as_ref().unwrap().pc, 0x6B4EC);
        assert_eq!(vm.stack, vec![99]);
    }

    #[test]
    fn voice_info_consumes_four_values_and_keeps_name_on_negative_resource() {
        let (assets, points) = empty_core_assets(false);
        let mut vm = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        let name = vm.store_dynamic_string("vo01".into());
        vm.stack = vec![99, 1, 37, name, 2];
        vm.dispatch_extcall(13, 4, &[], &assets, &points, None, None, None, None, None);
        assert_eq!(vm.stack, vec![99]);
        assert_eq!(vm.text_state.voice_info[&2].name, "vo01");
        assert_eq!(vm.text_state.voice_info[&2].volume, 37);
        vm.stack.extend([0, 61, -1, 2]);
        vm.dispatch_extcall(13, 4, &[], &assets, &points, None, None, None, None, None);
        assert_eq!(vm.stack, vec![99]);
        assert_eq!(vm.text_state.voice_info[&2].name, "vo01");
        assert_eq!(vm.text_state.voice_info[&2].volume, 61);
        assert_eq!(vm.text_state.voice_info[&2].enabled, 0);
    }

    #[test]
    fn native_voice_info_applies_once_to_voice_text_history_and_mix_members() {
        let root = std::env::temp_dir().join(format!("sena_voice_info_{}_{}", std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&root).unwrap();
        let mut wav = Vec::new();
        wav.extend(b"RIFF"); wav.extend(40u32.to_le_bytes());
        wav.extend(b"WAVEfmt "); wav.extend(16u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes()); wav.extend(1u16.to_le_bytes());
        wav.extend(8000u32.to_le_bytes()); wav.extend(16000u32.to_le_bytes());
        wav.extend(2u16.to_le_bytes()); wav.extend(16u16.to_le_bytes());
        wav.extend(b"data"); wav.extend(4u32.to_le_bytes()); wav.extend([0u8; 4]);
        for name in ["vo01_line", "vo02_line", "VO01_upper", "unknown"] {
            std::fs::write(root.join(format!("{name}.wav")), &wav).unwrap();
        }
        let mut mix = vec![0u8; 64 * 3];
        for (lane, name) in ["vo01_line", "VO01_upper", "unknown"].iter().enumerate() {
            for (index, unit) in name.encode_utf16().enumerate() {
                mix[lane * 64 + index * 2..lane * 64 + index * 2 + 2]
                    .copy_from_slice(&unit.to_le_bytes());
            }
        }
        std::fs::write(root.join("group.mix"), mix).unwrap();
        let mut resources = ResourceManager::new(&root, Nls::ShiftJis);
        let mut audio = AudioSystem::new(AudioConfig { enabled: false }).unwrap();
        let (assets, points) = empty_core_assets(false);
        let mut vm = ScriptRuntime::boot(12, ScriptRuntimeConfig::default());
        vm.voice_unit_headers = Some(vec!["vo01".into(), "vo02".into()]);
        vm.voice_ex_volume_percent.insert(0, 7);
        vm.voice_ex_volume_percent.insert(1, 9);
        vm.stack = vec![1];
        vm.dispatch_extcall(13, 7, &[], &assets, &points, None, None, None, None, None);
        for (slot, name, volume, enabled) in [(0, "vo", 40, 1), (1, "vo01", 20, 1),
            (2, "VO", 80, 0), (-1, "", 15, 1)] {
            let name = vm.store_dynamic_string(name.into());
            vm.stack = vec![991, enabled, volume, name, slot];
            vm.dispatch_extcall(13, 4, &[], &assets, &points, None, None, None, None, None);
            assert_eq!(vm.stack, vec![991]);
        }
        vm.stack = vec![50];
        vm.dispatch_extcall(13, 2, &[], &assets, &points, None, None, None, Some(&mut audio), None);
        let voice = vm.store_dynamic_string("vo01_line".into());
        vm.stack = vec![991, 0, 0, voice, 0];
        vm.dispatch_extcall(13, 0, &[], &assets, &points, Some(&mut resources), None, None, Some(&mut audio), None);
        assert_eq!(vm.stack, vec![991]);
        let handle = vm.game_audio[&(13, 0)];
        assert_eq!(audio.channel_volume(handle).unwrap().raw(), 4000);
        assert_eq!(audio.effective_volume(handle).unwrap().raw(), 2000);
        let voice = vm.store_dynamic_string("vo02_line".into());
        vm.try_play_text_voice(voice, &assets, Nls::ShiftJis, Some(&mut resources), Some(&mut audio));
        assert_eq!(audio.channel_volume(vm.game_audio[&(13, 0)]).unwrap().raw(), 4000);
        vm.replay_history_voice("unknown", Some(&mut resources), Some(&mut audio));
        assert_eq!(audio.channel_volume(vm.game_audio[&(13, 0)]).unwrap().raw(), 1500);
        assert!(vm.load_voice_audio(0, PalSoundGroup::GROUP1, "group", 0, 7, true, 0,
            Some(&mut resources), Some(&mut audio)));
        for (lane, volume) in [(0, 4000), (1, 0), (2, 1500)] {
            assert_eq!(audio.channel_volume(vm.game_audio[&(13, lane)]).unwrap().raw(), volume);
        }
        assert!(vm.load_voice_audio(0, PalSoundGroup::GROUP1, "group", 0x40, 7, true, 0,
            Some(&mut resources), Some(&mut audio)));
        for lane in 0..3 {
            assert_eq!(audio.channel_volume(vm.game_audio[&(13, lane)]).unwrap(), PalVolume::MAX);
        }
        vm.stack = vec![0];
        vm.dispatch_extcall(13, 7, &[], &assets, &points, None, None, None, None, None);
        vm.replay_history_voice("unknown", Some(&mut resources), Some(&mut audio));
        assert_eq!(audio.channel_volume(vm.game_audio[&(13, 0)]).unwrap(), PalVolume::MAX);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn debug_print_consumes_all_eight_arguments_in_each_release() {
        let mut runtime = ScriptRuntime::boot(0, ScriptRuntimeConfig::default());
        runtime.stack = vec![
            77,
            0x0FFF_FFFF,
            0x0FFF_FFFF,
            0x0FFF_FFFF,
            0x0FFF_FFFF,
            0x0FFF_FFFF,
            1,
            20,
            11439,
        ];
        assert!(matches!(
            runtime.dispatch_misc_system_stub(4, false),
            ExtCallOutcome::Value(1)
        ));
        assert_eq!(runtime.stack, vec![77]);

        runtime.stack = vec![88, 7, 6, 5, 4, 3, 2, 1, 11439];
        assert!(matches!(
            runtime.dispatch_misc_system_stub(4, false),
            ExtCallOutcome::Value(1)
        ));
        assert_eq!(runtime.stack, vec![88]);

        runtime.stack = vec![99, 4, 103, 102, 101, 0, 1, 23, 23212];
        assert!(matches!(
            runtime.dispatch_misc_system_stub(4, true),
            ExtCallOutcome::Value(1)
        ));
        assert_eq!(runtime.stack, vec![99]);
    }

    #[test]
    fn adv_text_frame_clips_body_text_at_the_smooth_reveal_limit() {
        let mut panel_rgba = vec![0u8; 8 * 4 * 4];
        for px in panel_rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[16, 16, 20, 255]);
        }
        let mut text_rgba = vec![0u8; 8 * 2 * 4];
        for px in text_rgba.chunks_exact_mut(4) {
            px.copy_from_slice(&[200, 200, 200, 255]);
        }
        let cache = AdvTextPanelCache {
            thresholds: None,
            panel_width: 8,
            panel_height: 4,
            panel_rgba,
            text_width: 8,
            text_rgba,
            text_origin_x: 0,
            text_origin_y: 1,
            lines: vec![
                AdvTextLineLayout {
                    y: 0,
                    height: 1,
                    char_start: 0,
                    char_count: 4,
                    char_x: vec![0, 2, 4, 6, 8],
                },
                AdvTextLineLayout {
                    y: 1,
                    height: 1,
                    char_start: 4,
                    char_count: 2,
                    char_x: vec![0, 2, 4],
                },
            ],
            full_char_count: 6,
            sprite_x: 3,
            sprite_y: 5,
        };
        let panel_pixel = |rgba: &[u8], x: u32, y: u32| {
            let index = ((y * 8 + x) * 4) as usize;
            rgba[index..index + 4].to_vec()
        };

        // Half-way through the second character of the first line: only the
        // first three text columns are blended, the second line stays hidden.
        let (w, h, rgba, sx, sy) = compose_adv_text_frame(&cache, Some((0, 3)), None, None);
        assert_eq!((w, h, sx, sy), (8, 4, 3, 5));
        assert_eq!(panel_pixel(&rgba, 2, 1), vec![200, 200, 200, 255]);
        assert_eq!(panel_pixel(&rgba, 3, 1), vec![16, 16, 20, 255]);
        assert_eq!(panel_pixel(&rgba, 0, 2), vec![16, 16, 20, 255]);

        // A limit on the second line keeps the first line fully visible.
        let (_, _, rgba, _, _) = compose_adv_text_frame(&cache, Some((1, 2)), None, None);
        assert_eq!(panel_pixel(&rgba, 7, 1), vec![200, 200, 200, 255]);
        assert_eq!(panel_pixel(&rgba, 1, 2), vec![200, 200, 200, 255]);
        assert_eq!(panel_pixel(&rgba, 2, 2), vec![16, 16, 20, 255]);

        // No limit presents the entire cached text block.
        let (_, _, full, _, _) = compose_adv_text_frame(&cache, None, None, None);
        assert_eq!(panel_pixel(&full, 7, 1), vec![200, 200, 200, 255]);
        assert_eq!(panel_pixel(&full, 3, 2), vec![200, 200, 200, 255]);
    }

    #[test]
    fn wait_mark_frame_uses_grayscale_as_coverage() {
        let mut rgba = vec![0u8; 4 * 4 * 4];
        rgba[0] = 255;
        rgba[3] = 255;
        let sheet = DecodedImage {
            width: 4,
            height: 2,
            cell_width: 2,
            cell_height: 2,
            offset_x: 0,
            offset_y: 0,
            rgba,
        };
        let (w, h, glyph) = wait_mark_frame(&sheet, 0, [10, 20, 30, 255]);
        assert_eq!((w, h), (2, 2));
        assert_eq!(&glyph[0..4], &[10, 20, 30, 255]);
        assert_eq!(&glyph[4..8], &[0, 0, 0, 0]);
    }

    fn file_table_decode_falls_back_when_configured_nls_rejects_comments() {
        let bytes = b"// invalid comment byte for some NLS: \x80\n\"vo01\",\"vo01_test\",1\n";
        let table = parse_file_table(bytes, Nls::Gbk).expect("ASCII CSV rows should parse");

        assert_eq!(table.entries.len(), 3);
        assert_eq!(table.entries[2], 1);
    }

    #[test]
    fn portable_save_snapshot_round_trips() {
        let root = std::env::temp_dir().join(format!(
            "sena_rs_save_roundtrip_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let snapshot = RuntimeSaveSnapshot {
            pc: 0x1234_5678,
            call_stack: vec![0x1000, 0x2000],
            user_mem: vec![1, 2, 3],
            system_mem: vec![4, 5],
            temp_mem: vec![6, 7, 8, 9],
            mem_dat_words: vec![10, 11],
            history_records: vec![[1, 2, 3, 4, 5, 6, 7, 8, 9]],
            text_args: [12, 13, 14, 15],
            text_base: 16,
            text_mode: 17,
            text_alpha: Some(200),
            text_visible: true,
            resume_wait_click: true,
            vars: vec![21, 22],
            argument_base: 64,
            title_bytes: b"line".to_vec(),
            bgm_tracks: vec![SavedBgmTrack {
                slot: 0,
                name: "bgm01".to_owned(),
                looping: true,
                loop_start: 1000,
                loop_end: 2000,
            }],
            sprites: vec![SavedSprite {
                slot: 7,
                transition_source: false,
                x: 640,
                y: 360,
                z: 0,
                offset_x: 0,
                offset_y: 0,
                priority: 10,
                scale_bits: 1.5_f32.to_bits(),
                scale_y_bits: Some(0.75_f32.to_bits()),
                color: 0xFFFF_FFFF,
                visible: true,
                rect: [0, 0, 4, 4],
                width: 4,
                height: 4,
                native_projection: Some((0.5, 0.25)),
                center_scale: true,
                rgba: Arc::new(vec![0xAB; 4 * 4 * 4]),
                option_type: 0,
                option_backup: Vec::new(),
            }],
            ..RuntimeSaveSnapshot::default()
        };

        let path =
            write_runtime_save_snapshot(&root, 3, &snapshot).expect("portable save should write");
        assert!(path.is_file());
        let restored =
            read_runtime_save_snapshot(&root, 3).expect("portable save should read back");

        assert_eq!(restored.pc, snapshot.pc);
        assert_eq!(restored.call_stack, snapshot.call_stack);
        assert_eq!(restored.user_mem, snapshot.user_mem);
        assert_eq!(restored.system_mem, snapshot.system_mem);
        assert_eq!(restored.temp_mem, snapshot.temp_mem);
        assert_eq!(restored.mem_dat_words, snapshot.mem_dat_words);
        assert_eq!(restored.history_records, snapshot.history_records);
        assert_eq!(restored.text_args, snapshot.text_args);
        assert_eq!(restored.text_base, snapshot.text_base);
        assert_eq!(restored.text_mode, snapshot.text_mode);
        assert_eq!(restored.text_alpha, Some(200));
        assert_eq!(restored.text_visible, snapshot.text_visible);
        assert_eq!(restored.vars, snapshot.vars);
        assert_eq!(restored.argument_base, 64);
        assert_eq!(restored.title_bytes, b"line");
        assert_eq!(restored.version, PORTABLE_SAVE_VERSION);
        assert!(restored.resume_wait_click);
        assert_eq!(restored.sprites.len(), 1);
        let sprite = &restored.sprites[0];
        assert_eq!(sprite.slot, 7);
        assert_eq!(sprite.native_projection, Some((0.5, 0.25)));
        assert!(sprite.center_scale);
        assert_eq!(sprite.scale_bits, 1.5_f32.to_bits());
        assert_eq!(sprite.scale_y_bits, Some(0.75_f32.to_bits()));
        assert_eq!(restored.bgm_tracks.len(), 1);
        let track = &restored.bgm_tracks[0];
        assert_eq!(track.slot, 0);
        assert_eq!(track.name, "bgm01");
        assert!(track.looping);
        assert_eq!((track.loop_start, track.loop_end), (1000, 2000));

        let v2_snapshot = RuntimeSaveSnapshot {
            pc: snapshot.pc,
            ..RuntimeSaveSnapshot::default()
        };
        let mut older_bytes = encode_runtime_save_snapshot(&v2_snapshot).expect("encode v6");
        older_bytes[8..12].copy_from_slice(&2_u32.to_le_bytes());
        older_bytes.pop();
        let older = decode_runtime_save_snapshot(&mut older_bytes.as_slice()).expect("read v2");
        assert_eq!(older.version, 2);
        assert!(!older.resume_wait_click);
        assert_eq!(older.text_alpha, None, "a save without the field has no alpha");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restored_save_keeps_the_adv_window_alpha() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        runtime.text_state.alpha = 137;
        let snapshot = runtime.capture_save_snapshot();
        assert_eq!(snapshot.version, PORTABLE_SAVE_VERSION);
        let mut bytes = encode_runtime_save_snapshot(&snapshot).expect("encode the save");
        let decoded = decode_runtime_save_snapshot(&mut bytes.as_slice()).expect("decode the save");
        assert_eq!(decoded.text_alpha, Some(137));

        runtime.text_state.alpha = 0;
        runtime.restore_save_snapshot(decoded);
        assert_eq!(
            runtime.text_state.alpha, 137,
            "a restored ADV window keeps the opacity the save recorded"
        );

        // Saves written before the field existed leave the window shown: alpha
        // 0 would paint the base panel away and drop the text onto the CG.
        let mut legacy = RuntimeSaveSnapshot {
            version: 6,
            text_alpha: None,
            ..RuntimeSaveSnapshot::default()
        };
        legacy.pc = 0x2000;
        runtime.text_state.alpha = 0;
        runtime.restore_save_snapshot(legacy);
        assert_eq!(runtime.text_state.alpha, 255);
    }

    #[test]
    fn modal_menu_suspends_and_reparks_adv_click_wait() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        runtime.status = RuntimeStatus::WaitClick { pc: 0x2000 };
        runtime.wait_task_kind = Some(WaitRequest::Click);
        runtime.pending_gosub_points.push_back(7);
        runtime.text_state.visible = true;
        runtime.text_state.show_wait_mark = true;
        assert!(runtime.should_suspend_wait_for_modal());

        runtime.suspend_wait_for_modal();
        assert!(matches!(runtime.status, RuntimeStatus::Running { pc: 0x2000 }));
        assert_eq!(runtime.modal_wait_suspensions.len(), 1);

        // The engine injects the menu gosub: drain the queued point, push the
        // parked PC, and jump.
        assert_eq!(runtime.pending_gosub_points.pop_front(), Some(7));
        runtime.call_stack.push(0x2000);
        runtime.pc = 0x9000;
        // Still inside the modal; no re-park yet.
        assert!(runtime.take_modal_wait_repark().is_none());
        // The menu returns to the PC after the suspended wait instruction.
        runtime.pc = runtime.call_stack.pop().unwrap();
        assert_eq!(
            runtime.take_modal_wait_repark(),
            Some(WaitRequest::Click),
            "the ADV click wait must be re-parked after the modal returns"
        );
        assert!(matches!(runtime.status, RuntimeStatus::WaitClick { pc: 0x2000 }));
        assert!(runtime.text_state.visible);
        assert!(runtime.text_state.show_wait_mark);
        assert!(runtime.modal_wait_suspensions.is_empty());

        // A modal that exits through a different path must not re-park.
        runtime.status = RuntimeStatus::WaitClick { pc: 0x3000 };
        runtime.wait_task_kind = Some(WaitRequest::Click);
        runtime.pending_gosub_points.push_back(8);
        runtime.suspend_wait_for_modal();
        runtime.pc = 0x4560;
        assert_eq!(runtime.take_modal_wait_repark(), None);
        assert!(runtime.modal_wait_suspensions.is_empty());
        assert!(matches!(runtime.status, RuntimeStatus::Running { .. }));
    }

    #[test]
    fn portable_snapshot_serialization_preserves_the_selected_checkpoint() {
        let root = std::env::temp_dir().join(format!(
            "sena_rs_savepoint_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        runtime.pc = 0x5AB34;
        runtime.vars[3] = 42;
        runtime.save_state.armed = true;
        runtime.save_state.checkpoint = Some(runtime.capture_save_snapshot());
        runtime.pc = 0x51F8C;
        runtime.vars[3] = 99;
        let checkpoint = runtime.save_state.checkpoint.clone().expect("checkpoint");
        runtime
            .write_original_save(&root, 4, &checkpoint, Nls::ShiftJis)
            .expect("checkpoint save");
        let restored = read_runtime_save_snapshot(&root, 4).expect("read checkpoint");
        assert_eq!(restored.pc, 0x5AB34);
        assert_eq!(restored.vars[3], 42);
        assert!(restored.temp_mem.len() <= DEFAULT_MEM_SIZE);
        runtime.restore_save_snapshot(restored);
        assert_eq!(runtime.pc, 0x5AB34);
        assert_eq!(runtime.vars[3], 42);
        assert_ne!(runtime.vars[3], 99);

        let mut huge = b"SENARSAV".to_vec();
        huge.extend_from_slice(&2_u32.to_le_bytes());
        huge.extend_from_slice(&0x1000_u32.to_le_bytes());
        huge.extend_from_slice(&0_u32.to_le_bytes());
        huge.extend_from_slice(&0_u32.to_le_bytes());
        huge.extend_from_slice(&0_u32.to_le_bytes());
        huge.extend_from_slice(&0x1000_0000_u32.to_le_bytes());
        let err = decode_runtime_save_snapshot(&mut huge.as_slice());
        assert!(err.is_err(), "oversized temp_mem must not be allocated");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn save_page_keeps_all_slot_thumbnails_and_clears_them_on_refresh() {
        let root = std::env::temp_dir().join(format!(
            "sena_rs_save_page_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("save")).expect("save dir");
        for slot in 0..2 {
            let prefix = OriginalSavePrefix {
                lock: 0,
                title: Nls::ShiftJis
                    .encode(&format!("保存{slot}"))
                    .expect("SJIS title"),
                mosaic: 0,
                resume_pc: 0,
                secondary_pc: -1,
                thumb_width: 2,
                thumb_height: 2,
                pixels: vec![slot as u8 + 1; 16],
            };
            std::fs::write(
                original_save_path(&root, slot),
                encode_original_save(&prefix, &[]),
            )
            .expect("save fixture");
        }
        let mut manager = ResourceManager::new(&root, Nls::ShiftJis);
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        // Native scripts paint save drawings into a canvas sprite slot.  The
        // drawing sprites must inherit that canvas layer instead of floating
        // on a fixed band above popup dialogs.
        let canvas_priority = game_sprite_priority(77, -268);
        let canvas = sprites
            .create_rgba_sprite(
                2,
                2,
                vec![0; 16],
                PalVec3::new(0, 0, 0),
                canvas_priority,
                "sp_create:77",
            )
            .unwrap();
        runtime.game_sprites.insert(77, canvas);
        for (slot, x) in [(0, 10), (1, 200)] {
            runtime.stack.extend_from_slice(&[20, x, slot, 77]);
            assert!(matches!(
                runtime.ext_thumbnail_set(Some(&mut manager), Some(&mut sprites)),
                ExtCallOutcome::Value(1)
            ));
        }
        assert_eq!(runtime.save_state.thumbnail_sprites.len(), 2);
        assert!(runtime
            .save_state
            .thumbnail_sprites
            .values()
            .all(|handle| sprites.get(*handle).is_some_and(|sprite| {
                sprite.effective_priority() == canvas_priority
                    && sprite.color.alpha() == 255
                    && sprite.draw_command(&sprites).is_some()
            })));
        runtime.stack.extend_from_slice(&[60, 200, 1, 77]);
        runtime.ext_save_text_draw(Some(&mut manager), Some(&mut sprites));
        assert_eq!(runtime.save_state.text_sprites.len(), 1);
        let text_handle = *runtime.save_state.text_sprites.values().next().unwrap();
        let text_sprite = sprites.get(text_handle).unwrap();
        assert!(text_sprite.source_name.contains("保存1"));
        assert_eq!(text_sprite.effective_priority(), canvas_priority);
        assert!(text_sprite.draw_command(&sprites).is_some());
        assert!(sprites
            .surface(text_sprite.surface)
            .unwrap()
            .to_scene_texture()
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[3] != 0));
        let handles = runtime
            .save_state
            .thumbnail_sprites
            .values()
            .chain(runtime.save_state.text_sprites.values())
            .copied()
            .collect::<Vec<_>>();
        runtime.clear_save_drawings(&mut sprites, 77);
        assert!(runtime.save_state.thumbnail_sprites.is_empty());
        assert!(runtime.save_state.text_sprites.is_empty());
        assert!(handles
            .into_iter()
            .all(|handle| sprites.get(handle).is_none()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn save_prefers_the_last_adv_wait_over_the_menu_pc() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        runtime.pc = 0x5AB34;
        runtime.save_state.checkpoint = Some(runtime.capture_save_snapshot());
        runtime.pc = 0x31634;
        runtime.save_state.resume_checkpoint = Some(runtime.capture_save_snapshot());
        runtime.pc = 0x51F8C;
        assert_eq!(runtime.resumable_save_snapshot().unwrap().pc, 0x31634);
    }

    #[test]
    fn only_adv_wait_click_marks_a_resumable_scene() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        runtime.text_state.visible = true;
        runtime.text_state.last_text_value = 42;
        runtime.stack.push(-1);
        assert!(matches!(
            runtime.dispatch_wait_ext(1),
            ExtCallOutcome::Wait {
                request: WaitRequest::Click,
                ..
            }
        ));
        assert!(std::mem::take(&mut runtime.adv_wait_checkpoint_pending));
        runtime.stack.push(500);
        assert!(matches!(
            runtime.dispatch_wait_ext(1),
            ExtCallOutcome::Wait {
                request: WaitRequest::ClickOrTime(500),
                ..
            }
        ));
        assert!(!runtime.adv_wait_checkpoint_pending);
    }

    #[test]
    fn save_title_accepts_native_nls_and_older_utf8_headers() {
        let title = "昔から、無口な子供だった。";
        let encoded = Nls::ShiftJis.encode(title).expect("native title");
        assert_eq!(decode_save_title(&encoded, Nls::ShiftJis), title);
        assert_eq!(decode_save_title(title.as_bytes(), Nls::ShiftJis), title);
    }

    #[test]
    fn sprite_range_clear_removes_the_popup_frame() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        for slot in [125, 126, 127] {
            let handle = sprites
                .create_rgba_sprite(
                    2,
                    2,
                    vec![255; 16],
                    PalVec3::new(0, 0, 0),
                    slot,
                    format!("popup:{slot}"),
                )
                .unwrap();
            runtime.game_sprites.insert(slot, handle);
        }
        runtime.stack.extend_from_slice(&[2, 126]);
        assert!(matches!(
            runtime.ext_sp_cls_ex(Some(&mut sprites), None),
            ExtCallOutcome::Value(1)
        ));
        assert!(runtime.stack.is_empty());
        assert!(runtime.game_sprites.contains_key(&125));
        assert!(!runtime.game_sprites.contains_key(&126));
        assert!(!runtime.game_sprites.contains_key(&127));
        assert_eq!(sprites.commands().len(), 1);
    }

    #[test]
    fn loading_pre_menu_scene_releases_menu_sprites() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let background = sprites
            .create_rgba_sprite(2, 2, vec![255; 16], PalVec3::new(0, 0, 0), 1, "scene")
            .unwrap();
        runtime.game_sprites.insert(1, background);
        runtime.pc = 0x1234;
        let snapshot = runtime.capture_resumable_scene(Some(&sprites));
        let menu = sprites
            .create_rgba_sprite(2, 2, vec![128; 16], PalVec3::new(0, 0, 0), 77, "menu")
            .unwrap();
        runtime.game_sprites.insert(77, menu);
        let thumbnail = sprites
            .create_rgba_sprite(2, 2, vec![64; 16], PalVec3::new(20, 20, 0), 77, "thumbnail")
            .unwrap();
        runtime
            .save_state
            .thumbnail_sprites
            .insert((77, 20, 20), thumbnail);
        let transition = sprites.create_transition_handle();
        runtime.game_sprite_transitions.insert(77, transition);
        assert_eq!(runtime.effect_system.effect(1, 10_000, 0), 1);
        runtime.restore_save_snapshot(snapshot.clone());
        runtime.restore_checkpoint_scene(&snapshot, &mut sprites);
        assert_eq!(runtime.pc, 0x1234);
        assert_eq!(runtime.game_sprites.len(), 1);
        assert!(runtime.game_sprites.contains_key(&1));
        assert!(sprites.get(menu).is_none());
        assert!(sprites.get(thumbnail).is_none());
        assert!(runtime.save_state.thumbnail_sprites.is_empty());
        assert!(!runtime.effect_system.active());
        assert!(runtime.game_sprite_transitions.is_empty());
    }

    #[test]
    fn completed_sprite_transition_releases_old_image_in_game_wrapper() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let first = sprites.create_rgba_sprite(1, 1, vec![255; 4], PalVec3::default(), 0, "new").unwrap();
        let second = sprites.create_rgba_sprite(1, 1, vec![128; 4], PalVec3::default(), 0, "old").unwrap();
        let transition = sprites.create_transition_handle();
        runtime.game_sprites.insert(0, first);
        runtime.game_sprite_transition_sources.insert(0, second);
        runtime.game_sprite_transitions.insert(0, transition);
        assert!(sprites.set_transition(transition, 1, Some(first), Some(second), 1, 100, 100));
        sprites.advance_transitions(200, false);
        assert_eq!(sprites.transition_state(transition), 2);
        runtime.release_retired_sprites(&mut sprites);
        assert_eq!(sprites.transition_state(transition), 0);
        assert!(sprites.get(first).is_some());
        assert!(sprites.get(second).is_none());
        assert!(!runtime.game_sprite_transition_sources.contains_key(&0));
    }

    #[test]
    fn restored_scene_keeps_native_projection_and_center_scale() {
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let standing = sprites
            .create_rgba_sprite(
                2,
                2,
                vec![255; 16],
                PalVec3::new(960, 540, 0),
                10,
                "ST01A_A",
            )
            .unwrap();
        let _ = sprites.set_native_projection(standing, Some((2.0 / 3.0, 2.0 / 3.0)));
        let _ = sprites.set_scale_xy(standing, 0.8, 1.2);
        if let Some(sprite) = sprites.get_mut(standing) {
            sprite.center_scale = true;
        }
        runtime.game_sprites.insert(3, standing);
        let snapshot = runtime.capture_resumable_scene(Some(&sprites));
        assert_eq!(snapshot.sprites.len(), 1);
        assert_eq!(
            snapshot.sprites[0].native_projection,
            Some((2.0 / 3.0, 2.0 / 3.0))
        );
        assert!(snapshot.sprites[0].center_scale);

        runtime.restore_save_snapshot(snapshot.clone());
        runtime.restore_checkpoint_scene(&snapshot, &mut sprites);
        let restored_handle = runtime.game_sprites[&3];
        let restored = sprites.get(restored_handle).expect("restored sprite");
        assert_eq!(restored.native_projection, Some((2.0 / 3.0, 2.0 / 3.0)));
        assert!(restored.center_scale);
        assert_eq!(restored.position.x, 960.0);
        assert_eq!(restored.position.y, 540.0);
        assert!((restored.scale - 0.8).abs() < f32::EPSILON);
        assert_eq!(restored.scale_y, Some(1.2));
    }

    #[test]
    fn version_eleven_sprite_records_retain_uniform_scale() {
        // Independent v11 wire record: count, seven signed fields, scale/color,
        // visibility, rectangle, dimensions, projection, center flag and RGBA.
        // Do not construct old fixtures by truncating the current serializer.
        let mut bytes = 1_u32.to_le_bytes().to_vec();
        bytes.extend_from_slice(&[0; 7 * 4]);
        bytes.extend_from_slice(&0.75_f32.to_bits().to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.push(1);
        for word in [0_i32, 0, 1, 1] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in [1_u32, 1, (-1.0_f32).to_bits(), (-1.0_f32).to_bits()] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.push(0);
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&[255; 4]);
        let mut reader = bytes.as_slice();
        let restored = read_saved_sprites(&mut reader, 11).unwrap();
        assert!(reader.is_empty());
        assert_eq!(restored[0].scale_bits, 0.75_f32.to_bits());
        assert_eq!(restored[0].scale_y_bits, None);
    }

    #[test]
    fn native_save_fixture_draws_title_and_thumbnail_when_available() {
        let Some(root) = std::env::var_os("KOIKAKE_ORIGINAL_SAVE_ROOT") else {
            return;
        };
        let root = PathBuf::from(root);
        let mut manager = ResourceManager::new(&root, Nls::ShiftJis);
        let mut runtime = ScriptRuntime::boot(0x1000, ScriptRuntimeConfig::default());
        let mut sprites = SpriteSystem::new();
        let new_prefix = read_original_save_prefix(&root, 5).expect("second native save");
        let old_prefix = read_original_save_prefix(&root, 10).expect("first native save");
        assert_eq!(
            decode_save_title(&new_prefix.title, Nls::ShiftJis),
            "世間に興味が無かったからとかじゃない。"
        );
        assert_eq!(
            decode_save_title(&old_prefix.title, Nls::ShiftJis),
            "昔から、無口な子供だった。"
        );
        assert_ne!(new_prefix.pixels, old_prefix.pixels);
        runtime.stack.extend_from_slice(&[20, 10, 5, 77]);
        assert!(matches!(
            runtime.ext_thumbnail_set(Some(&mut manager), Some(&mut sprites)),
            ExtCallOutcome::Value(1)
        ));
        runtime.stack.extend_from_slice(&[100, 200, 5, 77]);
        assert!(matches!(
            runtime.ext_save_text_draw(Some(&mut manager), Some(&mut sprites)),
            ExtCallOutcome::Value(1)
        ));
        assert_eq!(runtime.save_state.thumbnail_sprites.len(), 1);
        assert_eq!(runtime.save_state.text_sprites.len(), 1);
        let thumb_handle = *runtime
            .save_state
            .thumbnail_sprites
            .values()
            .next()
            .unwrap();
        let thumb_sprite = sprites.get(thumb_handle).unwrap();
        let thumb_pixels = &sprites
            .surface(thumb_sprite.surface)
            .unwrap()
            .to_scene_texture()
            .pixels;
        assert!(thumb_pixels.iter().any(|&value| value != 0));
        let text_handle = *runtime.save_state.text_sprites.values().next().unwrap();
        assert!(sprites
            .get(text_handle)
            .unwrap()
            .draw_command(&sprites)
            .is_some());
        assert!(read_runtime_save_snapshot(&root, 5).is_err());
        assert!(read_runtime_save_snapshot(&root, 10).is_err());

        let original = std::fs::read(original_save_path(&root, 10)).expect("native save");
        let copy = std::env::temp_dir().join(format!(
            "sena_rs_native_lock_{}_{}.dat",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::write(&copy, &original).expect("copy fixture");
        write_save_lock_dword(&copy, 1).expect("patch copied lock");
        let patched = std::fs::read(&copy).expect("patched fixture");
        assert_eq!(&patched[..4], &1_i32.to_le_bytes());
        assert_eq!(&patched[4..], &original[4..]);
        let _ = std::fs::remove_file(copy);
    }

    #[test]
    fn import_original_save_resumes_at_recorded_text_pc() {
        let root = std::env::temp_dir().join(format!(
            "sena_rs_import_original_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("save")).expect("save dir");
        let prefix = OriginalSavePrefix {
            lock: 0,
            title: Nls::ShiftJis.encode("import").expect("SJIS title"),
            mosaic: 0,
            resume_pc: 0x1234,
            secondary_pc: -1,
            thumb_width: 2,
            thumb_height: 2,
            pixels: vec![0; 16],
        };
        std::fs::write(
            original_save_path(&root, 0),
            encode_original_save(&prefix, &[]),
        )
        .expect("save fixture");

        let mut runtime = ScriptRuntime::boot(0x100, ScriptRuntimeConfig::default());
        runtime.user_mem[7] = 99;
        runtime.system_mem[8] = 88;
        runtime.stack = vec![1, 2, 3];
        runtime.call_stack = vec![0x500];

        let script_bytes = vec![0u8; 0x2000];
        let assets = CoreAssets {
            script: LoadedAsset {
                name: "Script.src".to_owned(),
                bytes: script_bytes,
                source: AssetSource::Loose {
                    path: PathBuf::from("Script.src"),
                },
            },
            file_dat: LoadedAsset {
                name: "File.dat".to_owned(),
                bytes: Vec::new(),
                source: AssetSource::Loose {
                    path: PathBuf::from("File.dat"),
                },
            },
            text_dat: LoadedAsset {
                name: "Text.dat".to_owned(),
                bytes: Vec::new(),
                source: AssetSource::Loose {
                    path: PathBuf::from("Text.dat"),
                },
            },
            mem_dat: LoadedAsset {
                name: "Mem.dat".to_owned(),
                bytes: Vec::new(),
                source: AssetSource::Loose {
                    path: PathBuf::from("Mem.dat"),
                },
            },
            point_dat: LoadedAsset {
                name: "Point.dat".to_owned(),
                bytes: Vec::new(),
                source: AssetSource::Loose {
                    path: PathBuf::from("Point.dat"),
                },
            },
            graphic_dat: None,
            text_dat_multilingual: false,
            file_dat_layout: FileDatLayout::default(),
            script_check_value: 0,
            script_entry_pc: 0x100,
            extended_softpal: false,
            native_build: Default::default(),
            registered_softpal: false,
            point_table: PointTable::parse(&[]).expect("empty Point.dat should parse"),
            graphic_index: None,
        };

        let snapshot = runtime
            .import_original_save(&root, 0, &assets)
            .expect("original save without trailer should import");
        // The resume target is the instruction after the recorded text call.
        assert_eq!(snapshot.pc, 0x1240);
        assert!(snapshot.call_stack.is_empty());
        assert!(snapshot.stack.is_empty());
        assert!(snapshot.sprites.is_empty());
        assert!(snapshot.buttons.is_empty());
        assert!(snapshot.resume_wait_click);
        // The original image carries no memory bodies; they stay live.
        assert_eq!(snapshot.user_mem[7], 99);
        assert_eq!(snapshot.system_mem[8], 88);

        // An out-of-script resume offset is rejected.
        let mut bad = prefix.clone();
        bad.resume_pc = 0x4000;
        std::fs::write(
            original_save_path(&root, 1),
            encode_original_save(&bad, &[]),
        )
        .expect("bad save fixture");
        assert!(runtime.import_original_save(&root, 1, &assets).is_none());
        // A missing file imports nothing.
        assert!(runtime.import_original_save(&root, 7, &assets).is_none());

        let _ = std::fs::remove_dir_all(root);
    }

    /// Real-game fixture: `testcase/` must be a koikake game root with the
    /// original engine's `save010.dat` copied to `save/save000.dat`.
    #[test]
    #[ignore = "needs a local koikake game root at testcase/ with save/save000.dat"]
    fn original_save_fixture_replays_the_first_line() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testcase");
        let mut resource_manager =
            ResourceManager::bootstrap(&root, Nls::ShiftJis).expect("game root");
        let assets = CoreAssets::load(&mut resource_manager, None).expect("core assets");
        let mut runtime = ScriptRuntime::boot(
            assets.script_entry_pc,
            ScriptRuntimeConfig::default(),
        );
        let snapshot = runtime
            .import_original_save(&root, 0, &assets)
            .expect("original save010 should import");
        assert_eq!(snapshot.pc, 0x6B6F8, "resume past the parked text command");
        runtime.restore_save_snapshot(snapshot);

        // The parked line is restored visible without running any script.
        assert!(runtime.text_state.visible);
        assert_eq!(runtime.text_state.last_text_value, 6557);
        assert!(runtime.text_state.show_wait_mark);

        // Continuing runs into the next line's text command.
        let config = ScriptRuntimeConfig::default();
        let mut waited = false;
        for _ in 0..64 {
            let tick = runtime
                .run_frame(&assets, &config)
                .expect("frame should run");
            if matches!(tick.status, RuntimeStatus::WaitClick { .. })
                || matches!(tick.status, RuntimeStatus::WaitFrame { .. })
            {
                if runtime.text_state.last_text_value == 6588 {
                    waited = true;
                    break;
                }
            }
        }
        assert!(waited, "the next line should display after resume");
    }

    /// Real-game fixture: ADV text commands park in a click wait without an
    /// intervening wait_click, so the resumable checkpoint must refresh at
    /// each parked line. Saving must not fall back to the savepoint pc, which
    /// replays the section from its start.
    ///
    /// `testcase/` must be a koikake game root with the original engine's
    /// `save010.dat` copied to `save/save000.dat`.
    #[test]
    #[ignore = "needs a local koikake game root at testcase/ with save/save000.dat"]
    fn adv_lines_refresh_the_resume_checkpoint() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testcase");
        let mut resource_manager =
            ResourceManager::bootstrap(&root, Nls::ShiftJis).expect("game root");
        let assets = CoreAssets::load(&mut resource_manager, None).expect("core assets");
        let mut runtime = ScriptRuntime::boot(
            assets.script_entry_pc,
            ScriptRuntimeConfig::default(),
        );
        // Enter the ADV section through the original save parked on the first
        // line, then arm saving the way the script's savepoint extcall does.
        let snapshot = runtime
            .import_original_save(&root, 0, &assets)
            .expect("original save010 should import");
        runtime.restore_save_snapshot(snapshot);
        runtime.save_state.armed = true;
        let config = ScriptRuntimeConfig::default();

        // Run until the third ADV line (text id 6649) is parked.
        let mut parked = false;
        for _ in 0..40_000 {
            runtime.run_frame(&assets, &config).expect("frame");
            if runtime.text_state.last_text_value == 6649 {
                parked = true;
                break;
            }
            if matches!(runtime.status, RuntimeStatus::WaitClick { .. }) {
                runtime.resolve_pending_wait();
            }
        }
        assert!(parked, "the third ADV line should be parked");
        let checkpoint = runtime
            .save_state
            .resume_checkpoint
            .clone()
            .expect("ADV line waits must refresh the resume checkpoint");
        assert_eq!(checkpoint.text_args[1], 6649);
        assert!(checkpoint.resume_wait_click);
        assert!(checkpoint.show_wait_mark);
        // The checkpoint resumes right after the parked text command
        // (0x6B750 follows the text call at 0x6B744 for line 6649).
        assert_eq!(checkpoint.pc, 0x6B750);

        // Restoring into a fresh runtime resumes at the parked line and the
        // next click advances into the following line (text id 6692).
        let mut restored = ScriptRuntime::boot(
            assets.script_entry_pc,
            ScriptRuntimeConfig::default(),
        );
        restored.restore_save_snapshot(checkpoint);
        assert!(restored.text_state.visible);
        assert_eq!(restored.text_state.last_text_value, 6649);
        let mut advanced = false;
        for _ in 0..400 {
            restored.run_frame(&assets, &config).expect("frame");
            if restored.text_state.last_text_value == 6692 {
                advanced = true;
                break;
            }
            if matches!(restored.status, RuntimeStatus::WaitClick { .. }) {
                restored.resolve_pending_wait();
            }
        }
        assert!(advanced, "resume should continue into the next line");
    }

    /// Real-game fixture: a save snapshot records the playing BGM and a restore
    /// stops the outgoing scene's track before replaying the saved one.
    ///
    /// `testcase/` must be a koikake game root; needs a working audio device.
    #[test]
    #[ignore = "needs a local koikake game root at testcase/ and an audio device"]
    fn restored_save_replays_bgm_and_stops_the_outgoing_track() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testcase");
        let mut resource_manager =
            ResourceManager::bootstrap(&root, Nls::ShiftJis).expect("game root");
        let assets = CoreAssets::load(&mut resource_manager, None).expect("core assets");
        let mut audio = AudioSystem::new(AudioConfig::default()).expect("audio system");
        if !audio.is_enabled() {
            eprintln!("audio device unavailable; skipping bgm replay fixture");
            return;
        }
        let mut runtime = ScriptRuntime::boot(
            assets.script_entry_pc,
            ScriptRuntimeConfig::default(),
        );
        assert!(runtime.load_named_audio(
            4,
            0,
            PalSoundGroup::GROUP3,
            "BGM01",
            1,
            100,
            true,
            None,
            Some(&mut resource_manager),
            Some(&mut audio),
        ));
        let saved_handle = runtime.game_audio[&(4, 0)];
        assert!(audio.is_playing(saved_handle).unwrap_or(false));
        let snapshot = runtime.capture_save_snapshot();
        assert_eq!(snapshot.bgm_tracks.len(), 1);
        assert_eq!(snapshot.bgm_tracks[0].slot, 0);
        assert_eq!(snapshot.bgm_tracks[0].name, "BGM01");
        assert!(snapshot.bgm_tracks[0].looping);
        let bytes = encode_runtime_save_snapshot(&snapshot).expect("encode v5");
        let decoded =
            decode_runtime_save_snapshot(&mut bytes.as_slice()).expect("decode v5");
        assert_eq!(decoded.bgm_tracks.len(), 1);
        assert_eq!(decoded.bgm_tracks[0].name, "BGM01");
        // The saving session is over; its channels do not share the audio
        // backend with the loading session.
        audio.release(saved_handle).expect("release saved track");

        // The runtime that loads the save is playing a different track.  Audio
        // handles are slot ids, so prove the release through the slot's loop
        // region instead: it survives a stale slot but not release+reload.
        let mut restored = ScriptRuntime::boot(
            assets.script_entry_pc,
            ScriptRuntimeConfig::default(),
        );
        assert!(restored.load_named_audio(
            4,
            1,
            PalSoundGroup::GROUP3,
            "BGM16",
            1,
            100,
            true,
            None,
            Some(&mut resource_manager),
            Some(&mut audio),
        ));
        let outgoing_handle = restored.game_audio[&(4, 1)];
        audio
            .set_loop_samples(outgoing_handle, 123, 456)
            .expect("mark outgoing track");

        restored.restore_save_snapshot(decoded);
        assert!(restored.bgm_replay_pending);
        restored.replay_restored_bgm(Some(&mut resource_manager), Some(&mut audio));
        assert!(!restored.bgm_replay_pending);
        assert!(
            !restored.game_audio.contains_key(&(4, 1)),
            "the outgoing scene's script slot must be released by the restore replay"
        );
        let restored_handle = restored.game_audio[&(4, 0)];
        assert_eq!(
            restored.bgm_slots.get(&0).map(|state| state.name.as_str()),
            Some("BGM01")
        );
        assert!(
            audio.is_playing(restored_handle).unwrap_or(false),
            "the saved BGM must be playing after the restore replay"
        );
        assert_ne!(
            audio.loop_samples(restored_handle).ok(),
            Some((123, 456)),
            "the reused channel must come from a fresh load, not the outgoing track"
        );

        // Full round trip through the script-facing save/load extcalls: the
        // written save file must carry the BGM track and a load must re-arm
        // the replay.  Slot 777 keeps the file away from real save pages.
        let slot = 777;
        runtime.save_state.armed = true;
        runtime.save_state.checkpoint = Some(runtime.capture_save_snapshot());
        runtime.stack.push(0);
        runtime.stack.push(slot);
        let outcome = runtime.dispatch_save_stub(
            0,
            &assets,
            Nls::ShiftJis,
            Some(&mut resource_manager),
            None,
        );
        assert!(matches!(outcome, ExtCallOutcome::Value(1)));

        let mut loaded = ScriptRuntime::boot(
            assets.script_entry_pc,
            ScriptRuntimeConfig::default(),
        );
        loaded.stack.push(slot);
        let outcome = loaded.dispatch_save_stub(
            1,
            &assets,
            Nls::ShiftJis,
            Some(&mut resource_manager),
            None,
        );
        assert!(matches!(
            outcome,
            ExtCallOutcome::Value(1) | ExtCallOutcome::Wait { .. }
        ));
        assert!(loaded.bgm_replay_pending);
        assert_eq!(
            loaded.bgm_slots.get(&0).map(|state| state.name.as_str()),
            Some("BGM01")
        );
        loaded.replay_restored_bgm(Some(&mut resource_manager), Some(&mut audio));
        let loaded_handle = loaded.game_audio[&(4, 0)];
        assert!(
            audio.is_playing(loaded_handle).unwrap_or(false),
            "the extcall-loaded save must replay its BGM"
        );
        let _ = std::fs::remove_file(original_save_path(&root, slot));
    }
}
