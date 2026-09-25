use std::path::PathBuf;

use pal_asset::{AssetSource, LoadedAsset};
use pal_script::PointTable;
use pal_vm::{CoreAssets, FrameEvent, RuntimeStatus, ScriptRuntime, ScriptRuntimeConfig};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn asset(name: &str, bytes: Vec<u8>) -> LoadedAsset {
    LoadedAsset {
        name: name.to_owned(),
        bytes,
        source: AssetSource::Loose {
            path: PathBuf::from(name),
        },
    }
}

/// Build an instruction word: hi=1, lo=opcode.
fn opcode(op: u16) -> [u8; 4] {
    (0x0001_0000u32 | op as u32).to_le_bytes()
}

fn word(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

/// Operand: immediate value (kind 0x0).
fn imm(value: i32) -> u32 {
    value as u32
}

/// Operand: variable slot (kind 0x4).
fn var(slot: u16) -> u32 {
    0x4000_0000u32 | slot as u32
}

fn ext_raw(category: u16, index: u16) -> u32 {
    ((category as u32) << 16) | index as u32
}

fn dst_slot(slot: u16) -> u32 {
    slot as u32
}

/// Operand: ArgumentBase (kind 0x9, raw = 0x90000000).
fn arg_base() -> u32 {
    0x9000_0000u32
}

/// Operand: ArgumentStack lo index (kind 0x8).
fn arg_stack(lo: u16) -> u32 {
    0x8000_0000u32 | lo as u32
}

fn temp_mem(bank: u16, var_slot: u16) -> u32 {
    0x5000_0000u32 | ((bank as u32) << 16) | var_slot as u32
}

fn memdat_direct(bank: u16, var_slot: u16) -> u32 {
    0x6000_0000u32 | ((bank as u32) << 16) | var_slot as u32
}

fn memdat_indirect(bank: u16, var_slot: u16) -> u32 {
    0x7000_0000u32 | ((bank as u32) << 16) | var_slot as u32
}

/// Build a minimal CoreAssets from a script byte buffer.
/// The script header "Sv20" is prepended at offset 0; the real code starts at offset 8
/// (after the 4-byte magic + 4-byte entry-pc word).
fn assets_with_script(entry_pc: u32, script_body: Vec<u8>) -> CoreAssets {
    let mut script = Vec::new();
    // PAL script magic
    script.extend_from_slice(b"Sv20");
    // Placeholder for check value (offset 4..8 is NOT the entry pc; check ScriptImage::parse)
    // Actually the script image format: first 4 bytes = "Sv20", next 4 = check value,
    // next 4 = entry_pc offset. Let's look at how runtime_fixture does it:
    // In runtime_fixture: entry_pc = 12, script starts: "Sv20" + word(0) + word(12)
    // So offset 4 = check value = 0, offset 8 = entry_pc = 12.
    script.extend_from_slice(&0u32.to_le_bytes()); // check value
    script.extend_from_slice(&entry_pc.to_le_bytes()); // entry_pc
    script.extend_from_slice(&script_body);

    CoreAssets {
        script: asset("Script.src", script),
        file_dat: asset("File.dat", Vec::new()),
        text_dat: asset("Text.dat", Vec::new()),
        mem_dat: asset("Mem.dat", Vec::new()),
        point_dat: asset("Point.dat", Vec::new()),
        graphic_dat: None,
        extended_softpal: false,
        script_check_value: 0,
        script_entry_pc: entry_pc,
        point_table: PointTable::parse(&[]).expect("empty Point.dat should parse"),
        graphic_index: None,
    }
}

/// Run to completion (Halted or UnsupportedCommand or Faulted).
fn run_to_stop(runtime: &mut ScriptRuntime, assets: &CoreAssets, config: &ScriptRuntimeConfig) {
    for _ in 0..1000 {
        let tick = runtime
            .run_frame(assets, config)
            .expect("run_frame should not propagate error");
        match &tick.status {
            RuntimeStatus::Running { .. } => continue,
            RuntimeStatus::WaitFrame { .. } | RuntimeStatus::WaitClick { .. } => {
                // Resolve waits immediately for test purposes.
                runtime.resolve_pending_wait();
                continue;
            }
            _ => break,
        }
    }
}

// ---------------------------------------------------------------------------
// ArgumentBase tests
// ---------------------------------------------------------------------------

/// ArgumentBase (kind 0x9) reads as 0 on a freshly booted runtime.
#[test]
fn argument_base_reads_zero_initially() {
    // Script: mov var[0] = arg_base; halt
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(1)); // mov
    body.extend_from_slice(&word(var(0))); // dst = var[0]
    body.extend_from_slice(&word(arg_base())); // src = arg_base
    body.extend_from_slice(&opcode(21)); // halt

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);

    assert_eq!(runtime.vars()[0], 0, "argument_base should be 0 on boot");
}

/// Write to ArgumentBase then read it back via a variable.
#[test]
fn argument_base_write_read_roundtrip() {
    // Script: mov arg_base = imm(42); mov var[0] = arg_base; halt
    let entry_pc = 12u32;
    let mut body = Vec::new();
    // mov arg_base = 42
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(arg_base()));
    body.extend_from_slice(&word(imm(42)));
    // mov var[0] = arg_base
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(0)));
    body.extend_from_slice(&word(arg_base()));
    // halt
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);

    assert_eq!(runtime.vars()[0], 42, "argument_base roundtrip failed");
}

// ---------------------------------------------------------------------------
// ArgumentStack / pack_args tests
// ---------------------------------------------------------------------------

/// Push three values, pack them, then read them via arg_stack operand.
/// After pack_args, PAL's formal arg area exposes lo=1 as the first source
/// argument. Game.exe sub_42CAA0 pops the value stack top-first into the
/// argument array, then sub_42C910 resolves arg_stack[-N] as arg_top - N.
#[test]
fn pack_args_and_arg_stack_access() {
    // Push 10, 20, 30 onto stack; pack 3 args; read arg[1] → var[0], arg[2] → var[1], arg[3] → var[2]; halt
    let entry_pc = 12u32;
    let mut body = Vec::new();

    // push 10
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(10)));
    // push 20
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(20)));
    // push 30
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(30)));
    // pack_args 3
    body.extend_from_slice(&opcode(32));
    body.extend_from_slice(&word(imm(3)));

    // mov var[0] = arg_stack[1]  (native arg[-1], first source argument = 10)
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(0)));
    body.extend_from_slice(&word(arg_stack(1)));
    // mov var[1] = arg_stack[2]  (native arg[-2], middle = 20)
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(1)));
    body.extend_from_slice(&word(arg_stack(2)));
    // mov var[2] = arg_stack[3]  (native arg[-3], newest source argument = 30)
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(2)));
    body.extend_from_slice(&word(arg_stack(3)));
    // halt
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);

    assert_eq!(
        runtime.vars()[0],
        10,
        "arg_stack[1] should be native arg[-1] / first source argument (10)"
    );
    assert_eq!(
        runtime.vars()[1],
        20,
        "arg_stack[2] should be middle value (20)"
    );
    assert_eq!(
        runtime.vars()[2],
        30,
        "arg_stack[3] should be native arg[-3] / newest source argument (30)"
    );
}

/// pack_args then drop_args: argument stack depth returns to zero.
#[test]
fn pack_args_drop_args_cycle() {
    // Push 5 values; pack 5; drop 5; halt
    let entry_pc = 12u32;
    let mut body = Vec::new();
    for i in 0..5i32 {
        body.extend_from_slice(&opcode(31));
        body.extend_from_slice(&word(imm(i)));
    }
    body.extend_from_slice(&opcode(32));
    body.extend_from_slice(&word(imm(5)));
    body.extend_from_slice(&opcode(33));
    body.extend_from_slice(&word(imm(5)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);

    assert_eq!(
        runtime.argument_stack_depth(),
        0,
        "argument_stack should be empty after drop_args"
    );
    assert!(
        matches!(runtime.status(), RuntimeStatus::Halted { .. }),
        "runtime should be halted"
    );
}

// ---------------------------------------------------------------------------
// UnsupportedCommand sets faulted-like status without crashing
// ---------------------------------------------------------------------------

/// An unknown opcode (e.g. opcode 99) sets UnsupportedCommand status; the runtime stays alive.
#[test]
fn unsupported_command_sets_status_without_crash() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    // opcode 99 is not implemented
    body.extend_from_slice(&opcode(99));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);

    assert!(
        matches!(
            runtime.status(),
            RuntimeStatus::UnsupportedCommand { opcode: 99, .. }
        ),
        "expected UnsupportedCommand(99), got {:?}",
        runtime.status()
    );
}

// ---------------------------------------------------------------------------
// WaitClick creates task and runtime stays in WaitClick
// ---------------------------------------------------------------------------

/// Opcode 253 (WaitClick) emits a WaitRequest and leaves the runtime in WaitClick status.
/// The window must NOT close on this — it's a normal blocking state.
#[test]
fn wait_click_emits_request_and_status_is_wait_click() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(253)); // WaitClick

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());

    let tick = runtime
        .run_frame(&assets, &config)
        .expect("run_frame should succeed on WaitClick");

    assert!(
        tick.wait_request
            .map_or(false, |r| r == pal_vm::WaitRequest::Click),
        "expected WaitRequest::Click"
    );
    assert!(
        matches!(runtime.status(), RuntimeStatus::WaitClick { .. }),
        "expected WaitClick status, got {:?}",
        runtime.status()
    );
}

#[test]
fn wait_click_zero_duration_is_one_ms_click_or_time() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(0)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 1)));
    body.extend_from_slice(&word(dst_slot(0)));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());

    let tick = runtime
        .run_frame(&assets, &config)
        .expect("run_frame should succeed on wait_click(0)");

    assert_eq!(tick.wait_request, Some(pal_vm::WaitRequest::ClickOrTime(1)));
    assert!(
        matches!(runtime.status(), RuntimeStatus::WaitClick { .. }),
        "expected WaitClick status, got {:?}",
        runtime.status()
    );
}

#[test]
fn cancellable_wait_accepts_input_before_timeout() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(1)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(4000)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 0)));
    body.extend_from_slice(&word(dst_slot(0)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime.run_frame(&assets, &config).unwrap();
    assert_eq!(tick.wait_request, Some(pal_vm::WaitRequest::ClickOrTime(4000)));
}

#[test]
fn wait_click_no_anim_zero_duration_is_one_ms_click_or_time() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(0)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 7)));
    body.extend_from_slice(&word(dst_slot(0)));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());

    let tick = runtime
        .run_frame(&assets, &config)
        .expect("run_frame should succeed on wait_click_no_anim(0)");

    assert_eq!(tick.wait_request, Some(pal_vm::WaitRequest::ClickOrTime(1)));
    assert!(
        matches!(runtime.status(), RuntimeStatus::WaitClick { .. }),
        "expected WaitClick status, got {:?}",
        runtime.status()
    );
}

#[test]
fn wait_sync_release_emits_timed_wait() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(250)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 3)));
    body.extend_from_slice(&word(dst_slot(0)));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.set_pal_time(1000);

    let tick = runtime
        .run_frame(&assets, &config)
        .expect("run_frame should succeed on wait_sync_release");

    assert_eq!(runtime.vars()[0], 1, "wait_sync_release should return 1");
    assert_eq!(tick.wait_request, Some(pal_vm::WaitRequest::Time(250)));
    assert!(
        matches!(runtime.status(), RuntimeStatus::WaitFrame { .. }),
        "expected WaitFrame status, got {:?}",
        runtime.status()
    );
}

#[test]
fn wait_sync_begin_and_get_time_are_stateful() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 2)));
    body.extend_from_slice(&word(dst_slot(0)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 8)));
    body.extend_from_slice(&word(dst_slot(1)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.set_pal_time(1234);
    run_to_stop(&mut runtime, &assets, &config);

    assert_eq!(runtime.vars()[0], 1, "wait_sync_begin should return 1");
    assert_eq!(
        runtime.vars()[1],
        0,
        "get_time in the same VM tick should see zero elapsed time"
    );
    assert!(
        matches!(runtime.status(), RuntimeStatus::Halted { .. }),
        "runtime should be halted"
    );
}

#[test]
fn title_sprite_extcalls_are_handled_by_pal_indices() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    // sp_show(slot=3)
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(3)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(3, 46)));
    body.extend_from_slice(&word(dst_slot(0)));
    // spsetanime(slot=3, anim=63, entry=0)
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(0)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(63)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(3)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(3, 57)));
    body.extend_from_slice(&word(dst_slot(1)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("sprite extcalls should run");

    assert_eq!(runtime.vars()[0], 1, "sp_show should return success");
    assert_eq!(runtime.vars()[1], 1, "spsetanime should return success");
    assert!(
        tick.frame_events.is_empty(),
        "sprite title extcalls must not be reported as skipped"
    );
}

#[test]
fn action_run_count_over_uses_pal_index_zero_without_skipping() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(2000)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(-1)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(17, 0)));
    body.extend_from_slice(&word(dst_slot(0)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("action extcall should run");

    assert_eq!(
        runtime.vars()[0],
        1,
        "native action_run_count_over returns scheduling status 1; completion is queried separately"
    );
    assert!(
        tick.frame_events.is_empty(),
        "action_run_count_over must not be reported as skipped"
    );
}

#[test]
fn action_sync_run_count_over_emits_timed_wait() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(800)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(6)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(17, 1)));
    body.extend_from_slice(&word(dst_slot(0)));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.set_pal_time(100);
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("sync action extcall should run");

    assert_eq!(runtime.vars()[0], 1, "sync action returns success");
    assert_eq!(tick.wait_request, Some(pal_vm::WaitRequest::Time(800)));
    assert!(
        matches!(runtime.status(), RuntimeStatus::WaitFrame { .. }),
        "sync action should block through the wait pipeline"
    );
}

#[test]
fn title_system_button_extcalls_use_script_arities() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    // system_btn_set(index=0, image=4056, state=3)
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(0)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(4056)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(3)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(12, 0)));
    body.extend_from_slice(&word(dst_slot(0)));
    // system_btn_enable(index=65535, enabled=0)
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(0)));
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(65535)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(12, 2)));
    body.extend_from_slice(&word(dst_slot(1)));
    // system_btn_release(mask=65535)
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(65535)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(12, 1)));
    body.extend_from_slice(&word(dst_slot(2)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("system button extcalls should run");

    assert_eq!(runtime.vars()[0], 1);
    assert_eq!(runtime.vars()[1], 1);
    assert_eq!(runtime.vars()[2], 1);
    assert!(
        tick.frame_events.is_empty(),
        "system button title extcalls must not be skipped"
    );
}

#[test]
fn category9_index35_returns_pal_time_for_title_delta_logic() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(9, 35)));
    body.extend_from_slice(&word(dst_slot(1)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.set_pal_time(3456);
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("category 9 timer extcall should run");

    assert_eq!(runtime.vars()[1], 3456);
    assert!(
        tick.frame_events.is_empty(),
        "timer extcall must not be reported as skipped"
    );
}

#[test]
fn high_frequency_pal_extcalls_are_stack_safe() {
    let entry_pc = 12u32;
    let mut body = Vec::new();

    // btn_get_push(): no input should mean no selected/pushed title button.
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(8, 17)));
    body.extend_from_slice(&word(dst_slot(0)));

    // btn_set_anim(group=0, index=14, cell=31, mode=1)
    for value in [1, 31, 14, 0] {
        body.extend_from_slice(&opcode(31));
        body.extend_from_slice(&word(imm(value)));
    }
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(8, 21)));
    body.extend_from_slice(&word(dst_slot(1)));

    // btn_set_hit(group=0, index=11)
    for value in [11, 0] {
        body.extend_from_slice(&opcode(31));
        body.extend_from_slice(&word(imm(value)));
    }
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(8, 22)));
    body.extend_from_slice(&word(dst_slot(2)));

    // sp_set_shake(slot=7, amplitude=1, count=4, axis=-1)
    for value in [-1, 4, 1, 7] {
        body.extend_from_slice(&opcode(31));
        body.extend_from_slice(&word(imm(value)));
    }
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(3, 39)));
    body.extend_from_slice(&word(dst_slot(3)));

    // sp_set_transition(name=2, slot=7, kind=1, duration=300). The fixture has
    // no File.dat/resource manager entries, so a known extcall should consume
    // its stack safely and return a PAL-style failure status instead of being
    // reported as skipped.
    for value in [300, 1, 7, 2] {
        body.extend_from_slice(&opcode(31));
        body.extend_from_slice(&word(imm(value)));
    }
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(3, 50)));
    body.extend_from_slice(&word(dst_slot(4)));

    // category 16 window/effect fade stubs.
    for value in [255, 255, 255, 100] {
        body.extend_from_slice(&opcode(31));
        body.extend_from_slice(&word(imm(value)));
    }
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(16, 2)));
    body.extend_from_slice(&word(dst_slot(5)));

    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("high-frequency PAL extcalls should run");

    assert_eq!(runtime.vars()[0], 0);
    for index in 1..=3 {
        assert_eq!(
            runtime.vars()[index],
            1,
            "var[{index}] should report success"
        );
    }
    assert_eq!(
        runtime.vars()[4],
        0,
        "sp_set_transition without a resolvable image resource reports failure"
    );
    assert_eq!(runtime.vars()[5], 1, "window/effect fade reports success");
    assert!(
        tick.frame_events.is_empty(),
        "PAL extcalls must not be reported as skipped"
    );
}

#[test]
fn wait_sync_step_yields_one_frame() {
    let entry_pc = 12u32;
    let mut body = Vec::new();

    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 5)));
    body.extend_from_slice(&word(dst_slot(0)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("wait-sync step should run");

    assert_eq!(runtime.vars()[0], 1);
    assert_eq!(tick.wait_request, Some(pal_vm::WaitRequest::Frame(1)));
    assert!(
        matches!(runtime.status(), RuntimeStatus::WaitFrame { .. }),
        "wait-sync step should suspend until the next frame, got {:?}",
        runtime.status()
    );
    assert!(
        tick.frame_events
            .iter()
            .all(|event| matches!(event, FrameEvent::WaitEmitted { .. })),
        "wait-sync step must not be reported as skipped"
    );
}

#[test]
fn wait_sync_step_does_not_clear_title_wait_memdat_flag() {
    let entry_pc = 12u32;
    let mut body = Vec::new();

    // ext_000F_0005(1) latches native misc-system state near the title loop.
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(1)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(15, 5)));
    body.extend_from_slice(&word(dst_slot(0)));

    // memdat[var[0]+1100] = 1; var[0] stays zero across wait_sync_step,
    // matching the title loop's explicit `bitxor var[0], var[0]`.
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(1)));
    body.extend_from_slice(&opcode(30));
    body.extend_from_slice(&word(memdat_direct(1100, 0)));

    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(7, 5)));
    body.extend_from_slice(&word(dst_slot(1)));

    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(2)));
    body.extend_from_slice(&word(memdat_direct(1100, 0)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.load_mem_dat(&vec![0; (1100 + 5) * 4]);

    run_to_stop(&mut runtime, &assets, &config);

    assert_eq!(
        runtime.vars()[0],
        0,
        "debug_window_set returns the previous PAL debug-window state"
    );
    assert_eq!(runtime.vars()[1], 1);
    assert_eq!(
        runtime.vars()[2],
        1,
        "wait_sync_step is only a frame fence; title callbacks or timeout script code must clear memdat[1100]"
    );
}

#[test]
fn title_btn_get_push_consumes_group_argument() {
    let entry_pc = 12u32;
    let mut body = Vec::new();

    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(5)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(8, 17)));
    body.extend_from_slice(&word(dst_slot(0)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("btn_get_push(group) should run");

    assert_eq!(runtime.vars()[0], 0);
    assert_eq!(
        runtime.stack_depth(),
        0,
        "btn_get_push(group) must consume the title button group argument"
    );
    assert!(
        tick.frame_events.is_empty(),
        "button extcall must not be reported as skipped"
    );
}

#[test]
fn memdat_direct_write_reads_back_from_shadow_words() {
    let entry_pc = 12u32;
    let mut body = Vec::new();

    // var[1] = 0; memdat[var[1]+1100] = 1; var[2] = memdat[var[1]+1100]
    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(1)));
    body.extend_from_slice(&opcode(30));
    body.extend_from_slice(&word(memdat_direct(1100, 1)));
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(2)));
    body.extend_from_slice(&word(memdat_direct(1100, 1)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.load_mem_dat(&vec![0; (1100 + 5) * 4]);
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("memdat write/read should run");

    assert_eq!(runtime.vars()[2], 1);
    assert!(
        tick.frame_events.is_empty(),
        "memdat direct access must stay inside the normal VM path"
    );
}

#[test]
fn memdat_direct_write_can_extend_shadow_work_area() {
    let entry_pc = 12u32;
    let mut body = Vec::new();

    // var[1] = 4; memdat[var[1]+20] = 77; var[2] = memdat[var[1]+20]
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(1)));
    body.extend_from_slice(&word(imm(4)));
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(memdat_direct(20, 1)));
    body.extend_from_slice(&word(imm(77)));
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(2)));
    body.extend_from_slice(&word(memdat_direct(20, 1)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.load_mem_dat(&vec![0; 8 * 4]);
    runtime
        .run_frame(&assets, &config)
        .expect("memdat direct work area should grow on write");

    assert_eq!(runtime.vars()[2], 77);
}

#[test]
fn memdat_indirect_follows_the_word_stored_at_bank_plus_header() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    // var[1] = 1; var[0] = memdat_ind[var[1] via bank 0x50F]
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(1)));
    body.extend_from_slice(&word(imm(1)));
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(0)));
    body.extend_from_slice(&word(memdat_indirect(0x50F, 1)));
    // var[1] = 2; var[2] = same indirect; then store 99 and read it back
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(1)));
    body.extend_from_slice(&word(imm(2)));
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(2)));
    body.extend_from_slice(&word(memdat_indirect(0x50F, 1)));
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(memdat_indirect(0x50F, 1)));
    body.extend_from_slice(&word(imm(99)));
    body.extend_from_slice(&opcode(1));
    body.extend_from_slice(&word(var(3)));
    body.extend_from_slice(&word(memdat_indirect(0x50F, 1)));
    body.extend_from_slice(&opcode(21));

    let mut words = vec![0i32; 1302];
    // Pointer cell is mem[bank + 4]. Its payload is a direct operand word
    // whose bank is 1295, matching koikake.exe 0x42116b.
    words[0x50F + 4] = 0x650F_0000u32 as i32;
    words[0x50F + 1 + 4] = 42;
    words[0x50F + 2 + 4] = -1;
    let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect::<Vec<_>>();

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    runtime.load_mem_dat(&bytes);
    runtime
        .run_frame(&assets, &config)
        .expect("memdat indirect should run");

    assert_eq!(runtime.vars()[0], 42);
    assert_eq!(runtime.vars()[2], -1);
    assert_eq!(runtime.vars()[3], 99);
}

#[test]
fn voice_autopan_size_over_consumes_all_four_args() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    for _ in 0..23_000 {
        body.extend_from_slice(&opcode(31)); // push
        body.extend_from_slice(&word(imm(0)));
        body.extend_from_slice(&opcode(31)); // push
        body.extend_from_slice(&word(imm(0x0FFF_FFFF)));
        body.extend_from_slice(&opcode(31)); // push
        body.extend_from_slice(&word(imm(-1)));
        body.extend_from_slice(&opcode(31)); // push
        body.extend_from_slice(&word(imm(0)));
        body.extend_from_slice(&opcode(23)); // extcall set_voice_autopan_size_over
        body.extend_from_slice(&word(ext_raw(13, 16)));
        body.extend_from_slice(&word(0));
    }
    body.extend_from_slice(&opcode(21)); // halt

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig {
        instructions_per_frame: 300_000,
        ..ScriptRuntimeConfig::default()
    };
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);
    assert!(matches!(runtime.status(), RuntimeStatus::Halted { .. }));
    assert_eq!(
        runtime.stack_depth(),
        0,
        "set_voice_autopan_size_over must consume its four PAL arguments"
    );
}

#[test]
fn voice_get_volume_takes_no_stack_arguments() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(13, 3)));
    body.extend_from_slice(&word(dst_slot(0)));
    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);

    assert_eq!(
        runtime.vars()[0],
        50,
        "Game.sqlite sub_4445A0 returns 100 * default_volume_5000 / 10000 without popping a slot"
    );
    assert_eq!(
        runtime.stack_depth(),
        0,
        "voice_get_volume must not consume or require a VM stack argument"
    );
}

#[test]
fn temp_mem_bank_operand_roundtrips_through_binary_write() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(1)); // mov arg_base, 64
    body.extend_from_slice(&word(arg_base()));
    body.extend_from_slice(&word(imm(64)));
    body.extend_from_slice(&opcode(1)); // mov var[2], 0
    body.extend_from_slice(&word(var(2)));
    body.extend_from_slice(&word(imm(0)));
    body.extend_from_slice(&opcode(8)); // bitxor var[3], var[3]
    body.extend_from_slice(&word(var(3)));
    body.extend_from_slice(&word(var(3)));
    body.extend_from_slice(&opcode(1)); // mov mem_temp[var[3]+base+130], var[2]
    body.extend_from_slice(&word(temp_mem(130, 3)));
    body.extend_from_slice(&word(var(2)));
    for _ in 0..20 {
        body.extend_from_slice(&opcode(8)); // bitxor var[1], var[1]
        body.extend_from_slice(&word(var(1)));
        body.extend_from_slice(&word(var(1)));
        body.extend_from_slice(&opcode(2)); // add mem_temp[var[1]+base+130], 1
        body.extend_from_slice(&word(temp_mem(130, 1)));
        body.extend_from_slice(&word(imm(1)));
    }
    body.extend_from_slice(&opcode(8)); // bitxor var[1], var[1]
    body.extend_from_slice(&word(var(1)));
    body.extend_from_slice(&word(var(1)));
    body.extend_from_slice(&opcode(1)); // mov var[0], mem_temp[var[1]+base+130]
    body.extend_from_slice(&word(var(0)));
    body.extend_from_slice(&word(temp_mem(130, 1)));
    body.extend_from_slice(&opcode(21)); // halt

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);
    assert_eq!(runtime.vars()[0], 20);
}

#[test]
fn temp_mem_bank_operand_grows_for_large_argument_base() {
    let entry_pc = 12u32;
    let mut body = Vec::new();
    body.extend_from_slice(&opcode(1)); // mov arg_base, 200000
    body.extend_from_slice(&word(arg_base()));
    body.extend_from_slice(&word(imm(200_000)));
    body.extend_from_slice(&opcode(1)); // mov var[3], 4
    body.extend_from_slice(&word(var(3)));
    body.extend_from_slice(&word(imm(4)));
    body.extend_from_slice(&opcode(1)); // mov mem_temp[var[3]+base+130], 77
    body.extend_from_slice(&word(temp_mem(130, 3)));
    body.extend_from_slice(&word(imm(77)));
    body.extend_from_slice(&opcode(1)); // mov var[0], mem_temp[var[3]+base+130]
    body.extend_from_slice(&word(var(0)));
    body.extend_from_slice(&word(temp_mem(130, 3)));
    body.extend_from_slice(&opcode(21)); // halt

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    run_to_stop(&mut runtime, &assets, &config);
    assert_eq!(runtime.vars()[0], 77);
}

#[test]
fn complex_action_pal_extcalls_are_stack_safe() {
    let entry_pc = 12u32;
    let mut body = Vec::new();

    for (slot, ext_index, args) in [
        (0, 5, &[71, 3, 2, 1, 0, 72][..]),
        (1, 6, &[71, 1, 2, 73, 0, 72][..]),
        (2, 7, &[2, 1, 16, 72][..]),
        (3, 10, &[2, 3, 0, 0, 72, 71][..]),
        (4, 14, &[71, 1, 2, 3, 0, 72][..]),
        (5, 15, &[71, 1, 2, 3, 0, 72][..]),
        (6, 20, &[71, 1, 73, 2, 72][..]),
    ] {
        for value in args.iter().rev().copied() {
            body.extend_from_slice(&opcode(31));
            body.extend_from_slice(&word(imm(value)));
        }
        body.extend_from_slice(&opcode(23));
        body.extend_from_slice(&word(ext_raw(17, ext_index)));
        body.extend_from_slice(&word(dst_slot(slot)));
    }

    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(17, 28)));
    body.extend_from_slice(&word(dst_slot(7)));

    body.extend_from_slice(&opcode(31));
    body.extend_from_slice(&word(imm(-1)));
    body.extend_from_slice(&opcode(23));
    body.extend_from_slice(&word(ext_raw(17, 30)));
    body.extend_from_slice(&word(dst_slot(8)));

    body.extend_from_slice(&opcode(21));

    let assets = assets_with_script(entry_pc, body);
    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("complex action extcalls should run");

    for index in 0..=8 {
        assert_eq!(
            runtime.vars()[index],
            1,
            "var[{index}] should report success"
        );
    }
    assert!(
        tick.frame_events.is_empty(),
        "complex action PAL extcalls must not be skipped"
    );
}
