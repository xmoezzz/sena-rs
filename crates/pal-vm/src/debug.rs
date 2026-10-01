use crate::runtime::{FrameEvent, RuntimeStatus, WaitRequest};
use crate::scene::{DrawCommand, FrameScene};
use crate::sprite::SpriteSystem;
use crate::task::{TaskDumpEntry, TaskState, TaskSystem};

pub fn pal_debug_enabled() -> bool {
    std::env::var("PAL_DEBUG")
        .ok()
        .as_deref()
        .map_or(false, |v| v == "1")
}

pub fn pal_debug_frame_enabled(frame_index: u64) -> bool {
    if pal_debug_enabled() {
        return true;
    }
    std::env::var("PAL_DEBUG_FRAME")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .is_some_and(|target| target == frame_index)
}

#[derive(Debug)]
pub struct FrameDebugDump {
    pub frame_index: u64,
    pub pal_time_ms: u32,
    pub delta_ms: u32,
    pub runtime_status: String,
    pub current_pc: Option<u32>,
    pub frame_events: Vec<String>,
    pub task_count: usize,
    pub blocking_task: bool,
    pub sprite_count: usize,
    pub surface_count: usize,
    pub render_node_count: usize,
    pub draw_command_count: usize,
    pub logical_size: (u32, u32),
    pub task_entries: Vec<TaskDumpEntry>,
    pub sprite_entries: Vec<SpriteDumpEntry>,
    pub render_entries: Vec<RenderDumpEntry>,
    pub draw_entries: Vec<DrawCmdDumpEntry>,
}

#[derive(Debug)]
pub struct SpriteDumpEntry {
    pub handle: u32,
    pub source_name: String,
    pub surface_id: u64,
    pub texture_size: (u32, u32),
    pub cell_size: (u32, u32),
    pub source_rect: (i32, i32, i32, i32),
    pub position: (f32, f32, f32),
    pub offset: (i32, i32),
    pub center_offset: (i32, i32),
    pub base_priority: i32,
    pub effective_priority: i32,
    pub visible: bool,
    pub locked: bool,
    pub transition_block: u32,
    pub color: u32,
    pub scale: f32,
    pub rotation: (f32, f32, f32),
    pub render_mode: u32,
}

#[derive(Debug)]
pub struct RenderDumpEntry {
    pub node_id: u32,
    pub sprite_handle: u32,
    pub priority: i32,
}

#[derive(Debug)]
pub struct DrawCmdDumpEntry {
    pub index: usize,
    pub texture_id: u64,
    pub dst: (f32, f32, f32, f32),
    pub src: (f32, f32, f32, f32),
    pub source_rect: (i32, i32, i32, i32),
    pub texture_size: (u32, u32),
    pub cell_size: (u32, u32),
    pub position: (f32, f32, f32),
    pub offset: (i32, i32),
    pub center_offset: (f32, f32),
    pub priority: i32,
    pub color: [f32; 4],
    pub render_mode: u32,
    pub source_kind: &'static str,
}

pub fn collect_frame_dump(
    frame_index: u64,
    pal_time_ms: u32,
    delta_ms: u32,
    runtime_status: &RuntimeStatus,
    frame_events: &[FrameEvent],
    tasks: &TaskSystem,
    sprites: &SpriteSystem,
    scene: &FrameScene,
) -> FrameDebugDump {
    let current_pc = runtime_status.pc();
    let runtime_status_str = runtime_status.to_string();

    let formatted_events: Vec<String> = frame_events
        .iter()
        .map(|ev| match ev {
            FrameEvent::ExtCallSkipped {
                pc,
                category,
                index,
                name,
            } => {
                let n = name.as_deref().unwrap_or("?");
                format!("ext_skip pc=0x{pc:08X} cat={category} idx={index} name={n}")
            }
            FrameEvent::WaitEmitted { pc, kind } => {
                let k = match kind {
                    WaitRequest::Frame(n) => format!("Frame({n})"),
                    WaitRequest::Time(ms) => format!("Time({ms})"),
                    WaitRequest::Click => "Click".to_owned(),
                    WaitRequest::ClickOrTime(ms) => format!("ClickOrTime({ms})"),
                    WaitRequest::AutoClickOrTime(ms) => format!("AutoClickOrTime({ms})"),
                    WaitRequest::TextReveal(ms) => format!("TextReveal({ms})"),
                };
                format!("wait pc=0x{pc:08X} kind={k}")
            }
            FrameEvent::UnsupportedCmd { pc, opcode, name } => {
                let n = name.as_deref().unwrap_or("?");
                format!("unsup_cmd pc=0x{pc:08X} opcode={opcode} name={n}")
            }
            FrameEvent::UnsupportedExt {
                pc,
                category,
                index,
                name,
            } => {
                let n = name.as_deref().unwrap_or("?");
                format!("unsup_ext pc=0x{pc:08X} cat={category} idx={index} name={n}")
            }
        })
        .collect();

    let task_entries = tasks.dump_entries();
    let task_count = task_entries.len();
    let blocking_task = tasks.has_blocking_task();

    let sprite_entries: Vec<SpriteDumpEntry> = sprites
        .iter_sprites()
        .map(|(handle, sp)| SpriteDumpEntry {
            handle: handle.0,
            source_name: sp.source_name.clone(),
            surface_id: sp.surface.0,
            texture_size: (sp.texture_size.width, sp.texture_size.height),
            cell_size: (sp.cell_size.width, sp.cell_size.height),
            source_rect: (
                sp.source_rect.left,
                sp.source_rect.top,
                sp.source_rect.right,
                sp.source_rect.bottom,
            ),
            position: (sp.position.x, sp.position.y, sp.position.z),
            offset: (sp.offset.x, sp.offset.y),
            center_offset: (sp.center_offset.x, sp.center_offset.y),
            base_priority: sp.base_priority,
            effective_priority: sp.effective_priority(),
            visible: sp.visible,
            locked: sp.locked,
            transition_block: sp.transition_block,
            color: sp.color.0,
            scale: sp.scale,
            rotation: (sp.rotation.x, sp.rotation.y, sp.rotation.z),
            render_mode: sp.render_mode.raw(),
        })
        .collect();

    let render_entries: Vec<RenderDumpEntry> = sprites
        .render_nodes()
        .map(|node| RenderDumpEntry {
            node_id: node.id.0,
            sprite_handle: node.sprite.0,
            priority: node.priority,
        })
        .collect();

    let draw_entries: Vec<DrawCmdDumpEntry> = scene
        .commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| match cmd {
            DrawCommand::Sprite(sp) => DrawCmdDumpEntry {
                index: i,
                texture_id: sp.texture_id.0,
                dst: (sp.dst.x, sp.dst.y, sp.dst.w, sp.dst.h),
                src: (sp.src.x, sp.src.y, sp.src.w, sp.src.h),
                source_rect: (
                    sp.source_rect[0],
                    sp.source_rect[1],
                    sp.source_rect[2],
                    sp.source_rect[3],
                ),
                texture_size: (sp.texture_size[0], sp.texture_size[1]),
                cell_size: (sp.cell_size[0], sp.cell_size[1]),
                position: (sp.position[0], sp.position[1], sp.position[2]),
                offset: (sp.offset[0], sp.offset[1]),
                center_offset: (sp.center_offset[0], sp.center_offset[1]),
                priority: sp.priority,
                color: sp.color,
                render_mode: sp.render_mode,
                source_kind: "sprite",
            },
            DrawCommand::SolidQuad(q) => DrawCmdDumpEntry {
                index: i,
                texture_id: 0,
                dst: (q.dst.x, q.dst.y, q.dst.w, q.dst.h),
                src: (0.0, 0.0, 1.0, 1.0),
                source_rect: (0, 0, q.dst.w as i32, q.dst.h as i32),
                texture_size: (0, 0),
                cell_size: (q.dst.w as u32, q.dst.h as u32),
                position: (q.dst.x, q.dst.y, 0.0),
                offset: (0, 0),
                center_offset: (0.0, 0.0),
                priority: 0,
                color: q.color,
                render_mode: 0,
                source_kind: "solid_quad",
            },
        })
        .collect();

    FrameDebugDump {
        frame_index,
        pal_time_ms,
        delta_ms,
        runtime_status: runtime_status_str,
        current_pc,
        frame_events: formatted_events,
        task_count,
        blocking_task,
        sprite_count: sprites.len(),
        surface_count: sprites.surface_count(),
        render_node_count: sprites.render_node_count(),
        draw_command_count: scene.commands.len(),
        logical_size: (scene.logical_width, scene.logical_height),
        task_entries,
        sprite_entries,
        render_entries,
        draw_entries,
    }
}

pub fn print_dump(dump: &FrameDebugDump) {
    eprintln!("[PAL_DEBUG] ===== Frame {} =====", dump.frame_index);
    eprintln!(
        "[PAL_DEBUG] time: pal_ms={} delta_ms={}",
        dump.pal_time_ms, dump.delta_ms
    );
    eprintln!(
        "[PAL_DEBUG] target: logical={}x{}",
        dump.logical_size.0, dump.logical_size.1
    );
    eprintln!("[PAL_DEBUG] runtime: {}", dump.runtime_status);
    if let Some(pc) = dump.current_pc {
        eprintln!("[PAL_DEBUG] pc: 0x{pc:08X}");
    }
    eprintln!("[PAL_DEBUG] counts: tasks={} blocking={} sprites={} surfaces={} render_nodes={} draw_cmds={}",
        dump.task_count, dump.blocking_task,
        dump.sprite_count, dump.surface_count,
        dump.render_node_count, dump.draw_command_count
    );

    if !dump.frame_events.is_empty() {
        eprintln!("[PAL_DEBUG] frame events ({}):", dump.frame_events.len());
        for ev in &dump.frame_events {
            eprintln!("[PAL_DEBUG]   {ev}");
        }
    }

    eprintln!(
        "[PAL_DEBUG] --- Task tree ({}) ---",
        dump.task_entries.len()
    );
    for entry in &dump.task_entries {
        let state = match entry.state {
            TaskState::Free => "free",
            TaskState::Active => "active",
            TaskState::PendingFree => "pending_free",
            TaskState::ChildSentinel => "child_sentinel",
        };
        let parent_str = entry
            .parent_index
            .map_or("none".to_owned(), |p| format!("{p}"));
        eprintln!(
            "[PAL_DEBUG]   handle=0x{:08X} kind={} state={} flags=0x{:X} blocking={} pending_free={} parent={} children={}",
            entry.handle.0,
            entry.kind_name,
            state,
            entry.flags,
            entry.is_blocking,
            entry.is_pending_free,
            parent_str,
            entry.children_count,
        );
    }

    eprintln!(
        "[PAL_DEBUG] --- Sprite tree ({}) ---",
        dump.sprite_entries.len()
    );
    for sp in &dump.sprite_entries {
        let vis_str = if sp.visible { "vis" } else { "hid" };
        let lock_str = if sp.locked { " LOCKED" } else { "" };
        let trans_str = if sp.transition_block != 0 {
            format!(" trans={}", sp.transition_block)
        } else {
            String::new()
        };
        eprintln!(
            "[PAL_DEBUG]   sprite={} surface={} {}{}{} pos=({:.0},{:.0},{:.0}) offset=({},{}) center=({},{}) \
             tex={}x{} cell={}x{} src=({},{},{},{}) prio={} scale={:.2} mode={} color=0x{:08X} src_name={:?}",
            sp.handle, sp.surface_id,
            vis_str, lock_str, trans_str,
            sp.position.0, sp.position.1, sp.position.2,
            sp.offset.0, sp.offset.1,
            sp.center_offset.0, sp.center_offset.1,
            sp.texture_size.0, sp.texture_size.1,
            sp.cell_size.0, sp.cell_size.1,
            sp.source_rect.0, sp.source_rect.1, sp.source_rect.2, sp.source_rect.3,
            sp.effective_priority, sp.scale, sp.render_mode, sp.color,
            sp.source_name,
        );
    }

    eprintln!(
        "[PAL_DEBUG] --- Render nodes ({}) ---",
        dump.render_entries.len()
    );
    for rn in &dump.render_entries {
        eprintln!(
            "[PAL_DEBUG]   node={} sprite={} priority={}",
            rn.node_id, rn.sprite_handle, rn.priority
        );
    }

    eprintln!(
        "[PAL_DEBUG] --- Draw commands ({}) ---",
        dump.draw_entries.len()
    );
    for dc in &dump.draw_entries {
        eprintln!(
            "[PAL_DEBUG]   [{}] kind={} tex={} dst=({:.0},{:.0},{:.0}x{:.0}) src_uv=({:.3},{:.3},{:.3}x{:.3}) src_px=({},{},{},{}) tex={}x{} cell={}x{} raw_pos=({:.0},{:.0},{:.0}) offset=({},{}) center=({:.0},{:.0}) prio={} mode={}",
            dc.index, dc.source_kind, dc.texture_id,
            dc.dst.0, dc.dst.1, dc.dst.2, dc.dst.3,
            dc.src.0, dc.src.1, dc.src.2, dc.src.3,
            dc.source_rect.0, dc.source_rect.1, dc.source_rect.2, dc.source_rect.3,
            dc.texture_size.0, dc.texture_size.1,
            dc.cell_size.0, dc.cell_size.1,
            dc.position.0, dc.position.1, dc.position.2,
            dc.offset.0, dc.offset.1,
            dc.center_offset.0, dc.center_offset.1,
            dc.priority, dc.render_mode,
        );
    }
}
