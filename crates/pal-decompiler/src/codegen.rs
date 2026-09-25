//! Lua code generation from the structural IR.

use std::collections::BTreeMap;
use std::fmt::Write;

use pal_asset::Nls;
use pal_script::extsig::{lookup_sig, observed_pop_count, ParamKind, ReturnKind};
use pal_script::{Argument, Instruction, Operand, OperandKind};

use crate::cfg::{extract_static_target_pc, instr_end_pc, Block};
use crate::decompile::DecompileContext;
use crate::strings::{resolve_resource_str, resolve_string_id, resolve_text_str};

// ─── Operand → Lua expression ─────────────────────────────────────────────────

/// Render an Operand as a Lua expression.
/// `file_entries` and `text_entries` are used for LiteralSlot resolution.
pub fn operand_to_lua(
    op: &Operand,
    file_entries: &[String],
    text_entries: &[(u32, String)],
    nls: Option<Nls>,
) -> String {
    match op.kind {
        OperandKind::Immediate => {
            let v = op.raw as i32;
            if v >= 0 && v < 0x1000 {
                format!("{v}")
            } else {
                format!("0x{:X}", op.raw)
            }
        }
        OperandKind::UserMemoryViaVar => format!("mem_user[v{}]", op.lo),
        OperandKind::SystemMemoryViaVar => format!("mem_sys[v{}]", op.lo),
        OperandKind::StackSlot => format!("s{}", op.lo),
        OperandKind::VariableSlot => format!("v{}", op.lo),
        OperandKind::TempMemoryViaVar => format!("mem_tmp[v{}+{}]", op.lo, op.bank),
        OperandKind::MemDatDirect => format!("memdat[v{}+{}]", op.lo, op.bank),
        OperandKind::MemDatIndirect => format!("memdat_ind[v{}]", op.lo),
        OperandKind::ArgumentStack => format!("arg[-{}]", op.lo),
        OperandKind::ArgumentBase => "arg_base".to_owned(),
        OperandKind::LiteralSlot => resolve_string_id(op.raw, file_entries, text_entries, nls),
    }
}

// ─── Instruction → Lua statement ─────────────────────────────────────────────

/// Emit a single instruction as a Lua statement string (without trailing newline).
pub fn emit_single_instr_lua(
    instr: &Instruction,
    file_entries: &[String],
    text_entries: &[(u32, String)],
    nls: Option<Nls>,
    pending_args: &[Operand],
) -> String {
    match instr {
        Instruction::DataWord { pc, word, .. } => {
            format!("-- .word 0x{word:08X}  -- pc=0x{pc:08X}")
        }
        Instruction::Primary {
            opcode, args, pc, ..
        } => emit_primary(
            *opcode,
            args,
            *pc,
            file_entries,
            text_entries,
            nls,
            pending_args,
            &[],
            &[],
        ),
    }
}

/// Emit a single instruction, also accepting raw file/text bytes for sig-based resolution.
pub fn emit_single_instr_lua_full(
    instr: &Instruction,
    file_entries: &[String],
    text_entries: &[(u32, String)],
    nls: Option<Nls>,
    pending_args: &[Operand],
    file_bytes: &[u8],
    text_bytes: &[u8],
) -> String {
    match instr {
        Instruction::DataWord { pc, word, .. } => {
            format!("-- .word 0x{word:08X}  -- pc=0x{pc:08X}")
        }
        Instruction::Primary {
            opcode, args, pc, ..
        } => emit_primary(
            *opcode,
            args,
            *pc,
            file_entries,
            text_entries,
            nls,
            pending_args,
            file_bytes,
            text_bytes,
        ),
    }
}

fn o2l(op: &Operand, fe: &[String], te: &[(u32, String)], nls: Option<Nls>) -> String {
    operand_to_lua(op, fe, te, nls)
}

fn emit_dynamic_jump(
    arg: Option<&Argument>,
    pc: u32,
    fe: &[String],
    te: &[(u32, String)],
    nls: Option<Nls>,
) -> String {
    match arg {
        Some(Argument::Point { id, .. }) => format!("dynamic_jump({id})"),
        Some(Argument::PointOperand { operand, .. } | Argument::Operand(operand)) => {
            format!("dynamic_jump({})", o2l(operand, fe, te, nls))
        }
        _ => format!("dynamic_jump(0) -- unresolved @ 0x{pc:08X}"),
    }
}

fn operand_arg(arg: &Argument) -> Option<&Operand> {
    match arg {
        Argument::Operand(op) => Some(op),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_primary(
    opcode: u16,
    args: &[Argument],
    pc: u32,
    fe: &[String],
    te: &[(u32, String)],
    nls: Option<Nls>,
    pending_args: &[Operand],
    file_bytes: &[u8],
    text_bytes: &[u8],
) -> String {
    match opcode {
        // mov: dst = src
        1 => {
            if let (Some(dst), Some(src)) = (
                args.first().and_then(operand_arg),
                args.get(1).and_then(operand_arg),
            ) {
                format!("{} = {}", o2l(dst, fe, te, nls), o2l(src, fe, te, nls))
            } else {
                format!("-- mov (malformed) @ 0x{pc:08X}")
            }
        }
        // Arithmetic: dst op= src
        op @ (2..=8 | 26..=28) => {
            let sym = arith_sym(op);
            if let (Some(dst), Some(src)) = (
                args.first().and_then(operand_arg),
                args.get(1).and_then(operand_arg),
            ) {
                let d = o2l(dst, fe, te, nls);
                let s = o2l(src, fe, te, nls);
                format!("{d} = {d} {sym} {s}")
            } else {
                format!("-- {sym} (malformed) @ 0x{pc:08X}")
            }
        }
        // Compare: dst = dst cmp src
        op @ (12..=19) => {
            let sym = cmp_sym(op);
            if let (Some(dst), Some(src)) = (
                args.first().and_then(operand_arg),
                args.get(1).and_then(operand_arg),
            ) {
                let d = o2l(dst, fe, te, nls);
                let s = o2l(src, fe, te, nls);
                format!("{d} = ({d} {sym} {s}) and 1 or 0")
            } else {
                format!("-- cmp (malformed) @ 0x{pc:08X}")
            }
        }
        // lnot_slot
        20 => {
            if let Some(Argument::RawSlot { slot, .. }) = args.first() {
                format!("v{slot} = (v{slot} == 0) and 1 or 0")
            } else if let Some(op) = args.first().and_then(operand_arg) {
                let d = o2l(op, fe, te, nls);
                format!("{d} = ({d} == 0) and 1 or 0")
            } else {
                format!("-- lnot_slot (malformed) @ 0x{pc:08X}")
            }
        }
        // neg_slot
        29 => {
            if let Some(Argument::RawSlot { slot, .. }) = args.first() {
                format!("v{slot} = -v{slot}")
            } else if let Some(op) = args.first().and_then(operand_arg) {
                let d = o2l(op, fe, te, nls);
                format!("{d} = -{d}")
            } else {
                format!("-- neg_slot (malformed) @ 0x{pc:08X}")
            }
        }
        // nop
        22 => String::new(),
        // end / ret — handled structurally, but emit as comment if we get here
        21 => "return".to_owned(),
        24 => "return".to_owned(),
        // reset_adv
        25 => "reset_adv()".to_owned(),
        // pop
        30 => {
            if let Some(op) = args.first().and_then(operand_arg) {
                format!("-- pop {}", o2l(op, fe, te, nls))
            } else {
                "-- pop".to_owned()
            }
        }
        // push — accumulated in arg_stack, consumed by next extcall; silent here
        31 | 32 | 33 => String::new(),
        // extcall
        23 => emit_extcall(args, fe, te, nls, pending_args, file_bytes, text_bytes),
        // jmp — handled structurally; if we reach here it's a static or dynamic jump
        9 => {
            if let Some(tgt) = extract_static_target_pc(args.first()) {
                format!("-- jump 0x{tgt:08X}")
            } else {
                emit_dynamic_jump(args.first(), pc, fe, te, nls)
            }
        }
        // jf — handled structurally
        10 => "-- jf (structural)".to_owned(),
        // gosub — handled structurally
        11 => "-- gosub (structural)".to_owned(),
        // create_message / get_message / get_message_param
        35 => "create_message()".to_owned(),
        36 => "get_message()".to_owned(),
        37 => "get_message_param()".to_owned(),
        // save/load family
        40 => "save()".to_owned(),
        41 => "load()".to_owned(),
        42 => "save_set_title()".to_owned(),
        43 => "save_data()".to_owned(),
        44 => "save_set_thumbnail_size()".to_owned(),
        45 => "thumbnail_set()".to_owned(),
        46 => "savetitledraw()".to_owned(),
        47 => "save_set_font_size()".to_owned(),
        48 => "getsaveday()".to_owned(),
        49 => "is_save()".to_owned(),
        _ => format!("-- unsupported opcode {opcode} @ 0x{pc:08X}"),
    }
}

/// Render a single Operand value according to the given ParamKind.
fn render_param(
    op: &Operand,
    kind: ParamKind,
    fe: &[String],
    te: &[(u32, String)],
    nls: Option<Nls>,
    file_bytes: &[u8],
    text_bytes: &[u8],
) -> String {
    match kind {
        ParamKind::ResourceStringFromFileDat
        | ParamKind::ResourceName
        | ParamKind::FileDatString => {
            // Resolve Immediate and LiteralSlot operands; variables stay as expressions.
            // LiteralSlot encodes a literal value (e.g., 0xFFFFFFFF) in the instruction stream.
            if matches!(op.kind, OperandKind::Immediate | OperandKind::LiteralSlot) {
                if let Some(n) = nls {
                    return resolve_resource_str(file_bytes, op.raw, n);
                }
            }
            o2l(op, fe, te, nls)
        }
        ParamKind::TextStringFromTextDat
        | ParamKind::TextString
        | ParamKind::TextId
        | ParamKind::DynamicString
        | ParamKind::IniSection
        | ParamKind::IniKey => {
            if matches!(op.kind, OperandKind::Immediate | OperandKind::LiteralSlot) {
                if let Some(n) = nls {
                    return resolve_text_str(text_bytes, op.raw, n);
                }
            }
            o2l(op, fe, te, nls)
        }
        ParamKind::IniFilename => {
            // Like TextStringFromTextDat but annotates when the string is not a *.ini filename.
            // The runtime uses ini_filename_or_system() which falls back to SYSTEM.INI
            // if the resolved string does not end in ".ini".
            let raw_str = if matches!(op.kind, OperandKind::Immediate | OperandKind::LiteralSlot) {
                if let Some(n) = nls {
                    Some(resolve_text_str(text_bytes, op.raw, n))
                } else {
                    None
                }
            } else {
                None
            };
            match raw_str {
                Some(ref s) if s.starts_with('"') => {
                    // Check if it ends with .ini (ignoring closing quote)
                    let inner = s.trim_matches('"');
                    if inner.to_ascii_lowercase().ends_with(".ini") {
                        s.clone()
                    } else {
                        format!("{s} --[[runtime uses SYSTEM.INI: not *.ini]]")
                    }
                }
                Some(s) => s,
                None => o2l(op, fe, te, nls),
            }
        }
        // Non-string kinds: plain Lua expression.
        // LiteralSlot operands encode literal integer sentinels (e.g., 0xFFFFFFFF = -1 for
        // "default/auto slot"); render them as signed integers, not as string IDs.
        ParamKind::Integer
        | ParamKind::Slot
        | ParamKind::SpriteSlot
        | ParamKind::ButtonSlot
        | ParamKind::SoundSlot
        | ParamKind::PointId
        | ParamKind::CoordinateX
        | ParamKind::CoordinateY
        | ParamKind::CoordinateZ
        | ParamKind::Coordinate
        | ParamKind::DurationMs
        | ParamKind::Duration
        | ParamKind::Volume
        | ParamKind::Color
        | ParamKind::Alpha
        | ParamKind::Flag
        | ParamKind::Mode
        | ParamKind::Handle
        | ParamKind::BufferPointer
        | ParamKind::Unknown => {
            if op.kind == OperandKind::LiteralSlot {
                return format!("{}", op.raw as i32);
            }
            o2l(op, fe, te, nls)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_extcall(
    args: &[Argument],
    fe: &[String],
    te: &[(u32, String)],
    nls: Option<Nls>,
    pending_args: &[Operand],
    file_bytes: &[u8],
    text_bytes: &[u8],
) -> String {
    let (cat, idx, name_opt) = match args.first() {
        Some(Argument::ExtCall {
            category,
            index,
            name,
            ..
        }) => (*category, *index, *name),
        _ => return "-- extcall (malformed)".to_owned(),
    };

    let dst_expr = match args.get(1) {
        Some(Argument::DestinationSlot { slot, raw }) => {
            if *raw == 0 && *slot == 0 {
                None
            } else {
                Some(format!("v{slot}"))
            }
        }
        _ => None,
    };

    // Fix: reverse arg_stack to get pop-order (args[0] = last pushed = runtime's args[0])
    let args_pop_order: Vec<Operand> = pending_args.iter().rev().cloned().collect();

    if let Some(sig) = lookup_sig(cat, idx) {
        let param_strs: Vec<String> = sig
            .params
            .iter()
            .map(|p| {
                args_pop_order
                    .get(p.pop_idx)
                    .map(|op| render_param(op, p.kind, fe, te, nls, file_bytes, text_bytes))
                    .unwrap_or_else(|| p.name.to_owned())
            })
            .collect();

        let args_str = param_strs.join(", ");
        let name = sig.name;

        return match dst_expr.filter(|_| sig.return_kind != ReturnKind::Void) {
            Some(dst) => format!("{dst} = {name}({args_str})"),
            None => format!("{name}({args_str})"),
        };
    }

    let func_name = name_opt
        .map(str::to_owned)
        .unwrap_or_else(|| format!("ext_{cat:04X}_{idx:04X}"));
    let args_str = raw_arg_list(&args_pop_order, fe, te, nls).join(", ");
    let call = format!("{func_name}({args_str})");
    match dst_expr {
        Some(dst) => format!("{dst} = {call}"),
        None => call,
    }
}

fn raw_arg_list(
    args_pop_order: &[Operand],
    fe: &[String],
    te: &[(u32, String)],
    nls: Option<Nls>,
) -> Vec<String> {
    args_pop_order
        .iter()
        .map(|op| o2l(op, fe, te, nls))
        .collect()
}

fn arith_sym(op: u16) -> &'static str {
    match op {
        2 => "+",
        3 => "-",
        4 => "*",
        5 => "/",
        6 => "&",
        7 => "|",
        8 => "~",
        26 => "%",
        27 => "<<",
        28 => ">>",
        _ => "?",
    }
}

fn cmp_sym(op: u16) -> &'static str {
    match op {
        12 => "==",
        13 => "~=",
        14 => "<=",
        15 => ">=",
        16 => "<",
        17 => ">",
        18 => "or",
        19 => "and",
        _ => "?",
    }
}

// ─── Structural Lua emission ─────────────────────────────────────────────────

/// Emit a complete Lua module from the decompile context.
pub fn emit_lua(ctx: &DecompileContext, entry_pc: u32) -> String {
    let mut out = String::new();

    emit_lua_header(&mut out);

    // Collect all gosub targets that need to become functions
    let mut func_pcs: Vec<u32> = ctx.gosub_targets.iter().copied().collect();
    // Also the entry point
    if !func_pcs.contains(&entry_pc) {
        func_pcs.insert(0, entry_pc);
    }
    func_pcs.sort();

    // Generate each function.
    // Fix: use exit_pc=None so functions end at ret/end, not at the next gosub target.
    for &func_pc in &func_pcs {
        let func_name = ctx.proc_name(func_pc);
        let _ = writeln!(out, "\nlocal function {func_name}()");

        // Pass fn_entry so emit_region knows not to inline other gosub targets
        let body = emit_region(ctx, func_pc, None, &mut vec![], 1, func_pc);
        out.push_str(&body);
        let _ = writeln!(out, "end");
    }

    let entry_name = ctx.proc_name(entry_pc);
    let _ = writeln!(out, "\n{entry_name}()");

    out
}

fn emit_lua_header(out: &mut String) {
    let _ = writeln!(out, "local AUTO_X = 0xFFFF");
    let _ = writeln!(out, "local AUTO_Y = 0xFFFF");
    let _ = writeln!(out);
    let _ = writeln!(out, "local function dynamic_jump(point)");
    let _ = writeln!(
        out,
        "    error(string.format(\"dynamic_jump(%s) requires runtime\", tostring(point)))"
    );
    let _ = writeln!(out, "end");
    let _ = writeln!(out);
    let _ = writeln!(out, "local function dynamic_string(id)");
    let _ = writeln!(
        out,
        "    return string.format(\"<dynamic_string:%08X>\", id)"
    );
    let _ = writeln!(out, "end");
}

fn indent(depth: usize) -> String {
    "    ".repeat(depth)
}

/// Re-emit a slice of non-terminal instructions using a fresh VM stack simulation.
/// Returns the Lua statements as a String (no trailing newline added per stmt).
fn emit_instrs_slice(instrs: &[Instruction], ctx: &DecompileContext, depth: usize) -> String {
    let mut out = String::new();
    let ind = indent(depth);
    let mut stack: Vec<Operand> = Vec::new();
    let mut arg_stack: Vec<Operand> = Vec::new();

    for instr in instrs {
        match instr {
            pal_script::Instruction::Primary {
                opcode: 31, args, ..
            } => {
                if let Some(pal_script::Argument::Operand(op)) = args.first() {
                    stack.push(*op);
                }
                // push itself is silent
            }
            pal_script::Instruction::Primary {
                opcode: 32, args, ..
            } => {
                if let Some(count) = static_count_arg(args) {
                    if count <= stack.len() {
                        let start = stack.len() - count;
                        arg_stack.extend(stack.drain(start..));
                    } else {
                        stack.clear();
                        arg_stack.clear();
                    }
                } else {
                    stack.clear();
                    arg_stack.clear();
                }
            }
            pal_script::Instruction::Primary {
                opcode: 33, args, ..
            } => {
                if let Some(count) = static_count_arg(args) {
                    if count <= arg_stack.len() {
                        let new_len = arg_stack.len() - count;
                        arg_stack.truncate(new_len);
                    } else {
                        arg_stack.clear();
                    }
                } else {
                    arg_stack.clear();
                }
            }
            pal_script::Instruction::Primary { opcode: 23, .. } => {
                let count = extcall_static_arg_count(instr, stack.len());
                let current_args = take_extcall_args(&mut stack, count);
                let stmt = emit_single_instr_lua_full(
                    instr,
                    ctx.file_entries,
                    ctx.text_entries,
                    ctx.nls,
                    &current_args,
                    ctx.file_bytes,
                    ctx.text_bytes,
                );
                if !stmt.is_empty() {
                    let _ = writeln!(out, "{ind}{stmt}");
                }
            }
            pal_script::Instruction::Primary { opcode: 30, .. } => {
                stack.pop();
            }
            _ => {
                let stmt = emit_single_instr_lua_full(
                    instr,
                    ctx.file_entries,
                    ctx.text_entries,
                    ctx.nls,
                    &[],
                    ctx.file_bytes,
                    ctx.text_bytes,
                );
                if !stmt.is_empty() {
                    let _ = writeln!(out, "{ind}{stmt}");
                }
            }
        }
    }

    out
}

fn static_count_arg(args: &[pal_script::Argument]) -> Option<usize> {
    let op = args.first().and_then(|arg| match arg {
        pal_script::Argument::Operand(op) => Some(op),
        _ => None,
    })?;
    (op.kind == OperandKind::Immediate)
        .then_some(op.raw as i32)
        .filter(|count| *count >= 0)
        .map(|count| count as usize)
}

fn extcall_static_arg_count(instr: &Instruction, available: usize) -> usize {
    let Instruction::Primary { args, .. } = instr else {
        return 0;
    };
    let Some(pal_script::Argument::ExtCall {
        category, index, ..
    }) = args.first()
    else {
        return 0;
    };
    lookup_sig(*category, *index)
        .map(|sig| sig.pop_count)
        .or_else(|| observed_pop_count(*category, *index))
        .unwrap_or(available)
}

fn take_extcall_args(arg_stack: &mut Vec<Operand>, count: usize) -> Vec<Operand> {
    if count > arg_stack.len() {
        let args = arg_stack.clone();
        arg_stack.clear();
        return args;
    }
    let start = arg_stack.len() - count;
    let args = arg_stack[start..].to_vec();
    arg_stack.truncate(start);
    args
}

/// If `block` is a while-loop header WITH non-terminal instructions (condition
/// computation), emit the entire `while true do … end` construct and return it.
/// Returns None if this is not a complex-while scenario.
fn try_emit_complex_while(
    ctx: &DecompileContext,
    block: &Block,
    exit_pc: Option<u32>,
    visited: &mut Vec<u32>,
    depth: usize,
    fn_entry: u32,
) -> Option<String> {
    let n = block.instrs.len();
    if n < 2 {
        return None;
    }

    // Last instruction must be jf
    let last = &block.instrs[n - 1];
    let (branch_pc, cond_op) = match last {
        Instruction::Primary {
            opcode: 10, args, ..
        } => match (args.first(), args.get(1)) {
            (
                Some(Argument::Point {
                    target_pc: Some(bp),
                    ..
                }),
                Some(Argument::Operand(cond)),
            ) => (*bp, cond),
            _ => return None,
        },
        _ => return None,
    };

    let fall_pc = instr_end_pc(last);
    let header_pc = block.start_pc;

    // Must be a while loop (body has a back-edge to header)
    if !has_back_edge_to(header_pc, fall_pc, &ctx.blocks, 64) {
        return None;
    }

    // Must have non-terminal instructions (otherwise, simple while handles it)
    let header_non_terms = &block.instrs[..n - 1];
    if header_non_terms.is_empty() {
        return None;
    }

    // Complex while confirmed — emit it all here
    let mut out = String::new();
    let ind = indent(depth);
    let inner = indent(depth + 1);
    let cond_expr = operand_to_lua(cond_op, ctx.file_entries, ctx.text_entries, ctx.nls);

    // Mark header as visited
    visited.push(header_pc);

    let header_label = ctx.label_name(header_pc);
    if needs_label(ctx, header_pc) {
        let _ = writeln!(out, "{ind}::{header_label}::");
    }

    let _ = writeln!(out, "{ind}while true do");

    // Re-emit header non-terminals inside the loop
    let header_inner = emit_instrs_slice(header_non_terms, ctx, depth + 1);
    out.push_str(&header_inner);

    // Break condition
    let _ = writeln!(out, "{inner}if {cond_expr} == 0 then break end");

    // Emit body
    let body_visited = &mut visited.clone();
    let body = emit_region(
        ctx,
        fall_pc,
        Some(branch_pc),
        body_visited,
        depth + 1,
        fn_entry,
    );
    out.push_str(&body);

    let _ = writeln!(out, "{ind}end");

    // Continue after the loop
    let sub = emit_region(ctx, branch_pc, exit_pc, visited, depth, fn_entry);
    out.push_str(&sub);

    Some(out)
}

/// Emit a region of the CFG as Lua statements, returning them as a String.
///
/// - `fn_entry`: the PC of the outermost function being emitted.  When we reach
///   a block whose PC is a gosub target AND is not `fn_entry`, we emit a call to
///   that function instead of inlining it, then stop.
fn emit_region(
    ctx: &DecompileContext,
    start_pc: u32,
    exit_pc: Option<u32>,
    visited: &mut Vec<u32>,
    depth: usize,
    fn_entry: u32,
) -> String {
    let mut out = String::new();
    let current_pc = start_pc;
    let ind = indent(depth);

    // Stop at exit boundary
    if let Some(exit) = exit_pc {
        if current_pc == exit {
            return out;
        }
    }

    // Fix: if we've reached another gosub target's entry (not our own function),
    // emit a call to it and stop inlining.
    if current_pc != fn_entry && ctx.gosub_targets.contains(&current_pc) {
        let name = ctx.proc_name(current_pc);
        let _ = writeln!(out, "{ind}{name}()");
        return out;
    }

    // Stop if already visited (cycle guard)
    if visited.contains(&current_pc) {
        let label = ctx.label_name(current_pc);
        let _ = writeln!(out, "{ind}-- back-edge to {label}");
        return out;
    }
    visited.push(current_pc);

    let block = match ctx.blocks.get(&current_pc) {
        Some(b) => b,
        None => {
            let _ = writeln!(out, "{ind}-- missing block at 0x{current_pc:08X}");
            return out;
        }
    };

    // Pre-check: if this block is a while-loop header with non-terminal instructions,
    // emit the whole complex-while BEFORE any non-terminals are printed.
    // This prevents the condition computation from appearing outside the loop.
    if let Some(complex_while) =
        try_emit_complex_while(ctx, block, exit_pc, visited, depth, fn_entry)
    {
        out.push_str(&complex_while);
        return out;
    }

    // Virtual operand/argument stacks for reconstruction. Game extcalls consume
    // ordinary VM stack values; opcode 32/33 separately model the arg_base
    // stack used by arg_get and script procedure arguments.
    let mut stack: Vec<Operand> = Vec::new();
    let mut arg_stack: Vec<Operand> = Vec::new();

    // Emit non-terminal instructions
    let n = block.instrs.len();
    for (i, instr) in block.instrs.iter().enumerate() {
        let is_last = i + 1 == n;

        match instr {
            pal_script::Instruction::Primary {
                opcode: 31, args, ..
            } => {
                if let Some(pal_script::Argument::Operand(op)) = args.first() {
                    stack.push(*op);
                }
                let stmt = emit_single_instr_lua_full(
                    instr,
                    ctx.file_entries,
                    ctx.text_entries,
                    ctx.nls,
                    &[],
                    ctx.file_bytes,
                    ctx.text_bytes,
                );
                if !stmt.is_empty() && !is_last {
                    let _ = writeln!(out, "{ind}{stmt}");
                }
                if is_last {
                    out.push_str(&emit_terminal(
                        ctx,
                        block,
                        instr,
                        exit_pc,
                        visited,
                        depth,
                        &[],
                        fn_entry,
                    ));
                }
                continue;
            }
            pal_script::Instruction::Primary {
                opcode: 32, args, ..
            } => {
                if let Some(count) = static_count_arg(args) {
                    if count <= stack.len() {
                        let start = stack.len() - count;
                        arg_stack.extend(stack.drain(start..));
                    } else {
                        stack.clear();
                        arg_stack.clear();
                    }
                } else {
                    stack.clear();
                    arg_stack.clear();
                }
            }
            pal_script::Instruction::Primary {
                opcode: 33, args, ..
            } => {
                if let Some(count) = static_count_arg(args) {
                    if count <= arg_stack.len() {
                        arg_stack.truncate(arg_stack.len() - count);
                    } else {
                        arg_stack.clear();
                    }
                } else {
                    arg_stack.clear();
                }
            }
            // extcall: use arg_stack, then clear it
            pal_script::Instruction::Primary { opcode: 23, .. } => {
                let count = extcall_static_arg_count(instr, stack.len());
                let current_args = take_extcall_args(&mut stack, count);
                if is_last {
                    out.push_str(&emit_terminal(
                        ctx,
                        block,
                        instr,
                        exit_pc,
                        visited,
                        depth,
                        &current_args,
                        fn_entry,
                    ));
                } else {
                    let stmt = emit_single_instr_lua_full(
                        instr,
                        ctx.file_entries,
                        ctx.text_entries,
                        ctx.nls,
                        &current_args,
                        ctx.file_bytes,
                        ctx.text_bytes,
                    );
                    if !stmt.is_empty() {
                        let _ = writeln!(out, "{ind}{stmt}");
                    }
                }
                continue;
            }
            pal_script::Instruction::Primary { opcode: 30, .. } => {
                stack.pop();
            }
            _ => {}
        }

        if is_last {
            out.push_str(&emit_terminal(
                ctx,
                block,
                instr,
                exit_pc,
                visited,
                depth,
                &[],
                fn_entry,
            ));
            break;
        }

        let stmt = emit_single_instr_lua_full(
            instr,
            ctx.file_entries,
            ctx.text_entries,
            ctx.nls,
            &[],
            ctx.file_bytes,
            ctx.text_bytes,
        );
        if !stmt.is_empty() {
            let _ = writeln!(out, "{ind}{stmt}");
        }
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn emit_terminal(
    ctx: &DecompileContext,
    block: &Block,
    instr: &Instruction,
    exit_pc: Option<u32>,
    visited: &mut Vec<u32>,
    depth: usize,
    pending_args: &[Operand],
    fn_entry: u32,
) -> String {
    let mut out = String::new();
    let ind = indent(depth);

    match instr {
        Instruction::Primary { opcode, args, .. } => match *opcode {
            // jmp_point (9)
            9 => {
                if let Some(target) = extract_static_target_pc(args.first()) {
                    if visited.contains(&target) {
                        let label = ctx.label_name(target);
                        let _ = writeln!(out, "{ind}-- back-edge to {label}");
                    } else if exit_pc == Some(target) {
                        // Jump to exit — fall through naturally
                    } else {
                        let sub = emit_region(ctx, target, exit_pc, visited, depth, fn_entry);
                        out.push_str(&sub);
                    }
                } else {
                    // Dynamic jump: static target not resolved
                    match args.first() {
                        Some(Argument::Point { id, .. }) => {
                            let _ = writeln!(out, "{ind}dynamic_jump({id})");
                        }
                        Some(Argument::PointOperand { operand, .. }) => {
                            let expr = operand_to_lua(
                                operand,
                                ctx.file_entries,
                                ctx.text_entries,
                                ctx.nls,
                            );
                            let _ = writeln!(out, "{ind}dynamic_jump({expr})");
                        }
                        _ => {
                            let stmt = emit_single_instr_lua_full(
                                instr,
                                ctx.file_entries,
                                ctx.text_entries,
                                ctx.nls,
                                pending_args,
                                ctx.file_bytes,
                                ctx.text_bytes,
                            );
                            let _ = writeln!(out, "{ind}{stmt}");
                        }
                    }
                }
            }

            // jf_point (10)
            10 => {
                let (branch_point_id, branch_pc, cond_op) = match (args.first(), args.get(1)) {
                    (Some(Argument::Point { id, target_pc }), Some(Argument::Operand(cond))) => {
                        (*id, *target_pc, cond)
                    }
                    _ => {
                        let _ = writeln!(out, "{ind}-- jf (malformed)");
                        return out;
                    }
                };
                let fall_pc = instr_end_pc(instr); // where we go when condition is TRUE

                let branch_target = match branch_pc {
                    Some(pc) => pc,
                    None => {
                        let cond_expr =
                            operand_to_lua(cond_op, ctx.file_entries, ctx.text_entries, ctx.nls);
                        if branch_point_id == 0 {
                            // id=0: dynamic/empty branch target (runtime convention)
                            let _ = writeln!(out, "{ind}if {cond_expr} == 0 then");
                            let _ = writeln!(out, "{ind}    dynamic_jump(0)");
                            let _ = writeln!(out, "{ind}end");
                        } else {
                            // id > 0 but no target_pc: static ID not in Point.dat
                            let _ = writeln!(
                                out,
                                "{ind}-- INVALID jf target: point_id={branch_point_id} not in Point.dat"
                            );
                            let _ = writeln!(out, "{ind}if {cond_expr} == 0 then");
                            let _ = writeln!(out, "{ind}    jf_invalid({branch_point_id})");
                            let _ = writeln!(out, "{ind}end");
                        }
                        // Continue with fallthrough
                        let sub = emit_region(ctx, fall_pc, exit_pc, visited, depth, fn_entry);
                        out.push_str(&sub);
                        return out;
                    }
                };

                // Check for while loop: is there a back-edge from the body to current block's start?
                let header_pc = block.start_pc;
                let is_while = has_back_edge_to(header_pc, fall_pc, &ctx.blocks, 64);

                let cond_expr =
                    operand_to_lua(cond_op, ctx.file_entries, ctx.text_entries, ctx.nls);

                if is_while {
                    // while loop: condition in this block, exit at branch_target
                    // Emit ::header_label:: before the while for potential goto
                    let header_label = ctx.label_name(header_pc);
                    if needs_label(ctx, header_pc) {
                        let _ = writeln!(out, "{ind}::{header_label}::");
                    }

                    // Fix: if the header block has non-terminal instructions (condition
                    // computation), emit `while true do` and move them inside the loop.
                    let header_non_terms = &block.instrs[..block.instrs.len() - 1];

                    if header_non_terms.is_empty() {
                        // Simple while: condition is just a variable reference or literal
                        let _ = writeln!(out, "{ind}while {cond_expr} ~= 0 do");

                        let body_visited = &mut visited.clone();
                        if !body_visited.contains(&header_pc) {
                            body_visited.push(header_pc);
                        }
                        let body = emit_region(
                            ctx,
                            fall_pc,
                            Some(branch_target),
                            body_visited,
                            depth + 1,
                            fn_entry,
                        );
                        out.push_str(&body);

                        let _ = writeln!(out, "{ind}end");
                    } else {
                        // Complex while: condition requires computation in the header block.
                        // Emit `while true do` and re-emit the condition computation inside.
                        let _ = writeln!(out, "{ind}while true do");
                        let inner = indent(depth + 1);

                        // Re-emit header non-terminals inside the loop
                        let header_inner = emit_instrs_slice(header_non_terms, ctx, depth + 1);
                        out.push_str(&header_inner);

                        // Break condition: if cond == 0 then break end
                        let _ = writeln!(out, "{inner}if {cond_expr} == 0 then break end");

                        let body_visited = &mut visited.clone();
                        if !body_visited.contains(&header_pc) {
                            body_visited.push(header_pc);
                        }
                        let body = emit_region(
                            ctx,
                            fall_pc,
                            Some(branch_target),
                            body_visited,
                            depth + 1,
                            fn_entry,
                        );
                        out.push_str(&body);

                        let _ = writeln!(out, "{ind}end");
                    }

                    // Continue after the loop at branch_target
                    let sub = emit_region(ctx, branch_target, exit_pc, visited, depth, fn_entry);
                    out.push_str(&sub);
                } else {
                    // if/else pattern
                    let merge_pc = find_merge_point(fall_pc, branch_target, &ctx.blocks);

                    let _ = writeln!(out, "{ind}if {cond_expr} ~= 0 then");

                    // Then body (fallthrough when cond is true)
                    let then_visited = &mut visited.clone();
                    let then_body = emit_region(
                        ctx,
                        fall_pc,
                        Some(merge_pc),
                        then_visited,
                        depth + 1,
                        fn_entry,
                    );
                    out.push_str(&then_body);

                    // Else body (branch when cond is false)
                    if branch_target != merge_pc {
                        let _ = writeln!(out, "{ind}else");
                        let else_visited = &mut visited.clone();
                        let else_body = emit_region(
                            ctx,
                            branch_target,
                            Some(merge_pc),
                            else_visited,
                            depth + 1,
                            fn_entry,
                        );
                        out.push_str(&else_body);
                    }

                    let _ = writeln!(out, "{ind}end");

                    // Continue at merge point
                    let sub = emit_region(ctx, merge_pc, exit_pc, visited, depth, fn_entry);
                    out.push_str(&sub);
                }
            }

            // gosub_point (11)
            11 => {
                let fall_pc = instr_end_pc(instr);
                match args.first() {
                    Some(Argument::Point {
                        target_pc: Some(tgt),
                        ..
                    }) => {
                        let func_name = ctx.proc_name(*tgt);
                        let _ = writeln!(out, "{ind}{func_name}()");
                    }
                    Some(Argument::Point {
                        id: 0,
                        target_pc: None,
                    }) => {
                        // id=0 convention: dynamic/empty gosub target
                        let _ = writeln!(out, "{ind}-- gosub: dynamic/empty point_id=0");
                    }
                    Some(Argument::Point {
                        id,
                        target_pc: None,
                    }) => {
                        // id > 0 but no target_pc: static ID not found in Point.dat
                        let _ = writeln!(
                            out,
                            "{ind}-- INVALID gosub target: point_id={id} not in Point.dat"
                        );
                        let _ = writeln!(out, "{ind}gosub_invalid({id})");
                    }
                    _ => {
                        let _ = writeln!(out, "{ind}-- gosub (malformed)");
                    }
                }
                // Continue with fallthrough
                let sub = emit_region(ctx, fall_pc, exit_pc, visited, depth, fn_entry);
                out.push_str(&sub);
            }

            // end / ret
            21 | 24 => {
                let _ = writeln!(out, "{ind}return");
            }

            // Everything else — emit as statement, then continue with successor
            _ => {
                let stmt = emit_single_instr_lua_full(
                    instr,
                    ctx.file_entries,
                    ctx.text_entries,
                    ctx.nls,
                    pending_args,
                    ctx.file_bytes,
                    ctx.text_bytes,
                );
                if !stmt.is_empty() {
                    let _ = writeln!(out, "{ind}{stmt}");
                }
                let next_pc = instr_end_pc(instr);
                let sub = emit_region(ctx, next_pc, exit_pc, visited, depth, fn_entry);
                out.push_str(&sub);
            }
        },
        Instruction::DataWord { .. } => {
            let stmt = emit_single_instr_lua_full(
                instr,
                ctx.file_entries,
                ctx.text_entries,
                ctx.nls,
                pending_args,
                ctx.file_bytes,
                ctx.text_bytes,
            );
            if !stmt.is_empty() {
                let _ = writeln!(out, "{ind}{stmt}");
            }
            let next_pc = instr_end_pc(instr);
            let sub = emit_region(ctx, next_pc, exit_pc, visited, depth, fn_entry);
            out.push_str(&sub);
        }
    }

    out
}

/// Check whether any block reachable from `body_start` (without crossing `limit`)
/// has a successor that points back to `header_pc`.
fn has_back_edge_to(
    header_pc: u32,
    body_start: u32,
    blocks: &BTreeMap<u32, Block>,
    max_depth: usize,
) -> bool {
    let mut stack = vec![(body_start, 0usize)];
    let mut seen = vec![body_start];

    while let Some((pc, depth)) = stack.pop() {
        if depth >= max_depth {
            continue;
        }
        if let Some(block) = blocks.get(&pc) {
            for &succ in &block.succs {
                if succ == header_pc {
                    return true;
                }
                if succ > header_pc && !seen.contains(&succ) {
                    seen.push(succ);
                    stack.push((succ, depth + 1));
                }
            }
        }
    }
    false
}

/// Find the merge/convergence point for an if/else pattern.
///
/// Heuristic: walk the fallthrough path until we either:
///   a) hit a `jmp` to branch_pc  → merge is branch_pc
///   b) reach branch_pc itself     → no else, merge is branch_pc
fn find_merge_point(fall_pc: u32, branch_pc: u32, blocks: &BTreeMap<u32, Block>) -> u32 {
    // Walk from fall_pc; if we find a jmp whose target == branch_pc, that jmp's
    // target IS the merge.  If fall_pc is already >= branch_pc, merge = branch_pc.
    if fall_pc >= branch_pc {
        return branch_pc;
    }

    let mut current = fall_pc;
    let mut depth = 0usize;
    loop {
        if depth > 256 {
            return branch_pc;
        }
        depth += 1;

        let block = match blocks.get(&current) {
            Some(b) => b,
            None => return branch_pc,
        };

        if let Some(last) = block.last_instr() {
            match last {
                Instruction::Primary {
                    opcode: 9, args, ..
                } => {
                    // jmp: check target
                    if let Some(Argument::Point {
                        target_pc: Some(tgt),
                        ..
                    }) = args.first()
                    {
                        if *tgt >= branch_pc {
                            return *tgt;
                        }
                        // jump forward but not past branch_pc; keep walking
                        current = *tgt;
                        continue;
                    }
                    return branch_pc;
                }
                Instruction::Primary {
                    opcode: 21 | 24, ..
                } => {
                    // end/ret — no merge, use branch_pc
                    return branch_pc;
                }
                _ => {
                    // fallthrough
                    let end = instr_end_pc(last);
                    if end >= branch_pc {
                        return branch_pc;
                    }
                    current = end;
                }
            }
        } else {
            return branch_pc;
        }
    }
}

/// Whether a given PC needs a `::label::` emitted before its block.
fn needs_label(ctx: &DecompileContext, pc: u32) -> bool {
    // For now: only if it's a back-edge target (i.e. a loop header)
    // Simple heuristic: check if any block in the CFG has a back-edge to pc
    for block in ctx.blocks.values() {
        for &succ in &block.succs {
            if succ == pc && block.start_pc >= pc {
                return true;
            }
        }
    }
    false
}
