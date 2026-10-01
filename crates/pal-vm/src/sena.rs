use std::path::PathBuf;

use clap::Parser;
use pal_asset::Nls;
use pal_vm::{
    run_sena, AudioConfig, DiagnosticAutoAdvance, DiagnosticClick, DiagnosticClickWhenHitEnabled,
    DiagnosticKeyEvent, DiagnosticPngAt, DiagnosticWheelEvent, FrameScene, MouseButton,
    ScriptRuntimeConfig, SenaConfig,
};

#[derive(Debug, Parser)]
#[command(name = "sena")]
#[command(about = "PAL engine launcher")]
struct Args {
    #[arg(value_name = "GAME_ROOT")]
    game_root: Option<PathBuf>,

    #[arg(long, default_value = "sjis")]
    nls: Nls,

    #[arg(long, default_value_t = FrameScene::PAL_DEFAULT_WIDTH)]
    width: u32,

    #[arg(long, default_value_t = FrameScene::PAL_DEFAULT_HEIGHT)]
    height: u32,

    #[arg(long, default_value = "sena")]
    title: String,

    #[arg(long)]
    ignore_config_window_size: bool,

    #[arg(long)]
    print_loaded_assets: bool,

    /// Trace every script opcode and extcall (verbose).
    #[arg(long)]
    trace_script: bool,

    /// Trace scene composition (clear color, command count) each frame.
    #[arg(long)]
    trace_scene: bool,

    /// Trace sprite state (position, visibility, source rect) each frame.
    #[arg(long)]
    trace_sprites: bool,

    /// Trace asset open / decode / upload events.
    #[arg(long)]
    trace_assets: bool,

    /// Trace draw command submission each frame.
    #[arg(long)]
    trace_render: bool,

    /// Trace button state (group, index, name, rect, enabled, visible, hit) every frame.
    #[arg(long)]
    trace_buttons: bool,

    /// Trace input masks every headless diagnostic frame.
    #[arg(long)]
    trace_input: bool,

    /// Accepted compatibility flag; text trace logs are emitted through RUST_LOG.
    #[arg(long)]
    trace_text: bool,

    /// Accepted compatibility flag; action trace logs are emitted through RUST_LOG.
    #[arg(long)]
    trace_actions: bool,

    /// Accepted compatibility flag; extcall trace logs are emitted by --trace-script.
    #[arg(long)]
    trace_extcalls: bool,

    #[arg(long, default_value_t = ScriptRuntimeConfig::default().instructions_per_frame)]
    script_budget_per_frame: usize,

    #[arg(long, default_value_t = false)]
    no_audio: bool,

    /// Run the engine without creating a window or GPU renderer.
    #[arg(long, alias = "handless", alias = "gui-handless")]
    headless: bool,

    /// Number of frames to execute in --headless diagnostics.
    #[arg(long, default_value_t = 120)]
    diagnostic_frames: usize,

    /// Real-time delay between --headless diagnostic frames.
    #[arg(long, default_value_t = 16)]
    diagnostic_frame_ms: u64,

    /// Use a fixed VM timestep in both GUI and headless modes.
    #[arg(long)]
    fixed_timestep_ms: Option<u64>,

    /// Write the final --headless diagnostic frame to a PNG file.
    #[arg(long)]
    diagnostic_png: Option<PathBuf>,

    /// Write a --headless diagnostic frame to a PNG path as frame:path. Can be passed more than once.
    #[arg(long, value_parser = parse_diagnostic_png_at)]
    diagnostic_png_at: Vec<DiagnosticPngAt>,

    /// Dump the normal window renderer's physical surface to PNG as frame:path.
    #[arg(long, value_parser = parse_diagnostic_png_at)]
    window_dump_frame_at: Vec<DiagnosticPngAt>,

    /// Inject headless clicks as frame:x:y in PAL logical coordinates. Can be passed more than once.
    #[arg(long, value_parser = parse_diagnostic_click)]
    diagnostic_click: Vec<DiagnosticClick>,

    /// Inject headless right-button clicks as frame:x:y in PAL logical coordinates.
    #[arg(long, value_parser = parse_diagnostic_right_click)]
    diagnostic_right_click: Vec<DiagnosticClick>,

    /// Inject one click at x:y once that logical point hits an enabled button.
    #[arg(long, value_parser = parse_diagnostic_click_when_hit_enabled)]
    diagnostic_click_when_hit_enabled: Vec<DiagnosticClickWhenHitEnabled>,

    /// Inject a keyboard down edge as frame:key. Can be passed more than once.
    #[arg(long, value_parser = parse_diagnostic_key_down)]
    diagnostic_key_down: Vec<DiagnosticKeyEvent>,

    /// Inject a keyboard up edge as frame:key. Can be passed more than once.
    #[arg(long, value_parser = parse_diagnostic_key_up)]
    diagnostic_key_up: Vec<DiagnosticKeyEvent>,

    /// Inject a held keyboard key as start:end:key. Can be passed more than once.
    #[arg(long, value_parser = parse_diagnostic_key_hold)]
    diagnostic_key_hold: Vec<(DiagnosticKeyEvent, DiagnosticKeyEvent)>,

    /// Inject a mouse wheel step as frame:delta_y. Can be passed more than once.
    #[arg(long, value_parser = parse_diagnostic_wheel)]
    diagnostic_wheel: Vec<DiagnosticWheelEvent>,

    /// Inject repeated diagnostic clicks to advance wait-click/text paths.
    #[arg(long)]
    diagnostic_auto_advance: bool,

    /// Logical PAL click position for --diagnostic-auto-advance as x:y.
    #[arg(long, value_parser = parse_diagnostic_auto_advance_click, default_value = "640:640")]
    diagnostic_auto_advance_click: (i32, i32),

    /// Minimum frame interval between diagnostic auto-advance click presses.
    #[arg(long, default_value_t = 8)]
    diagnostic_auto_advance_min_frames: usize,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let diagnostic_key_events = diagnostic_key_events(&args);
    let diagnostic_auto_advance = args
        .diagnostic_auto_advance
        .then_some(DiagnosticAutoAdvance {
            x: args.diagnostic_auto_advance_click.0,
            y: args.diagnostic_auto_advance_click.1,
            min_frames: args.diagnostic_auto_advance_min_frames.max(1),
        });
    let trace_script = args.trace_script || args.trace_extcalls;
    run_sena(SenaConfig {
        game_root: args.game_root,
        nls: args.nls,
        title: args.title,
        width: args.width,
        height: args.height,
        prefer_config_window_size: !args.ignore_config_window_size,
        print_loaded_assets: args.print_loaded_assets,
        trace_script,
        trace_scene: args.trace_scene,
        trace_sprites: args.trace_sprites,
        trace_assets: args.trace_assets,
        trace_render: args.trace_render,
        trace_buttons: args.trace_buttons,
        trace_input: args.trace_input,
        trace_text: args.trace_text,
        trace_actions: args.trace_actions,
        script_budget_per_frame: args.script_budget_per_frame,
        audio: AudioConfig {
            enabled: !args.no_audio,
        },
        headless: args.headless,
        diagnostic_frames: args.diagnostic_frames,
        diagnostic_frame_ms: args.diagnostic_frame_ms,
        fixed_timestep_ms: args.fixed_timestep_ms,
        diagnostic_png: args.diagnostic_png,
        diagnostic_png_at: args.diagnostic_png_at,
        window_dump_frame_at: args.window_dump_frame_at,
        diagnostic_clicks: args
            .diagnostic_click
            .iter()
            .copied()
            .chain(args.diagnostic_right_click.iter().copied())
            .collect(),
        diagnostic_click_when_hit_enabled: args.diagnostic_click_when_hit_enabled,
        diagnostic_key_events,
        diagnostic_wheel_events: args.diagnostic_wheel,
        diagnostic_auto_advance,
        ..SenaConfig::default()
    })
}

fn diagnostic_key_events(args: &Args) -> Vec<DiagnosticKeyEvent> {
    let mut events = Vec::new();
    events.extend(args.diagnostic_key_down.iter().cloned());
    events.extend(args.diagnostic_key_up.iter().cloned());
    for (down, up) in &args.diagnostic_key_hold {
        events.push(down.clone());
        events.push(up.clone());
    }
    events.sort_by_key(|event| (event.frame, !event.pressed));
    events
}

fn parse_diagnostic_click(raw: &str) -> Result<DiagnosticClick, String> {
    let mut parts = raw.split(':');
    let frame = parts
        .next()
        .ok_or_else(|| "missing frame".to_owned())?
        .parse::<usize>()
        .map_err(|err| format!("invalid frame: {err}"))?;
    let x = parts
        .next()
        .ok_or_else(|| "missing x".to_owned())?
        .parse::<i32>()
        .map_err(|err| format!("invalid x: {err}"))?;
    let y = parts
        .next()
        .ok_or_else(|| "missing y".to_owned())?
        .parse::<i32>()
        .map_err(|err| format!("invalid y: {err}"))?;
    if parts.next().is_some() {
        return Err("expected frame:x:y".to_owned());
    }
    Ok(DiagnosticClick {
        frame,
        x,
        y,
        button: MouseButton::Left,
    })
}

fn parse_diagnostic_right_click(raw: &str) -> Result<DiagnosticClick, String> {
    let mut click = parse_diagnostic_click(raw)?;
    click.button = MouseButton::Right;
    Ok(click)
}

fn parse_diagnostic_click_when_hit_enabled(
    raw: &str,
) -> Result<DiagnosticClickWhenHitEnabled, String> {
    let mut parts = raw.split(':');
    let x = parts
        .next()
        .ok_or_else(|| "missing x".to_owned())?
        .parse::<i32>()
        .map_err(|err| format!("invalid x: {err}"))?;
    let y = parts
        .next()
        .ok_or_else(|| "missing y".to_owned())?
        .parse::<i32>()
        .map_err(|err| format!("invalid y: {err}"))?;
    if parts.next().is_some() {
        return Err("expected x:y".to_owned());
    }
    Ok(DiagnosticClickWhenHitEnabled { x, y })
}

fn parse_diagnostic_key_down(raw: &str) -> Result<DiagnosticKeyEvent, String> {
    parse_diagnostic_key_event(raw, true)
}

fn parse_diagnostic_key_up(raw: &str) -> Result<DiagnosticKeyEvent, String> {
    parse_diagnostic_key_event(raw, false)
}

fn parse_diagnostic_key_event(raw: &str, pressed: bool) -> Result<DiagnosticKeyEvent, String> {
    let mut parts = raw.split(':');
    let frame = parts
        .next()
        .ok_or_else(|| "missing frame".to_owned())?
        .parse::<usize>()
        .map_err(|err| format!("invalid frame: {err}"))?;
    let key = normalize_diagnostic_key(parts.next().ok_or_else(|| "missing key".to_owned())?);
    if parts.next().is_some() {
        return Err("expected frame:key".to_owned());
    }
    Ok(DiagnosticKeyEvent {
        frame,
        key,
        pressed,
    })
}

fn parse_diagnostic_key_hold(
    raw: &str,
) -> Result<(DiagnosticKeyEvent, DiagnosticKeyEvent), String> {
    let mut parts = raw.split(':');
    let start = parts
        .next()
        .ok_or_else(|| "missing start frame".to_owned())?
        .parse::<usize>()
        .map_err(|err| format!("invalid start frame: {err}"))?;
    let end = parts
        .next()
        .ok_or_else(|| "missing end frame".to_owned())?
        .parse::<usize>()
        .map_err(|err| format!("invalid end frame: {err}"))?;
    let key = normalize_diagnostic_key(parts.next().ok_or_else(|| "missing key".to_owned())?);
    if parts.next().is_some() {
        return Err("expected start:end:key".to_owned());
    }
    if end < start {
        return Err("end frame must be >= start frame".to_owned());
    }
    Ok((
        DiagnosticKeyEvent {
            frame: start,
            key: key.clone(),
            pressed: true,
        },
        DiagnosticKeyEvent {
            frame: end,
            key,
            pressed: false,
        },
    ))
}

fn parse_diagnostic_auto_advance_click(raw: &str) -> Result<(i32, i32), String> {
    let mut parts = raw.split(':');
    let x = parts
        .next()
        .ok_or_else(|| "missing x".to_owned())?
        .parse::<i32>()
        .map_err(|err| format!("invalid x: {err}"))?;
    let y = parts
        .next()
        .ok_or_else(|| "missing y".to_owned())?
        .parse::<i32>()
        .map_err(|err| format!("invalid y: {err}"))?;
    if parts.next().is_some() {
        return Err("expected x:y".to_owned());
    }
    Ok((x, y))
}

fn parse_diagnostic_png_at(raw: &str) -> Result<DiagnosticPngAt, String> {
    let Some((frame, path)) = raw.split_once(':') else {
        return Err("expected frame:path".to_owned());
    };
    let frame = frame
        .parse::<usize>()
        .map_err(|err| format!("invalid frame: {err}"))?;
    if path.is_empty() {
        return Err("missing path".to_owned());
    }
    Ok(DiagnosticPngAt {
        frame,
        path: PathBuf::from(path),
    })
}

fn parse_diagnostic_wheel(raw: &str) -> Result<DiagnosticWheelEvent, String> {
    let mut parts = raw.split(':');
    let frame = parts
        .next()
        .ok_or_else(|| "missing frame".to_owned())?
        .parse::<usize>()
        .map_err(|err| format!("invalid frame: {err}"))?;
    let delta_y = parts
        .next()
        .ok_or_else(|| "missing delta_y".to_owned())?
        .parse::<f32>()
        .map_err(|err| format!("invalid delta_y: {err}"))?;
    if parts.next().is_some() {
        return Err("expected frame:delta_y".to_owned());
    }
    Ok(DiagnosticWheelEvent { frame, delta_y })
}

fn normalize_diagnostic_key(raw: &str) -> String {    match raw {
        "Ctrl" | "ctrl" | "CTRL" | "Control" | "control" => "Control".to_owned(),
        "LControl" | "LeftControl" | "ControlLeft" | "LeftCtrl" => "ControlLeft".to_owned(),
        "RControl" | "RightControl" | "ControlRight" | "RightCtrl" => "ControlRight".to_owned(),
        other => other.to_owned(),
    }
}
