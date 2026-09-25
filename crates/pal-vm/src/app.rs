use std::path::PathBuf;
use std::sync::Arc;

use pal_asset::Nls;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::Key;
use winit::window::{Window, WindowId};

use crate::audio::AudioConfig;
use crate::engine::{Engine, EngineConfig, TraceConfig};
use crate::event::{InputEvent, MouseButton, PalEvent};
use crate::platform_time::{Duration, Instant};
use crate::renderer::{RenderOutcome, Renderer, RendererConfig};
use crate::runtime::{RuntimeStatus, ScriptRuntimeConfig};
use crate::scene::{rasterize_scene_rgba, FrameScene};

#[derive(Clone, Debug)]
pub struct SenaConfig {
    pub game_root: Option<PathBuf>,
    pub nls: Nls,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub renderer: RendererConfig,
    pub prefer_config_window_size: bool,
    pub print_loaded_assets: bool,
    pub trace_script: bool,
    pub trace_scene: bool,
    pub trace_sprites: bool,
    pub trace_assets: bool,
    pub trace_render: bool,
    pub trace_buttons: bool,
    pub trace_input: bool,
    pub trace_text: bool,
    pub trace_actions: bool,
    pub script_budget_per_frame: usize,
    pub audio: AudioConfig,
    pub headless: bool,
    pub diagnostic_frames: usize,
    pub diagnostic_frame_ms: u64,
    pub fixed_timestep_ms: Option<u64>,
    pub diagnostic_png: Option<PathBuf>,
    pub diagnostic_png_at: Vec<DiagnosticPngAt>,
    pub window_dump_frame_at: Vec<DiagnosticPngAt>,
    pub diagnostic_clicks: Vec<DiagnosticClick>,
    pub diagnostic_click_when_hit_enabled: Vec<DiagnosticClickWhenHitEnabled>,
    pub diagnostic_key_events: Vec<DiagnosticKeyEvent>,
    pub diagnostic_auto_advance: Option<DiagnosticAutoAdvance>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticClick {
    pub frame: usize,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticClickWhenHitEnabled {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticKeyEvent {
    pub frame: usize,
    pub key: String,
    pub pressed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticAutoAdvance {
    pub x: i32,
    pub y: i32,
    pub min_frames: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticPngAt {
    pub frame: usize,
    pub path: PathBuf,
}

impl Default for SenaConfig {
    fn default() -> Self {
        Self {
            game_root: None,
            nls: Nls::ShiftJis,
            title: "sena".to_owned(),
            width: FrameScene::PAL_DEFAULT_WIDTH,
            height: FrameScene::PAL_DEFAULT_HEIGHT,
            renderer: RendererConfig::default(),
            prefer_config_window_size: true,
            print_loaded_assets: false,
            trace_script: false,
            trace_scene: false,
            trace_sprites: false,
            trace_assets: false,
            trace_render: false,
            trace_buttons: false,
            trace_input: false,
            trace_text: false,
            trace_actions: false,
            script_budget_per_frame: ScriptRuntimeConfig::default().instructions_per_frame,
            audio: AudioConfig::default(),
            headless: false,
            diagnostic_frames: 120,
            diagnostic_frame_ms: 16,
            fixed_timestep_ms: None,
            diagnostic_png: None,
            diagnostic_png_at: Vec::new(),
            window_dump_frame_at: Vec::new(),
            diagnostic_clicks: Vec::new(),
            diagnostic_click_when_hit_enabled: Vec::new(),
            diagnostic_key_events: Vec::new(),
            diagnostic_auto_advance: None,
        }
    }
}

pub fn run_sena(config: SenaConfig) -> anyhow::Result<()> {
    if config.headless {
        return run_sena_headless(config);
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = SenaApplication::new(config)?;
    event_loop.run_app(&mut app)?;
    Ok(())
}

pub fn run_sena_headless(config: SenaConfig) -> anyhow::Result<()> {
    let mut engine = build_engine(&config)?;
    if config.print_loaded_assets {
        print_asset_summary(&engine);
    }
    let (width, height) = if config.prefer_config_window_size {
        engine.window_size_from_config(config.width, config.height)
    } else {
        (config.width, config.height)
    };
    engine.handle_event(PalEvent::Resized { width, height });
    let (logical_width, logical_height) = engine.logical_size_from_config(
        FrameScene::PAL_DEFAULT_WIDTH,
        FrameScene::PAL_DEFAULT_HEIGHT,
    );
    eprintln!(
        "[headless] start logical={}x{} window={}x{} frames={} frame_ms={} runtime={}",
        logical_width,
        logical_height,
        width,
        height,
        config.diagnostic_frames,
        config.frame_step_ms(),
        engine.runtime_status()
    );

    let mut last_frame = None;
    let mut last_auto_advance_press = None;
    let mut click_when_hit_done = vec![false; config.diagnostic_click_when_hit_enabled.len()];
    let mut click_when_hit_release = None;
    for frame_index in 0..config.diagnostic_frames {
        engine.input_begin_frame();
        if click_when_hit_release == Some(frame_index) {
            engine.handle_event(PalEvent::Input(InputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: false,
            }));
            click_when_hit_release = None;
            eprintln!("[headless] injected click-when-hit release frame={frame_index}");
        }
        for click in &config.diagnostic_clicks {
            if frame_index == click.frame {
                engine.handle_event(PalEvent::Input(InputEvent::CursorMoved {
                    x: click.x as f64,
                    y: click.y as f64,
                }));
                engine.handle_event(PalEvent::Input(InputEvent::MouseInput {
                    button: MouseButton::Left,
                    pressed: true,
                }));
                eprintln!(
                    "[headless] injected click press frame={} pos=({}, {})",
                    frame_index, click.x, click.y
                );
            } else if frame_index == click.frame.saturating_add(1) {
                engine.handle_event(PalEvent::Input(InputEvent::MouseInput {
                    button: MouseButton::Left,
                    pressed: false,
                }));
                eprintln!("[headless] injected click release frame={frame_index}");
            }
        }
        inject_headless_click_when_hit_enabled(
            &mut engine,
            &config,
            frame_index,
            &mut click_when_hit_done,
            &mut click_when_hit_release,
        );
        inject_headless_auto_advance(
            &mut engine,
            &config,
            frame_index,
            &mut last_auto_advance_press,
        );
        for key_event in &config.diagnostic_key_events {
            if frame_index == key_event.frame {
                engine.handle_event(PalEvent::Input(InputEvent::Keyboard {
                    key_name: key_event.key.clone(),
                    pressed: key_event.pressed,
                }));
                eprintln!(
                    "[headless] injected key {} frame={} key={}",
                    if key_event.pressed { "down" } else { "up" },
                    frame_index,
                    key_event.key
                );
            }
        }
        if config.trace_input {
            let input = engine.input_state();
            let (mouse_x, mouse_y) = input.mouse_position();
            eprintln!(
                "[trace-input] frame={} key_push=0x{:08X} key_on=0x{:08X} key_pull=0x{:08X} mouse_push=0x{:02X} mouse_on=0x{:02X} mouse_pull=0x{:02X} mouse=({}, {})",
                frame_index,
                input.raw_key_push(),
                input.raw_key_on(),
                input.raw_key_pull(),
                input.raw_mouse_push(),
                input.raw_mouse_on(),
                input.raw_mouse_pull(),
                mouse_x,
                mouse_y
            );
        }
        engine.handle_event(PalEvent::RedrawRequested);
        let frame = engine.update_with_delta(Duration::from_millis(config.frame_step_ms()))?;
        let event_count = frame
            .runtime_tick
            .as_ref()
            .map(|tick| tick.frame_events.len())
            .unwrap_or_default();
        eprintln!(
            "[headless] frame={} clock={} dt={}ms elapsed={}ms status={} textures={} draw_commands={} events={} clear={:?}",
            frame_index,
            frame.timing.frame_index,
            frame.timing.delta.as_millis(),
            frame.timing.elapsed.as_millis(),
            frame.runtime_status,
            frame.scene.textures.len(),
            frame.scene.commands.len(),
            event_count,
            frame.scene.clear_color
        );
        for snapshot in &config.diagnostic_png_at {
            if snapshot.frame == frame_index
                || (snapshot.frame == config.diagnostic_frames
                    && frame_index.saturating_add(1) == config.diagnostic_frames)
            {
                write_scene_png(&frame.scene, &snapshot.path)?;
                eprintln!(
                    "[headless] wrote diagnostic png-at frame={} {}",
                    frame_index,
                    snapshot.path.display()
                );
            }
        }
        let terminal = runtime_status_is_terminal(&frame.runtime_status);
        last_frame = Some(frame);
        if terminal {
            eprintln!(
                "[headless] stop: terminal runtime status {}",
                last_frame
                    .as_ref()
                    .expect("frame was just stored")
                    .runtime_status
            );
            break;
        }
    }
    if let (Some(path), Some(frame)) = (&config.diagnostic_png, last_frame.as_ref()) {
        write_scene_png(&frame.scene, path)?;
        eprintln!("[headless] wrote diagnostic png {}", path.display());
    }
    Ok(())
}

fn runtime_status_is_terminal(status: &RuntimeStatus) -> bool {
    matches!(
        status,
        RuntimeStatus::Halted { .. }
            | RuntimeStatus::UnsupportedCommand { .. }
            | RuntimeStatus::UnsupportedExtCall { .. }
            | RuntimeStatus::Faulted { .. }
    )
}

struct SenaApplication {
    config: SenaConfig,
    engine: Engine,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    diagnostic_frame_index: usize,
    last_auto_advance_press: Option<usize>,
    click_when_hit_done: Vec<bool>,
    click_when_hit_release: Option<usize>,
    next_redraw_at: Instant,
}

impl SenaApplication {
    fn new(config: SenaConfig) -> anyhow::Result<Self> {
        let engine = build_engine(&config)?;
        if config.print_loaded_assets {
            print_asset_summary(&engine);
        }
        let click_when_hit_len = config.diagnostic_click_when_hit_enabled.len();
        Ok(Self {
            config,
            engine,
            window: None,
            renderer: None,
            diagnostic_frame_index: 0,
            last_auto_advance_press: None,
            click_when_hit_done: vec![false; click_when_hit_len],
            click_when_hit_release: None,
            next_redraw_at: Instant::now(),
        })
    }

    fn window_diagnostic_active(&self) -> bool {
        !self.config.window_dump_frame_at.is_empty()
            || !self.config.diagnostic_clicks.is_empty()
            || !self.config.diagnostic_click_when_hit_enabled.is_empty()
            || !self.config.diagnostic_key_events.is_empty()
            || self.config.diagnostic_auto_advance.is_some()
    }

    fn window_dump_path_for_frame(&self, frame_index: usize) -> Option<&std::path::Path> {
        self.config
            .window_dump_frame_at
            .iter()
            .find(|snapshot| {
                snapshot.frame == frame_index
                    || (snapshot.frame == self.config.diagnostic_frames
                        && frame_index.saturating_add(1) == self.config.diagnostic_frames)
            })
            .map(|snapshot| snapshot.path.as_path())
    }

    fn create_window_and_renderer(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (width, height) = if self.config.prefer_config_window_size {
            self.engine
                .window_size_from_config(self.config.width, self.config.height)
        } else {
            (self.config.width, self.config.height)
        };
        let attrs = Window::default_attributes()
            .with_title(self.config.title.clone())
            .with_inner_size(LogicalSize::new(width as f64, height as f64));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("sena failed to create a winit window"),
        );
        let actual_size = window.inner_size();
        self.engine.handle_event(PalEvent::Resized {
            width: actual_size.width,
            height: actual_size.height,
        });
        let mut renderer_config = self.config.renderer;
        renderer_config.virtual_width = width.max(1);
        renderer_config.virtual_height = height.max(1);
        let renderer = pollster::block_on(Renderer::new(
            window.clone(),
            event_loop.owned_display_handle(),
            renderer_config,
        ))
        .expect("sena failed to initialize the software renderer");
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.engine.handle_event(PalEvent::Resized {
            width: size.width,
            height: size.height,
        });
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(size);
        }
    }

    fn inject_diagnostic_inputs(&mut self) {
        let frame_index = self.diagnostic_frame_index;
        if self.click_when_hit_release == Some(frame_index) {
            self.engine
                .handle_event(PalEvent::Input(InputEvent::MouseInput {
                    button: MouseButton::Left,
                    pressed: false,
                }));
            self.click_when_hit_release = None;
            eprintln!("[window-diagnostic] injected click-when-hit release frame={frame_index}");
        }
        for click in &self.config.diagnostic_clicks {
            if frame_index == click.frame {
                let (x, y) = self.diagnostic_logical_to_window_input(click.x, click.y);
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::CursorMoved { x, y }));
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::MouseInput {
                        button: MouseButton::Left,
                        pressed: true,
                    }));
                eprintln!(
                    "[window-diagnostic] injected click press frame={} logical=({}, {}) window_input=({:.3}, {:.3})",
                    frame_index, click.x, click.y, x, y
                );
            } else if frame_index == click.frame.saturating_add(1) {
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::MouseInput {
                        button: MouseButton::Left,
                        pressed: false,
                    }));
                eprintln!("[window-diagnostic] injected click release frame={frame_index}");
            }
        }
        self.inject_window_click_when_hit_enabled(frame_index);
        self.inject_window_auto_advance(frame_index);
        for key_event in &self.config.diagnostic_key_events {
            if frame_index == key_event.frame {
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::Keyboard {
                        key_name: key_event.key.clone(),
                        pressed: key_event.pressed,
                    }));
                eprintln!(
                    "[window-diagnostic] injected key {} frame={} key={}",
                    if key_event.pressed { "down" } else { "up" },
                    frame_index,
                    key_event.key
                );
            }
        }
        if self.config.trace_input {
            let input = self.engine.input_state();
            let (mouse_x, mouse_y) = input.mouse_position();
            eprintln!(
                "[trace-input] window_frame={} key_push=0x{:08X} key_on=0x{:08X} key_pull=0x{:08X} mouse_push=0x{:02X} mouse_on=0x{:02X} mouse_pull=0x{:02X} mouse=({}, {})",
                frame_index,
                input.raw_key_push(),
                input.raw_key_on(),
                input.raw_key_pull(),
                input.raw_mouse_push(),
                input.raw_mouse_on(),
                input.raw_mouse_pull(),
                mouse_x,
                mouse_y
            );
        }
    }

    fn inject_window_click_when_hit_enabled(&mut self, frame_index: usize) {
        for index in 0..self.config.diagnostic_click_when_hit_enabled.len() {
            if self
                .click_when_hit_done
                .get(index)
                .copied()
                .unwrap_or(false)
            {
                continue;
            }
            let click = self.config.diagnostic_click_when_hit_enabled[index];
            let Some(hit) = self.engine.diagnostic_button_hit_enabled(click.x, click.y) else {
                continue;
            };
            let (x, y) = self.diagnostic_logical_to_window_input(click.x, click.y);
            self.engine
                .handle_event(PalEvent::Input(InputEvent::CursorMoved { x, y }));
            self.engine
                .handle_event(PalEvent::Input(InputEvent::MouseInput {
                    button: MouseButton::Left,
                    pressed: true,
                }));
            self.click_when_hit_done[index] = true;
            self.click_when_hit_release = Some(frame_index.saturating_add(1));
            eprintln!(
                "[window-diagnostic] injected click-when-hit press frame={} logical=({}, {}) window_input=({:.3}, {:.3}) hit={hit:?}",
                frame_index, click.x, click.y, x, y
            );
        }
    }

    fn inject_window_auto_advance(&mut self, frame_index: usize) {
        let Some(auto) = self.config.diagnostic_auto_advance else {
            return;
        };
        if self.last_auto_advance_press == Some(frame_index.saturating_sub(1)) {
            self.engine
                .handle_event(PalEvent::Input(InputEvent::MouseInput {
                    button: MouseButton::Left,
                    pressed: false,
                }));
            eprintln!("[window-diagnostic] injected auto-advance release frame={frame_index}");
            return;
        }
        let min_frames = auto.min_frames.max(1);
        if frame_index == 0 || frame_index % min_frames != 0 {
            return;
        }
        if !matches!(
            self.engine.runtime_status(),
            RuntimeStatus::WaitClick { .. }
        ) {
            return;
        }
        let (x, y) = self.diagnostic_logical_to_window_input(auto.x, auto.y);
        self.engine
            .handle_event(PalEvent::Input(InputEvent::CursorMoved { x, y }));
        self.engine
            .handle_event(PalEvent::Input(InputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            }));
        self.last_auto_advance_press = Some(frame_index);
        eprintln!(
            "[window-diagnostic] injected auto-advance press frame={} logical=({}, {}) window_input=({:.3}, {:.3})",
            frame_index, auto.x, auto.y, x, y
        );
    }

    fn diagnostic_logical_to_window_input(&self, x: i32, y: i32) -> (f64, f64) {
        let Some(window) = self.window.as_ref() else {
            return (x as f64, y as f64);
        };
        let size = window.inner_size();
        let (logical_width, logical_height) = self.engine.logical_size_from_config(
            FrameScene::PAL_DEFAULT_WIDTH,
            FrameScene::PAL_DEFAULT_HEIGHT,
        );
        (
            x as f64 * size.width.max(1) as f64 / logical_width.max(1) as f64,
            y as f64 * size.height.max(1) as f64 / logical_height.max(1) as f64,
        )
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_diagnostic_active() {
            self.render_window_diagnostic_batch(event_loop);
            return;
        }
        self.engine.handle_event(PalEvent::RedrawRequested);
        let frame = match self.update_engine_frame() {
            Ok(frame) => frame,
            Err(err) => {
                // Log the error but keep the window open so the user can see what happened.
                log::error!("sena engine update failed: {err}");
                self.engine.input_begin_frame();
                return;
            }
        };
        let Some(renderer) = self.renderer.as_mut() else {
            self.engine.input_begin_frame();
            return;
        };
        match renderer.render(&frame.scene) {
            RenderOutcome::Rendered => {}
            RenderOutcome::Skipped => {}
            RenderOutcome::Reconfigured => log::debug!("software surface was reconfigured"),
        }
        self.engine.input_begin_frame();
    }

    fn render_window_diagnostic_batch(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_none() {
            event_loop.exit();
            return;
        }
        while self.diagnostic_frame_index < self.config.diagnostic_frames {
            let frame_index = self.diagnostic_frame_index;
            let dump_path = self
                .window_dump_path_for_frame(frame_index)
                .map(PathBuf::from);
            let is_last = frame_index.saturating_add(1) >= self.config.diagnostic_frames;
            let should_render = dump_path.is_some() || is_last;

            self.inject_diagnostic_inputs();
            self.engine.handle_event(PalEvent::RedrawRequested);
            let frame = match self
                .engine
                .update_with_delta(Duration::from_millis(self.config.frame_step_ms()))
            {
                Ok(frame) => frame,
                Err(err) => {
                    log::error!("sena engine update failed: {err}");
                    self.engine.input_begin_frame();
                    event_loop.exit();
                    return;
                }
            };

            if should_render {
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.render_with_png_dump(&frame.scene, dump_path.as_deref()) {
                        RenderOutcome::Rendered => {}
                        RenderOutcome::Skipped => {}
                        RenderOutcome::Reconfigured => {
                            log::debug!("software surface was reconfigured")
                        }
                    }
                }
            }
            if let Some(path) = dump_path.as_deref() {
                eprintln!(
                    "[window-diagnostic] wrote window frame dump frame={} {}",
                    frame_index,
                    path.display()
                );
            }

            let event_count = frame
                .runtime_tick
                .as_ref()
                .map(|tick| tick.frame_events.len())
                .unwrap_or_default();
            eprintln!(
                "[window-diagnostic] frame={} clock={} dt={}ms elapsed={}ms status={} textures={} draw_commands={} events={} clear={:?}",
                frame_index,
                frame.timing.frame_index,
                frame.timing.delta.as_millis(),
                frame.timing.elapsed.as_millis(),
                frame.runtime_status,
                frame.scene.textures.len(),
                frame.scene.commands.len(),
                event_count,
                frame.scene.clear_color
            );

            let terminal = runtime_status_is_terminal(&frame.runtime_status);
            self.engine.input_begin_frame();
            self.diagnostic_frame_index = self.diagnostic_frame_index.saturating_add(1);
            if terminal {
                break;
            }
        }
        event_loop.exit();
    }

    #[allow(dead_code)]
    fn render_single_window_diagnostic_frame(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_diagnostic_active() {
            self.inject_diagnostic_inputs();
        }
        self.engine.handle_event(PalEvent::RedrawRequested);
        let frame = if self.window_diagnostic_active() {
            self.engine
                .update_with_delta(Duration::from_millis(self.config.frame_step_ms()))
        } else {
            self.update_engine_frame()
        };
        let frame = match frame {
            Ok(frame) => frame,
            Err(err) => {
                // Log the error but keep the window open so the user can see what happened.
                log::error!("sena engine update failed: {err}");
                self.engine.input_begin_frame();
                return;
            }
        };
        let Some(renderer) = self.renderer.as_mut() else {
            self.engine.input_begin_frame();
            return;
        };
        let dump_path = self
            .config
            .window_dump_frame_at
            .iter()
            .find(|snapshot| snapshot.frame == self.diagnostic_frame_index)
            .map(|snapshot| snapshot.path.as_path());
        match renderer.render_with_png_dump(&frame.scene, dump_path) {
            RenderOutcome::Rendered => {}
            RenderOutcome::Skipped => {}
            RenderOutcome::Reconfigured => log::debug!("software surface was reconfigured"),
        }
        if let Some(path) = dump_path {
            eprintln!(
                "[window-diagnostic] wrote window frame dump frame={} {}",
                self.diagnostic_frame_index,
                path.display()
            );
        }
        if self.window_diagnostic_active() {
            let event_count = frame
                .runtime_tick
                .as_ref()
                .map(|tick| tick.frame_events.len())
                .unwrap_or_default();
            eprintln!(
                "[window-diagnostic] frame={} clock={} dt={}ms elapsed={}ms status={} textures={} draw_commands={} events={} clear={:?}",
                self.diagnostic_frame_index,
                frame.timing.frame_index,
                frame.timing.delta.as_millis(),
                frame.timing.elapsed.as_millis(),
                frame.runtime_status,
                frame.scene.textures.len(),
                frame.scene.commands.len(),
                event_count,
                frame.scene.clear_color
            );
        }
        self.engine.input_begin_frame();
        self.diagnostic_frame_index = self.diagnostic_frame_index.saturating_add(1);
        if self.window_diagnostic_active()
            && self.diagnostic_frame_index >= self.config.diagnostic_frames
        {
            event_loop.exit();
        }
    }

    fn update_engine_frame(&mut self) -> anyhow::Result<crate::engine::EngineFrame> {
        match self.config.fixed_timestep_ms {
            Some(ms) => self
                .engine
                .update_with_delta(Duration::from_millis(ms.max(1))),
            None => self
                .engine
                .update_with_delta(Duration::from_millis(self.config.frame_step_ms())),
        }
    }

    fn trace_window_input_state(&self, label: &str) {
        if !self.config.trace_input {
            return;
        }
        let input = self.engine.input_state();
        let (mouse_x, mouse_y) = input.mouse_position();
        let Some(window) = self.window.as_ref() else {
            eprintln!(
                "[trace-input-event] {label} no-window pal=({mouse_x},{mouse_y}) key_push=0x{:08X} key_on=0x{:08X} mouse_push=0x{:02X} mouse_on=0x{:02X}",
                input.raw_key_push(),
                input.raw_key_on(),
                input.raw_mouse_push(),
                input.raw_mouse_on()
            );
            return;
        };
        let size = window.inner_size();
        let (logical_width, logical_height) = self.engine.logical_size_from_config(
            FrameScene::PAL_DEFAULT_WIDTH,
            FrameScene::PAL_DEFAULT_HEIGHT,
        );
        eprintln!(
            "[trace-input-event] {label} frame={} window={}x{} scale_factor={:.3} logical={}x{} pal=({mouse_x},{mouse_y}) key_push=0x{:08X} key_on=0x{:08X} key_pull=0x{:08X} mouse_push=0x{:02X} mouse_on=0x{:02X} mouse_pull=0x{:02X}",
            self.diagnostic_frame_index,
            size.width,
            size.height,
            window.scale_factor(),
            logical_width,
            logical_height,
            input.raw_key_push(),
            input.raw_key_on(),
            input.raw_key_pull(),
            input.raw_mouse_push(),
            input.raw_mouse_on(),
            input.raw_mouse_pull(),
        );
    }
}

impl SenaConfig {
    fn frame_step_ms(&self) -> u64 {
        self.fixed_timestep_ms
            .unwrap_or(self.diagnostic_frame_ms)
            .max(1)
    }
}

pub(crate) fn build_engine(config: &SenaConfig) -> anyhow::Result<Engine> {
    Engine::new(EngineConfig {
        game_root: config.game_root.clone(),
        nls: config.nls,
        script_runtime: ScriptRuntimeConfig {
            instructions_per_frame: config.script_budget_per_frame,
            trace: config.trace_script,
        },
        audio: config.audio.clone(),
        trace: TraceConfig {
            script: config.trace_script,
            scene: config.trace_scene,
            sprites: config.trace_sprites,
            assets: config.trace_assets,
            render: config.trace_render,
            buttons: config.trace_buttons,
            input: config.trace_input,
            text: config.trace_text,
            actions: config.trace_actions,
        },
    })
}

fn inject_headless_auto_advance(
    engine: &mut Engine,
    config: &SenaConfig,
    frame_index: usize,
    last_press: &mut Option<usize>,
) {
    let Some(auto) = config.diagnostic_auto_advance else {
        return;
    };
    if *last_press == Some(frame_index.saturating_sub(1)) {
        engine.handle_event(PalEvent::Input(InputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: false,
        }));
        eprintln!("[headless] injected auto-advance release frame={frame_index}");
        return;
    }
    let min_frames = auto.min_frames.max(1);
    if frame_index == 0 || frame_index % min_frames != 0 {
        return;
    }
    if !matches!(engine.runtime_status(), RuntimeStatus::WaitClick { .. }) {
        return;
    }
    engine.handle_event(PalEvent::Input(InputEvent::CursorMoved {
        x: auto.x as f64,
        y: auto.y as f64,
    }));
    engine.handle_event(PalEvent::Input(InputEvent::MouseInput {
        button: MouseButton::Left,
        pressed: true,
    }));
    *last_press = Some(frame_index);
    eprintln!(
        "[headless] injected auto-advance press frame={} pos=({}, {})",
        frame_index, auto.x, auto.y
    );
}

fn inject_headless_click_when_hit_enabled(
    engine: &mut Engine,
    config: &SenaConfig,
    frame_index: usize,
    done: &mut [bool],
    release_frame: &mut Option<usize>,
) {
    for (index, click) in config
        .diagnostic_click_when_hit_enabled
        .iter()
        .copied()
        .enumerate()
    {
        if done.get(index).copied().unwrap_or(false) {
            continue;
        }
        let Some(hit) = engine.diagnostic_button_hit_enabled(click.x, click.y) else {
            continue;
        };
        engine.handle_event(PalEvent::Input(InputEvent::CursorMoved {
            x: click.x as f64,
            y: click.y as f64,
        }));
        engine.handle_event(PalEvent::Input(InputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        }));
        done[index] = true;
        *release_frame = Some(frame_index.saturating_add(1));
        eprintln!(
            "[headless] injected click-when-hit press frame={} pos=({}, {}) hit={hit:?}",
            frame_index, click.x, click.y
        );
    }
}

fn write_scene_png(scene: &FrameScene, path: &PathBuf) -> anyhow::Result<()> {
    let width = scene.logical_width.max(1);
    let height = scene.logical_height.max(1);
    let pixels = rasterize_scene_rgba(scene);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(path)?;
    let writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;
    Ok(())
}

impl ApplicationHandler for SenaApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window_and_renderer(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.engine.handle_event(PalEvent::CloseRequested);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => self.handle_resize(size),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if self.config.trace_input {
                    eprintln!(
                        "[trace-input-event] raw scale_factor_changed scale_factor={scale_factor:.3}"
                    );
                }
                self.engine
                    .handle_event(PalEvent::ScaleFactorChanged { scale_factor });
                // A display move can change the backing pixel count without a
                // separate resize event. Keep the software surface and PAL
                // input mapping in the window's current physical pixel space.
                self.handle_resize(window.inner_size());
            }
            WindowEvent::RedrawRequested => self.render(event_loop),
            WindowEvent::CursorMoved { position, .. } => {
                if self.config.trace_input {
                    let size = window.inner_size();
                    eprintln!(
                        "[trace-input-event] raw cursor_moved pos=({:.3},{:.3}) window={}x{} scale_factor={:.3}",
                        position.x,
                        position.y,
                        size.width,
                        size.height,
                        window.scale_factor()
                    );
                }
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::CursorMoved {
                        x: position.x,
                        y: position.y,
                    }));
                self.trace_window_input_state("after cursor_moved");
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if self.config.trace_input {
                    eprintln!(
                        "[trace-input-event] raw mouse_input button={button:?} state={state:?}"
                    );
                }
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::MouseInput {
                        button: map_mouse_button(button),
                        pressed: state == ElementState::Pressed,
                    }));
                self.trace_window_input_state("after mouse_input");
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::MouseWheel { delta_x, delta_y }));
                self.trace_window_input_state("after mouse_wheel");
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                let key_name = key_name(&event.logical_key);
                if self.config.trace_input {
                    eprintln!(
                        "[trace-input-event] raw keyboard key={:?} key_name={} state={:?} repeat={}",
                        event.logical_key,
                        key_name,
                        event.state,
                        event.repeat
                    );
                }
                self.engine
                    .handle_event(PalEvent::Input(InputEvent::Keyboard { key_name, pressed }));
                self.trace_window_input_state("after keyboard");
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if now >= self.next_redraw_at {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            self.next_redraw_at = now + Duration::from_millis(self.config.frame_step_ms());
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_redraw_at));
        if self.window_diagnostic_active() {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        } else if self.window.is_none() {
            // Ask winit to wake once the window has been created after resume.
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

fn map_mouse_button(button: WinitMouseButton) -> MouseButton {
    match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        WinitMouseButton::Other(value) => MouseButton::Other(value),
    }
}

fn key_name(key: &Key) -> String {
    match key {
        Key::Named(named) => format!("{named:?}"),
        Key::Character(text) => text.to_string(),
        Key::Unidentified(_) => "Unidentified".to_owned(),
        Key::Dead(ch) => match ch {
            Some(ch) => format!("Dead({ch})"),
            None => "Dead".to_owned(),
        },
    }
}

fn print_asset_summary(engine: &Engine) {
    let Some(assets) = engine.core_assets() else {
        println!("no game root was provided; core assets were not loaded");
        return;
    };
    println!(
        "Script.src: 0x{:X} bytes, check=0x{:08X}, entry=0x{:08X}",
        assets.script.bytes.len(),
        assets.script_check_value,
        assets.script_entry_pc
    );
    println!("File.dat:   0x{:X} bytes", assets.file_dat.bytes.len());
    println!("Text.dat:   0x{:X} bytes", assets.text_dat.bytes.len());
    println!("Mem.dat:    0x{:X} bytes", assets.mem_dat.bytes.len());
    println!(
        "Point.dat:  0x{:X} bytes, points={}",
        assets.point_dat.bytes.len(),
        assets.point_table.len()
    );
    println!("Runtime:    {}", engine.runtime_status());
    match assets.graphic_index.as_ref() {
        Some(index) => println!("graphic.dat: entries={}", index.len()),
        None => println!("graphic.dat: not present"),
    }
}
