use std::collections::HashSet;
use std::path::PathBuf;

use pal_asset::{Nls, ResourceManager};

use crate::assets::CoreAssets;
use crate::audio::{AudioConfig, AudioHandle, AudioSystem, PalSoundGroup};
use crate::config::{ini_graphics_size, parse_system_ini, EngineStartupConfig, TextLang};
use crate::debug::{collect_frame_dump, pal_debug_enabled, pal_debug_frame_enabled, print_dump};
use crate::effect::SceneTransition;
use crate::event::PalEvent;
use crate::input::PalInputState;
use crate::platform_time::{Duration, Instant};
use crate::runtime::{RuntimeStatus, RuntimeTick, ScriptRuntime, ScriptRuntimeConfig, WaitRequest};
use crate::scene::{DrawCommand, FrameScene};
use crate::sprite::SpriteSystem;
use crate::task::TaskSystem;

const MAX_PAL_FRAME_DELTA: Duration = Duration::from_millis(100);

#[derive(Clone, Debug, Default)]
pub struct TraceConfig {
    pub script: bool,
    pub scene: bool,
    pub sprites: bool,
    pub assets: bool,
    pub render: bool,
    pub buttons: bool,
    pub input: bool,
    pub text: bool,
    pub actions: bool,
}

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub game_root: Option<PathBuf>,
    pub nls: Nls,
    /// Which field of a multi-language text record the engine displays.
    pub lang: TextLang,
    /// TrueType face used as the text fallback instead of the auto-discovered
    /// `fonts/*.ttf` of the game root.
    pub font_file: Option<PathBuf>,
    pub script_runtime: ScriptRuntimeConfig,
    pub audio: AudioConfig,
    pub trace: TraceConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            game_root: None,
            nls: Nls::ShiftJis,
            lang: TextLang::Jp,
            font_file: None,
            script_runtime: ScriptRuntimeConfig::default(),
            audio: AudioConfig::default(),
            trace: TraceConfig::default(),
        }
    }
}

pub struct Engine {
    config: EngineConfig,
    startup_config: Option<EngineStartupConfig>,
    resource_manager: Option<ResourceManager>,
    core_assets: Option<CoreAssets>,
    runtime: Option<ScriptRuntime>,
    sprites: SpriteSystem,
    task_system: TaskSystem,
    audio: AudioSystem,
    frame_clock: FrameClock,
    last_runtime_status: RuntimeStatus,
    input: PalInputState,
    window_physical_size: (u32, u32),
    pal_debug: bool,
    last_scene: Option<FrameScene>,
    scene_transition: Option<SceneTransition>,
    #[cfg(any(feature="nfs", feature="smb"))]
    network_mount: Option<game_network::fs::Mount>,
}

impl Engine {
    /// Read network resources in place; all user data remains under `user_root`.
    /// The owner must call this on its bootstrap/resource worker.
    #[cfg(any(feature="nfs", feature="smb"))]
    pub fn new_network(config:EngineConfig,url:&str,user_root:&std::path::Path)->anyhow::Result<Self> {
        Self::new_with_resources(config, game_network::Directory::connect(url)?,user_root)
    }
    /// Use a host's existing read-only capability, without reconnecting.
    #[cfg(any(feature="nfs",feature="smb"))]
    pub fn new_with_resources(mut config:EngineConfig, source:game_network::Directory,user_root:&std::path::Path)->anyhow::Result<Self> {
        let mount=game_network::fs::Mount::from_directory(source,user_root)?;
        config.game_root=Some(mount.path().to_owned());
        let mut engine=Self::new(config)?;
        engine.network_mount=Some(mount);
        Ok(engine)
    }

    pub fn set_host_touch_available(&mut self, available: bool) {
        if let Some(runtime) = &mut self.runtime {
            runtime.set_host_touch_available(available);
        }
    }

    pub fn set_host_gpu_renderer(&mut self, enabled: bool) {
        if let Some(runtime) = &mut self.runtime {
            runtime.set_host_gpu_renderer(enabled);
        }
    }

    pub fn new(config: EngineConfig) -> anyhow::Result<Self> {
        #[cfg(any(feature="nfs",feature="smb"))]
        if game_network::fs::is_remote(config.game_root.as_deref().unwrap_or(std::path::Path::new(""))) {
            game_network::fs::user_root(config.game_root.as_deref().unwrap_or(std::path::Path::new("")))?.ok_or_else(||anyhow::anyhow!("network runtime requires an explicit local user root"))?;
        }

        let (startup_config, resource_manager, core_assets, runtime, last_runtime_status) =
            match &config.game_root {
                Some(root) => {
                    let mut startup_config = EngineStartupConfig::load(root, config.nls)?;
                    let mut resource_manager = ResourceManager::bootstrap_with_path_order(
                        root,
                        config.nls,
                        config.lang.archive_list(),
                        crate::assets::native_profile::has_registered_dispatch_exports(root),
                    )?;
                    // Releases that keep the whole PAL payload inside data.pac
                    // ship no loose SYSTEM.INI, so the logical size, font type
                    // and font names come from the archived copy. The loose
                    // file stays authoritative, and a missing one is still
                    // non-fatal.
                    if startup_config.system_ini.is_none() {
                        match resource_manager.open("SYSTEM.INI") {
                            Ok(asset) => match parse_system_ini(&asset.bytes, config.nls) {
                                Ok(ini) => {
                                    log::info!(
                                        "SYSTEM.INI loaded from the PAL archive; its logical size \
                                         and font keys apply"
                                    );
                                    startup_config.system_ini = Some(ini);
                                }
                                Err(err) => log::warn!("archived SYSTEM.INI parse failed: {err}"),
                            },
                            Err(err) => log::debug!("no loose or archived SYSTEM.INI: {err}"),
                        }
                    }
                    let core_assets = CoreAssets::load(
                        &mut resource_manager,
                        startup_config.system_dat.as_ref(),
                    )?;
                    log::info!(
                        "loaded PAL assets: script_check=0x{:08X}, entry=0x{:08X}, points={}, \
                         file_dat=0x{:X}, text_dat=0x{:X}, mem_dat=0x{:X}, graphic_entries={}",
                        core_assets.script_check_value,
                        core_assets.script_entry_pc,
                        core_assets.point_table.len(),
                        core_assets.file_dat.bytes.len(),
                        core_assets.text_dat.bytes.len(),
                        core_assets.mem_dat.bytes.len(),
                        core_assets
                            .graphic_index
                            .as_ref()
                            .map_or(0, |index| index.len())
                    );
                    let mut runtime = ScriptRuntime::boot(
                        core_assets.script_entry_pc,
                        config.script_runtime.clone(),
                    );
                    runtime.set_font_profile(
                        core_assets.registered_softpal,
                        core_assets.text_dat_multilingual,
                    );
                    runtime.set_text_lang(config.lang);
                    if let Some(system_ini) = startup_config.system_ini.clone() {
                        runtime.set_system_ini(system_ini);
                    }
                    runtime.load_configured_font(
                        &mut resource_manager,
                        config.nls,
                        Some(root),
                        config.font_file.as_deref(),
                        config.lang,
                        core_assets.text_dat_multilingual,
                    );
                    // Initialise the writable Mem.dat shadow used by MemDatDirect writes.
                    runtime.load_mem_dat(&core_assets.mem_dat.bytes);
                    runtime.load_portable_system_data(root);
                    let (window_width, window_height) = startup_config.window_size(
                        FrameScene::PAL_DEFAULT_WIDTH,
                        FrameScene::PAL_DEFAULT_HEIGHT,
                    );
                    runtime.observe_window_geometry(
                        0,
                        0,
                        window_width as i32,
                        window_height as i32,
                        false,
                    );
                    let status = runtime.status().clone();
                    (
                        Some(startup_config),
                        Some(resource_manager),
                        Some(core_assets),
                        Some(runtime),
                        status,
                    )
                }
                None => (None, None, None, None, RuntimeStatus::NotBooted),
            };

        let audio = AudioSystem::new(config.audio.clone())?;
        let sprites = SpriteSystem::new();
        let task_system = TaskSystem::new();

        let fallback_size = startup_config.as_ref().map_or(
            (
                FrameScene::PAL_DEFAULT_WIDTH,
                FrameScene::PAL_DEFAULT_HEIGHT,
            ),
            |config| {
                config.window_size(
                    FrameScene::PAL_DEFAULT_WIDTH,
                    FrameScene::PAL_DEFAULT_HEIGHT,
                )
            },
        );

        let mut engine = Self {
            #[cfg(any(feature="nfs", feature="smb"))]
            network_mount: None,
            config,
            startup_config,
            resource_manager,
            core_assets,
            runtime,
            sprites,
            task_system,
            audio,
            frame_clock: FrameClock::new(),
            last_runtime_status,
            input: PalInputState::new(),
            window_physical_size: fallback_size,
            pal_debug: pal_debug_enabled(),
            last_scene: None,
            scene_transition: None,
        };
        if let Some(runtime) = &engine.runtime {
            runtime.apply_persisted_audio_levels(&mut engine.audio);
            if engine
                .core_assets
                .as_ref()
                .is_some_and(|assets| assets.registered_softpal)
            {
                runtime.apply_registered_se_levels(&mut engine.audio);
            }
        }
        Ok(engine)
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    pub fn startup_config(&self) -> Option<&EngineStartupConfig> {
        self.startup_config.as_ref()
    }

    pub fn resource_manager(&mut self) -> Option<&mut ResourceManager> {
        self.resource_manager.as_mut()
    }

    pub fn core_assets(&self) -> Option<&CoreAssets> {
        self.core_assets.as_ref()
    }

    /// Write the current parked ADV checkpoint for host-driven reproduction.
    pub fn save_checkpoint(&self, slot: i32) -> anyhow::Result<std::path::PathBuf> {
        let root = self.config.game_root.as_deref().ok_or_else(|| anyhow::anyhow!("no game root"))?;
        let runtime = self.runtime.as_ref().ok_or_else(|| anyhow::anyhow!("no script runtime"))?;
        runtime.save_host_checkpoint(root, slot, &self.sprites, self.config.nls)
    }

    /// Restore before the first host step; normal VM execution resumes afterward.
    pub fn load_checkpoint(&mut self, slot: i32) -> anyhow::Result<()> {
        let root = self.config.game_root.as_deref().ok_or_else(|| anyhow::anyhow!("no game root"))?;
        let runtime = self.runtime.as_mut().ok_or_else(|| anyhow::anyhow!("no script runtime"))?;
        let (wait,clock)=runtime.load_host_checkpoint(root,slot,&mut self.sprites)?;
        self.task_system.pal_time_ms=clock;
        runtime.set_pal_time(clock);
        if let Some(wait)=wait {
            let handle=match wait {
                WaitRequest::Frame(n)=>self.task_system.create_wait_frame(n),
                WaitRequest::Time(ms)|WaitRequest::TextReveal(ms)=>self.task_system.create_wait_time(ms),
                WaitRequest::Click=>self.task_system.create_wait_click(),
                WaitRequest::ClickOrTime(ms)=>self.task_system.create_wait_click_or_time(ms),
                WaitRequest::AutoClickOrTime(ms)=>self.task_system.create_wait_click_or_time_gated(ms,true),
                WaitRequest::SpriteFx(sprite)=>self.task_system.create_wait_sprite_fx(sprite),
            };
            if let Some(handle)=handle {runtime.set_wait_handle(handle,wait);}
        }
        self.last_runtime_status = runtime.status().clone();
        Ok(())
    }

    pub fn runtime(&self) -> Option<&ScriptRuntime> {
        self.runtime.as_ref()
    }

    pub fn runtime_status(&self) -> &RuntimeStatus {
        &self.last_runtime_status
    }

    pub fn sprites(&self) -> &SpriteSystem {
        &self.sprites
    }

    pub fn sprites_mut(&mut self) -> &mut SpriteSystem {
        &mut self.sprites
    }

    pub fn task_system(&self) -> &TaskSystem {
        &self.task_system
    }

    pub fn task_system_mut(&mut self) -> &mut TaskSystem {
        &mut self.task_system
    }

    pub fn audio(&self) -> &AudioSystem {
        &self.audio
    }

    pub fn audio_mut(&mut self) -> &mut AudioSystem {
        &mut self.audio
    }

    pub fn take_core_assets(&mut self) -> Option<CoreAssets> {
        self.core_assets.take()
    }

    pub fn load_sound_from_resource(
        &mut self,
        name: &str,
        group: PalSoundGroup,
    ) -> anyhow::Result<AudioHandle> {
        let resource_manager = self
            .resource_manager
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("cannot load sound {:?} without a game root", name))?;
        self.audio
            .load_static_from_resource(resource_manager, name, group)
    }

    pub fn play_sound(&mut self, handle: AudioHandle, looping: bool) -> anyhow::Result<()> {
        self.audio.play(handle, looping)
    }

    pub fn stop_sound(&mut self, handle: AudioHandle) -> anyhow::Result<()> {
        self.audio.stop(handle)
    }

    pub fn window_size_from_config(&self, fallback_width: u32, fallback_height: u32) -> (u32, u32) {
        let (default_width, default_height) = self
            .startup_config
            .as_ref()
            .map_or((fallback_width, fallback_height), |config| {
                config.logical_size(fallback_width, fallback_height)
            });
        let width = self
            .startup_config
            .as_ref()
            .map_or(default_width, |config| config.window_width(default_width));
        let height = self
            .startup_config
            .as_ref()
            .map_or(default_height, |config| {
                config.window_height(default_height)
            });
        (width.max(640), height.max(480))
    }

    pub fn logical_size_from_config(
        &self,
        fallback_width: u32,
        fallback_height: u32,
    ) -> (u32, u32) {
        let Some(system_ini) = self
            .runtime
            .as_ref()
            .and_then(|runtime| runtime.parsed_system_ini())
        else {
            return self
                .startup_config
                .as_ref()
                .map_or((fallback_width.max(1), fallback_height.max(1)), |config| {
                    config.logical_size(fallback_width, fallback_height)
                });
        };
        ini_graphics_size(Some(system_ini), fallback_width, fallback_height)
    }

    /// Drain script window operations for the host event thread.
    pub fn take_window_requests(&mut self) -> Vec<crate::system::PalWindowRequest> {
        self.runtime
            .as_mut()
            .map_or_else(Vec::new, ScriptRuntime::take_window_requests)
    }

    /// Host dimensions use logical desktop units; pointer events remain physical pixels.
    pub fn observe_window_geometry(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        fullscreen: bool,
    ) {
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.observe_window_geometry(x, y, width, height, fullscreen);
        }
    }

    /// Clear per-frame transient input state after the current frame has consumed it.
    pub fn input_begin_frame(&mut self) {
        self.input.begin_frame();
    }

    pub fn handle_event(&mut self, event: PalEvent) {
        match event {
            PalEvent::Input(ref input_event) => {
                self.input.handle_input_event(input_event);
            }
            PalEvent::CloseRequested => {}
            PalEvent::Focused { focused } => {
                if let Some(runtime) = self.runtime.as_mut() {
                    runtime.set_host_active(focused);
                }
            }
            PalEvent::Resized { width, height } => {
                self.window_physical_size = (width.max(1), height.max(1));
            }
            PalEvent::ScaleFactorChanged { .. } => {}
            PalEvent::RedrawRequested => {}
        }
    }

    pub fn input_state(&self) -> &PalInputState {
        &self.input
    }

    pub fn diagnostic_button_hit_enabled(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.diagnostic_button_hit_enabled(&self.sprites, x, y))
    }

    pub fn update(&mut self) -> anyhow::Result<EngineFrame> {
        let timing = self.frame_clock.tick_capped(MAX_PAL_FRAME_DELTA);
        self.update_with_timing(timing)
    }

    pub fn update_with_delta(&mut self, delta: Duration) -> anyhow::Result<EngineFrame> {
        let timing = self.frame_clock.tick_fixed(delta);
        self.update_with_timing(timing)
    }

    fn update_with_timing(&mut self, timing: FrameTiming) -> anyhow::Result<EngineFrame> {
        let (logical_width, logical_height) = self.logical_size_from_config(
            FrameScene::PAL_DEFAULT_WIDTH,
            FrameScene::PAL_DEFAULT_HEIGHT,
        );
        self.sprites
            .configure_target_render_size(logical_width, logical_height);
        self.input.set_coordinate_space(
            self.window_physical_size.0,
            self.window_physical_size.1,
            logical_width,
            logical_height,
        );

        // Update PAL cached time used by all task timing (animation delays, wait timeouts).
        self.task_system
            .set_pal_time(timing.elapsed.as_millis() as u32);
        self.input.sample_repeat(self.task_system.pal_time_ms);

        if let Some(runtime) = self.runtime.as_mut() {
            runtime.apply_input_control(&mut self.input);
        }

        let delta_ms = timing.delta.as_millis().min(u32::MAX as u128) as u32;
        self.audio.set_pal_time(self.task_system.pal_time_ms);
        let movie_frame = if let Some(runtime) = self.runtime.as_mut() {
            runtime.set_pal_time(self.task_system.pal_time_ms);
            let active = runtime.advance_movie_transport(
                &mut self.sprites,
                &mut self.audio,
                &mut self.input,
                delta_ms,
            );
            runtime.apply_input_control(&mut self.input);
            active
        } else {
            false
        };

        let mut button_consumed_mouse_push = false;
        let mut button_consumed_mouse_pull = false;
        let mut button_consumed_push_edge = false;
        let mut text_reveal_consumed_push = false;
        if let Some(runtime) = self.runtime.as_mut().filter(|_| !movie_frame) {
            runtime.set_pal_time(self.task_system.pal_time_ms);
            if let Some(core_assets) = self.core_assets.as_ref() {
                let nls = self
                    .resource_manager
                    .as_ref()
                    .map(|manager| manager.nls())
                    .unwrap_or(Nls::ShiftJis);
                runtime.sync_text_sprite(
                    core_assets,
                    nls,
                    self.resource_manager.as_mut(),
                    &mut self.sprites,
                );
                runtime.sync_history_sprite(core_assets, nls, &mut self.sprites);
                runtime.sync_history_icons(core_assets, nls, self.resource_manager.as_mut(), &mut self.sprites);
                runtime.flush_history_icon_sounds(&mut self.audio);
                runtime.flush_history_checkpoints(self.resource_manager.as_ref());
                runtime.prepare_button_audio(
                    core_assets,
                    nls,
                    self.resource_manager.as_mut(),
                    Some(&mut self.audio),
                );
            }
            let button_outcome = runtime.update_button_input_state_with_audio(
                &mut self.sprites,
                &self.input,
                Some(&mut self.audio),
            );
            if runtime.button_reaction_pending() {
                self.input.clear();
            }
            if self.config.trace.buttons {
                let (mx, my) = self.input.mouse_position();
                runtime.dump_button_states(&self.sprites, timing.frame_index, mx, my);
            }
            button_consumed_mouse_push =
                button_outcome.consumed_mouse_push || button_outcome.consumed_mouse_capture;
            button_consumed_mouse_pull = button_outcome.consumed_mouse_pull;
            button_consumed_push_edge = button_outcome.consumed_push_edge;
            if button_outcome.callback_queued && runtime.should_suspend_wait_for_modal() {
                if let Some(handle) = runtime.pending_wait_handle() {
                    let _ = self.task_system.free(handle);
                    runtime.suspend_wait_for_modal();
                }
            } else if button_outcome.consumed_any() {
                if let Some(handle) = runtime.pending_wait_handle() {
                    let _ = self.task_system.free(handle);
                    if runtime.should_suspend_wait_for_modal() {
                        // The click opened a modal menu (SAVE/LOAD/SYSTEM)
                        // while the script was parked at an ADV click wait.
                        // Suspend the wait instead of resolving it so the
                        // story does not advance behind the menu; the runtime
                        // re-parks the wait when the menu's gosub returns.
                        runtime.suspend_wait_for_modal();
                    } else {
                        runtime.resolve_pending_wait();
                    }
                }
            } else if !button_outcome.consumed_mouse_capture
                && runtime.consume_text_reveal_push(&self.input)
            {
                if runtime.pending_wait_is_text_reveal() {
                    if let Some(handle) = runtime.pending_wait_handle() {
                        let _ = self.task_system.free(handle);
                    }
                    runtime.resolve_pending_wait();
                    runtime.flush_deferred_action_clears(&mut self.sprites);
                }
                text_reveal_consumed_push = true;
            }
        }

        // Tips scripts poll their own geometry and mouse state while ADV is
        // parked. Reuse the modal continuation path so returning from a poll
        // restores the same text wait instead of advancing the story.
        if !button_consumed_mouse_push && !text_reveal_consumed_push && !self.input.click_push() {
            if let Some(runtime) = self.runtime.as_mut().filter(|_| !movie_frame) {
                let can_poll = runtime.pending_wait_handle().is_none_or(|handle| {
                    self.task_system
                        .wait_can_poll(handle, runtime.any_voice_playing(&self.audio))
                });
                if can_poll
                    && !runtime.skip_active(&self.input)
                    && self.core_assets.as_ref().is_some_and(|assets| {
                        runtime
                            .queue_tips_tick_checked(&assets.point_table, assets.script.bytes.len())
                    })
                {
                    let clock = runtime
                        .pending_wait_handle()
                        .and_then(|handle| self.task_system.wait_clock(handle));
                    if let Some(handle) = runtime.pending_wait_handle() {
                        let _ = self.task_system.free(handle);
                    }
                    runtime.suspend_wait_for_tips(clock);
                }
            }
        }

        // Native completes the typewriter reveal in the same frame its text
        // tick sees a latched skip state (skip byte, scene skip, held Ctrl);
        // push-edge reveal completion already ran through
        // consume_text_reveal_push above.
        if let Some(runtime) = self.runtime.as_mut().filter(|_| !movie_frame) {
            if runtime.skip_active(&self.input) {
                runtime.complete_text_reveal_for_skip();
            }
        }

        // Process all tasks: animations update sprite source_rect, wait tasks check input.
        // Native button reactions and text reveal consume mouse pushes before
        // wait_click sees them. Otherwise clicking LOG/SAVE/SYSTEM also
        // advances ADV text, and the first click on a still-revealing line
        // skips the line instead of completing the typewriter pass.
        let task_input;
        let input_for_tasks = if text_reveal_consumed_push || button_consumed_push_edge {
            task_input = if button_consumed_mouse_pull {
                self.input.without_push_edges().without_mouse_edges()
            } else {
                self.input.without_push_edges()
            };
            &task_input
        } else if button_consumed_mouse_push || button_consumed_mouse_pull {
            task_input = self.input.without_mouse_edges();
            &task_input
        } else {
            &self.input
        };
        let skip_active = self
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.skip_active(&self.input));
        let voice_active = self
            .runtime
            .as_ref()
            .is_some_and(|runtime| runtime.any_voice_playing(&self.audio));
        self.task_system.process(
            &mut self.sprites,
            input_for_tasks,
            skip_active,
            voice_active,
        );
        self.sprites.set_fx_time(self.task_system.pal_time_ms);
        self.sprites.advance_motion_entries(delta_ms);
        // Native completes in-flight sprite transitions on the same frame a
        // skip state is active instead of playing the remaining frames out.
        self.sprites
            .advance_transitions(self.task_system.pal_time_ms, skip_active);
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.set_pal_time(self.task_system.pal_time_ms);
            if !movie_frame {
                runtime.advance_sprite_action_lanes(&mut self.sprites);
                runtime.advance_actions(&mut self.sprites);
                runtime.play_action_sounds(&mut self.audio);
            }
            runtime.release_retired_sprites(&mut self.sprites);
        }

        #[cfg(target_os = "vita")]
        Self::compact_retired_scene_capture(&mut self.last_scene, &self.sprites, timing.frame_index);

        // Check if the script runtime was waiting on a task that just completed.
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.set_pal_time(self.task_system.pal_time_ms);
            if let Some(handle) = runtime.pending_wait_handle() {
                if !self.task_system.is_alive(handle) {
                    runtime.resolve_pending_wait();
                }
            }
        }

        // Run script VM.  The VM sees the same consumed-stripped input as the
        // task system: a push that triggered a button must not also complete
        // the work-process ADV click wait behind a modal menu.
        let runtime_tick = match (
            self.runtime.as_mut(),
            self.core_assets.as_ref(),
            self.resource_manager.as_mut(),
        ) {
            (Some(runtime), Some(core_assets), resource_manager) if !movie_frame => {
                match runtime.run_frame_with_resources(
                    core_assets,
                    resource_manager,
                    Some(&mut self.sprites),
                    Some(&mut self.task_system),
                    Some(&mut self.audio),
                    Some(input_for_tasks),
                    &self.config.script_runtime,
                ) {
                    Ok(tick) => Some(tick),
                    Err(e) => {
                        log::error!("runtime error: {e}");
                        runtime.set_faulted(format!("{e}"));
                        None
                    }
                }
            }
            _ => None,
        };

        // Input mutations are visible to the host immediately and persist
        // before next frame's button/task/VM consumers read the same state.
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.apply_input_control(&mut self.input);
        }

        // Process any wait request emitted by the VM this tick.
        if let Some(tick) = runtime_tick.as_ref() {
            if let Some(wait_req) = &tick.wait_request {
                // Do not let the input edge that reached a new wait_click also
                // satisfy that freshly-created wait.  Native Game.exe rewinds
                // the script PC and waits for a later polling pass; consuming
                // the same edge here makes ADV lines auto-advance before the
                // typewriter pass is visible.
                let input_satisfied_click_wait = false;
                let tips_remaining_ms = self
                    .runtime
                    .as_mut()
                    .and_then(|runtime| runtime.take_tips_resumed_wait_ms());
                let handle = match wait_req {
                    WaitRequest::Click | WaitRequest::ClickOrTime(_)
                        if input_satisfied_click_wait =>
                    {
                        None
                    }
                    WaitRequest::SpriteFx(sprite) => {
                        self.task_system.create_wait_sprite_fx(*sprite)
                    }
                    WaitRequest::Frame(n) => self.task_system.create_wait_frame(*n),
                    WaitRequest::Time(ms) => {
                        let reveal_ms = self
                            .runtime
                            .as_ref()
                            .map_or(0, |runtime| runtime.text_reveal_remaining_ms());
                        let duration_ms = ms.saturating_add(reveal_ms);
                        if reveal_ms > 0 {
                            log::debug!(
                                "[trace-wait] wait_time extended for text reveal base_ms={ms} reveal_ms={reveal_ms} duration_ms={duration_ms}"
                            );
                        }
                        self.task_system.create_wait_time(duration_ms)
                    }
                    WaitRequest::TextReveal(ms) => self.task_system.create_wait_time(*ms),
                    WaitRequest::Click => self.task_system.create_wait_click(),
                    WaitRequest::ClickOrTime(ms) => {
                        let reveal_ms = self
                            .runtime
                            .as_ref()
                            .map_or(0, |runtime| runtime.text_reveal_remaining_ms());
                        self.task_system.create_wait_click_or_time(
                            tips_remaining_ms.unwrap_or_else(|| ms.saturating_add(reveal_ms)),
                        )
                    }
                    WaitRequest::AutoClickOrTime(ms) => {
                        let reveal_ms = self
                            .runtime
                            .as_ref()
                            .map_or(0, |runtime| runtime.text_reveal_remaining_ms());
                        self.task_system.create_wait_click_or_time_gated(
                            tips_remaining_ms.unwrap_or_else(|| ms.saturating_add(reveal_ms)),
                            true,
                        )
                    }
                };
                match (handle, input_satisfied_click_wait) {
                    (Some(h), _) => {
                        if let Some(runtime) = self.runtime.as_mut() {
                            runtime.set_wait_handle(h, *wait_req);
                        }
                    }
                    (None, true) => {
                        if let Some(runtime) = self.runtime.as_mut() {
                            runtime.resolve_pending_wait();
                        }
                    }
                    (None, false) => {
                        // Pool full: unblock VM immediately rather than hanging forever.
                        log::warn!(
                            "task pool full; wait request cannot be created, VM will continue"
                        );
                        if let Some(runtime) = self.runtime.as_mut() {
                            runtime.resolve_pending_wait();
                        }
                    }
                }
            }

            if tick.status != self.last_runtime_status {
                log::info!("script runtime: {}", tick.status);
            }
            self.last_runtime_status = tick.status.clone();
        }

        self.sprites.tick_fx_effects(self.task_system.pal_time_ms);
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.finish_global_fx_stop(&mut self.sprites);
            runtime.flush_button_audio_releases(&mut self.audio);
        }
        self.audio.update();

        if let Some(runtime) = self.runtime.as_mut() {
            runtime.cancel_scene_effect_on_click(self.input.raw_key_push() & 0x800000 != 0);
        }
        let effect = self
            .runtime
            .as_ref()
            .and_then(ScriptRuntime::effect_render_state);
        let mut scene = self.compose_scene(timing.elapsed, logical_width, logical_height);
        if let Some(effect) = effect {
            if !self.scene_transition.as_ref().is_some_and(|transition| {
                transition.start_ms == effect.start_ms && transition.effect_id == effect.effect_id
            }) {
                let previous = self.last_scene.as_ref().unwrap_or(&scene);
                self.scene_transition = SceneTransition::capture(
                    effect,
                    previous,
                    self.resource_manager.as_mut(),
                    timing.frame_index << 32,
                );
            }
            if let Some(transition) = self.scene_transition.as_mut() {
                transition.append(&mut scene, effect, self.task_system.pal_time_ms);
            } else if let Some(runtime) = self.runtime.as_mut() {
                // Native mask callbacks complete immediately when the file is missing.
                runtime.finish_scene_effect();
            }
        } else {
            self.scene_transition = None;
        }
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.effect_frame_presented();
        }
        self.last_scene = Some(scene.clone());

        if self.pal_debug || pal_debug_frame_enabled(timing.frame_index) {
            let frame_events = runtime_tick
                .as_ref()
                .map(|t| t.frame_events.as_slice())
                .unwrap_or(&[]);
            let dump = collect_frame_dump(
                timing.frame_index,
                self.task_system.pal_time_ms,
                timing.delta.as_millis() as u32,
                &self.last_runtime_status,
                frame_events,
                &self.task_system,
                &self.sprites,
                &scene,
            );
            print_dump(&dump);
        }

        if self.config.trace.sprites {
            log::debug!(
                "[trace-sprites] frame={} sprites={} surfaces={} render_nodes={}",
                timing.frame_index,
                self.sprites.len(),
                self.sprites.surface_count(),
                self.sprites.render_node_count(),
            );
            for (handle, sp) in self.sprites.iter_sprites() {
                log::debug!(
                    "[trace-sprites]   sprite={} vis={} pos=({:.0},{:.0}) src={:?} name={:?}",
                    handle.0,
                    sp.visible,
                    sp.position.x,
                    sp.position.y,
                    sp.source_name,
                    sp.source_name,
                );
            }
        }

        if self.config.trace.scene {
            log::debug!(
                "[trace-scene] frame={} draw_commands={} clear_color={:?}",
                timing.frame_index,
                scene.commands.len(),
                scene.clear_color,
            );
        }

        if self.config.trace.render {
            for (i, cmd) in scene.commands.iter().enumerate() {
                match cmd {
                    crate::scene::DrawCommand::Sprite(sp) => {
                        log::debug!(
                            "[trace-render] [{i}] sprite tex={} dst=({:.0},{:.0},{:.0}x{:.0}) src=({:.3},{:.3}) prio={}",
                            sp.texture_id.0, sp.dst.x, sp.dst.y, sp.dst.w, sp.dst.h,
                            sp.src.x, sp.src.y, sp.priority,
                        );
                    }
                    crate::scene::DrawCommand::SolidQuad(q) => {
                        log::debug!(
                            "[trace-render] [{i}] solid_quad dst=({:.0},{:.0},{:.0}x{:.0})",
                            q.dst.x,
                            q.dst.y,
                            q.dst.w,
                            q.dst.h,
                        );
                    }
                }
            }
        }

        if self.config.trace.text {
            if let Some(runtime) = self.runtime.as_ref() {
                runtime.dump_text_diagnostic_state(
                    &self.sprites,
                    &scene,
                    &self.last_runtime_status,
                    &self.input,
                    timing.frame_index,
                );
            }
        }

        if self.config.trace.input {
            let (mouse_x, mouse_y) = self.input.mouse_position();
            log::debug!(
                "[trace-input] frame={} key_push=0x{:08X} key_on=0x{:08X} key_pull=0x{:08X} mouse_push=0x{:02X} mouse_on=0x{:02X} mouse_pull=0x{:02X} mouse=({}, {})",
                timing.frame_index,
                self.input.raw_key_push(),
                self.input.raw_key_on(),
                self.input.raw_key_pull(),
                self.input.raw_mouse_push(),
                self.input.raw_mouse_on(),
                self.input.raw_mouse_pull(),
                mouse_x,
                mouse_y
            );
        }

        Ok(EngineFrame {
            timing,
            runtime_tick,
            runtime_status: self.last_runtime_status.clone(),
            scene,
        })
    }

    #[cfg(target_os = "vita")]
    fn compact_retired_scene_capture(last_scene: &mut Option<FrameScene>, sprites: &SpriteSystem, generation: u64) {
        let needs_capture = last_scene.as_ref().is_some_and(|scene| {
            scene.textures.iter().any(|texture| {
                texture.pixels.len() >= 8 * 1024 * 1024
                    && sprites.surface(crate::sprite::SpriteSurfaceId(texture.id.0)).is_none()
            })
        });
        if !needs_capture { return; }
        let previous = last_scene.take().expect("previous scene checked above");
        // Global effects already capture through this compositor. Freeze the
        // same pixels before replacing resources, retaining one logical frame
        // rather than full-size retired textures. Current GPU draws are intact.
        let pixels = crate::scene::rasterize_scene_rgba(&previous);
        let texture = crate::scene::SceneTexture::rgba8(
            crate::scene::SceneTextureId(u64::MAX - 3), generation,
            previous.logical_width, previous.logical_height, pixels);
        let draw = crate::scene::SpriteDraw::full_frame(
            &texture, previous.logical_width, previous.logical_height);
        let mut captured = FrameScene::boot()
            .with_logical_size(previous.logical_width, previous.logical_height);
        captured.textures.push(texture);
        captured.commands.push(DrawCommand::Sprite(draw));
        *last_scene = Some(captured);
    }

    fn compose_scene(
        &mut self,
        elapsed: Duration,
        logical_width: u32,
        logical_height: u32,
    ) -> FrameScene {
        let mut scene = FrameScene::from_runtime_status(&self.last_runtime_status, elapsed)
            .with_logical_size(logical_width, logical_height);
        scene.raster_stage = self.sprites.global_raster_stage();
        let mut sprite_commands = self.sprites.commands();
        sprite_commands.extend(self.sprites.transition_commands());
        // Transition sprites participate in the same PAL render tree as normal
        // sprites. Appending them after all live sprites lets background
        // transitions overdraw ADV text/buttons despite lower z/priority.
        sprite_commands.sort_by_key(|command| match command {
            crate::scene::DrawCommand::Sprite(sprite) => sprite.priority,
            crate::scene::DrawCommand::SolidQuad(_) => i32::MAX,
        });
        self.sprites.resolve_main_draw_state(&mut sprite_commands);
        let mut used_textures = HashSet::new();
        for command in &sprite_commands {
            if let crate::scene::DrawCommand::Sprite(sprite) = command {
                used_textures.insert(sprite.texture_id);
            }
        }
        for texture in self.sprites.textures() {
            if used_textures.contains(&texture.id) {
                scene.textures.push(texture.clone());
            }
        }
        scene.commands.extend(sprite_commands);
        if let Some(runtime) = self.runtime.as_ref() {
            let shake = runtime.effect_shake_offset();
            if shake != [0, 0] {
                translate_scene_commands(&mut scene.commands, shake[0], shake[1]);
            }
            if let Some(quad) = runtime.effect_overlay(logical_width, logical_height) {
                scene
                    .commands
                    .push(crate::scene::DrawCommand::SolidQuad(quad));
            }
        }
        scene
    }
}

fn translate_scene_commands(commands: &mut [DrawCommand], x: i32, y: i32) {
    for command in commands {
        match command {
            DrawCommand::Sprite(sprite) => {
                sprite.dst.x += x as f32;
                sprite.dst.y += y as f32;
            }
            DrawCommand::SolidQuad(quad) => {
                quad.dst.x += x as f32;
                quad.dst.y += y as f32;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EngineFrame {
    pub timing: FrameTiming,
    pub runtime_tick: Option<RuntimeTick>,
    pub runtime_status: RuntimeStatus,
    pub scene: FrameScene,
}

#[derive(Clone, Copy, Debug)]
pub struct FrameTiming {
    pub frame_index: u64,
    pub delta: Duration,
    pub elapsed: Duration,
}

#[derive(Debug)]
struct FrameClock {
    start: Instant,
    previous: Instant,
    frame_index: u64,
}

impl FrameClock {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            previous: now,
            frame_index: 0,
        }
    }

    fn tick_capped(&mut self, max_delta: Duration) -> FrameTiming {
        let now = Instant::now();
        let delta = now.saturating_duration_since(self.previous).min(max_delta);
        let elapsed = self
            .previous
            .saturating_duration_since(self.start)
            .saturating_add(delta);
        self.previous = now;
        self.frame_index += 1;
        FrameTiming {
            frame_index: self.frame_index,
            delta,
            elapsed,
        }
    }

    fn tick_fixed(&mut self, delta: Duration) -> FrameTiming {
        let delta = delta.min(MAX_PAL_FRAME_DELTA);
        let elapsed = self
            .previous
            .saturating_duration_since(self.start)
            .saturating_add(delta);
        self.previous = self.start + elapsed;
        self.frame_index += 1;
        FrameTiming {
            frame_index: self.frame_index,
            delta,
            elapsed,
        }
    }
}
