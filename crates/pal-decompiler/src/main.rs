//! PAL script decompiler — produces Lua source from Sv20 bytecode.

mod assets;
mod cfg;
mod codegen;
mod dat;
mod decompile;
mod strings;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use pal_asset::Nls;
use pal_script::extsig::{
    all_signatures, auto_signatures, lookup_sig, observed_pop_count, ImplStatus, ReturnKind,
};
use pal_script::opcodes::ext_opcode;
use pal_script::{
    Argument, DisassembleOptions, Instruction, Operand, ScriptImage, SCRIPT_CODE_BASE,
};

use assets::ScriptAssets;

// ─── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Parser)]
#[command(name = "pal-decompiler")]
#[command(about = "Decompile PAL Sv20 script bytecode to Lua")]
struct Cli {
    /// Read all 5 script assets from a single PAC archive
    #[arg(long = "input-pac", value_name = "PATH")]
    input_pac: Option<PathBuf>,

    /// Path to SCRIPT.SRC (unencrypted)
    #[arg(long = "input-script", value_name = "PATH")]
    input_script: Option<PathBuf>,

    /// Path to POINT.DAT (encrypted)
    #[arg(long = "input-point", value_name = "PATH")]
    input_point: Option<PathBuf>,

    /// Path to FILE.DAT (encrypted)
    #[arg(long = "input-file-dat", value_name = "PATH")]
    input_file_dat: Option<PathBuf>,

    /// Path to TEXT.DAT (encrypted)
    #[arg(long = "input-text-dat", value_name = "PATH")]
    input_text_dat: Option<PathBuf>,

    /// Path to MEM.DAT (encrypted)
    #[arg(long = "input-mem-dat", value_name = "PATH")]
    input_mem_dat: Option<PathBuf>,

    /// NLS encoding for strings (sjis | gbk | utf-8)
    #[arg(long, default_value = "sjis")]
    nls: String,

    /// Output Lua file path
    #[arg(long, value_name = "PATH")]
    output: PathBuf,

    /// Optional JSON coverage/sequence report for script extcalls.
    #[arg(long = "extcall-report", value_name = "PATH")]
    extcall_report: Option<PathBuf>,

    /// Optional DEBUG_VM text log to compare against the decompiler extcall sequence.
    #[arg(long = "runtime-trace-log", value_name = "PATH")]
    runtime_trace_log: Option<PathBuf>,

    /// Optional JSON output for runtime trace vs decompiler extcall comparison.
    #[arg(long = "trace-compare-output", value_name = "PATH")]
    trace_compare_output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let nls: Nls = cli
        .nls
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;

    // Validate input mode
    let has_pac = cli.input_pac.is_some();
    let has_explicit = cli.input_script.is_some()
        || cli.input_point.is_some()
        || cli.input_file_dat.is_some()
        || cli.input_text_dat.is_some()
        || cli.input_mem_dat.is_some();

    if has_pac && has_explicit {
        bail!("cannot mix --input-pac with --input-script/--input-point/etc.");
    }
    if !has_pac && !has_explicit {
        bail!("must provide either --input-pac or all explicit --input-* flags");
    }

    let assets = if has_pac {
        let pac_path = cli.input_pac.as_ref().unwrap();
        ScriptAssets::from_pac(pac_path, nls).with_context(|| {
            format!(
                "failed to load script assets from PAC {}",
                pac_path.display()
            )
        })?
    } else {
        let script_path = cli
            .input_script
            .as_ref()
            .context("--input-script is required when not using --input-pac")?;
        let point_path = cli
            .input_point
            .as_ref()
            .context("--input-point is required when not using --input-pac")?;
        let file_dat_path = cli
            .input_file_dat
            .as_ref()
            .context("--input-file-dat is required when not using --input-pac")?;
        let text_dat_path = cli
            .input_text_dat
            .as_ref()
            .context("--input-text-dat is required when not using --input-pac")?;
        let mem_dat_path = cli
            .input_mem_dat
            .as_ref()
            .context("--input-mem-dat is required when not using --input-pac")?;

        ScriptAssets::from_files(
            script_path,
            point_path,
            file_dat_path,
            text_dat_path,
            mem_dat_path,
            nls,
        )
        .context("failed to load script assets from explicit files")?
    };

    let lua = decompile::decompile(&assets, nls).context("decompilation failed")?;

    fs::write(&cli.output, lua.as_bytes())
        .with_context(|| format!("failed to write output to {}", cli.output.display()))?;

    let ext_report = analyze_extcalls(&assets).context("analyze extcall coverage")?;
    if let Some(path) = cli.extcall_report.as_ref() {
        fs::write(path, ext_report.to_json().as_bytes())
            .with_context(|| format!("failed to write extcall report to {}", path.display()))?;
        eprintln!("wrote {}", path.display());
    }

    if let Some(trace_log) = cli.runtime_trace_log.as_ref() {
        let trace_text = fs::read_to_string(trace_log)
            .with_context(|| format!("failed to read runtime trace {}", trace_log.display()))?;
        let compare = compare_runtime_trace(&ext_report.sequence, &trace_text);
        let path = cli
            .trace_compare_output
            .clone()
            .unwrap_or_else(|| PathBuf::from("out/extcall_trace_compare.json"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create trace compare dir {}", parent.display())
            })?;
        }
        fs::write(&path, compare.to_json().as_bytes()).with_context(|| {
            format!(
                "failed to write runtime/decompiler compare to {}",
                path.display()
            )
        })?;
        eprintln!("wrote {}", path.display());
    }

    eprintln!("wrote {}", cli.output.display());
    Ok(())
}

#[derive(Debug)]
struct ExtcallReport {
    total_opcode_table: usize,
    total_known_table: usize,
    unique_reachable: usize,
    reversed: usize,
    verified: usize,
    blocked: usize,
    partial: usize,
    heuristic: usize,
    auto_fallback: usize,
    stack_discipline_only: usize,
    stub: usize,
    unknown: usize,
    stack_mismatch: usize,
    static_arg_unresolved: usize,
    sequence: Vec<ExtcallEvent>,
}

#[derive(Clone, Debug)]
struct ExtcallEvent {
    ordinal: usize,
    pc: u32,
    category: u16,
    index: u16,
    name: String,
    arg_count: usize,
    expected_arg_count: Option<usize>,
    return_kind: String,
    status: String,
    stack_mismatch: bool,
    static_arg_unresolved: bool,
    args: Vec<String>,
}

#[derive(Debug)]
struct TraceCompare {
    decompiler_count: usize,
    runtime_count: usize,
    matched_prefix: usize,
    ordinal_mismatches: usize,
    pc_matched: usize,
    pc_mismatches: usize,
    mismatches: Vec<String>,
}

#[derive(Clone, Debug)]
struct RuntimeExtcallEvent {
    pc: Option<u32>,
    category: u16,
    index: u16,
}

fn analyze_extcalls(assets: &ScriptAssets) -> Result<ExtcallReport> {
    let script = ScriptImage::parse(&assets.script_bytes).context("parse SCRIPT.SRC header")?;
    let options = DisassembleOptions {
        start: SCRIPT_CODE_BASE as usize,
        end: None,
        point_table: None,
    };
    let instructions = pal_script::disassemble_script(&script, options)
        .context("disassemble SCRIPT.SRC for extcall report")?;

    let mut sequence = Vec::new();
    let mut stack: Vec<Operand> = Vec::new();
    let mut arg_stack: Vec<Operand> = Vec::new();
    for instr in &instructions {
        match instr {
            Instruction::Primary {
                opcode: 31, args, ..
            } => {
                if let Some(Argument::Operand(op)) = args.first() {
                    stack.push(*op);
                }
            }
            Instruction::Primary {
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
            Instruction::Primary {
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
            Instruction::Primary {
                opcode: 23,
                pc,
                args,
                ..
            } => {
                if let Some(Argument::ExtCall {
                    category,
                    index,
                    name,
                    ..
                }) = args.first()
                {
                    let sig = lookup_sig(*category, *index);
                    let expected_or_observed = sig
                        .map(|s| s.pop_count)
                        .or_else(|| observed_pop_count(*category, *index));
                    let count = expected_or_observed.unwrap_or(arg_stack.len());
                    // Game.exe extcalls pop their immediate arguments from the
                    // ordinary VM stack. `pack_args/drop_args` maintain the
                    // separate argument stack used by arg_base/arg_get-style
                    // helpers; treating that stack as the extcall source made
                    // direct `push ...; extcall` sites look like zero-arg calls.
                    let selected_args = take_extcall_args(&mut stack, count);
                    let pop_args: Vec<Operand> = selected_args.iter().rev().copied().collect();
                    let name = sig
                        .map(|s| s.name)
                        .or(*name)
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("ext_{category:04X}_{index:04X}"));
                    let expected = sig.map(|s| s.pop_count);
                    let status = sig
                        .map(|s| status_name(s.decompiler_status).to_owned())
                        .unwrap_or_else(|| "Unknown".to_owned());
                    let return_kind = sig
                        .map(|s| return_kind_name(s.return_kind).to_owned())
                        .unwrap_or_else(|| "UnsupportedUnknown".to_owned());
                    sequence.push(ExtcallEvent {
                        ordinal: sequence.len(),
                        pc: *pc,
                        category: *category,
                        index: *index,
                        name,
                        arg_count: pop_args.len(),
                        expected_arg_count: expected,
                        return_kind,
                        status,
                        stack_mismatch: false,
                        static_arg_unresolved: expected.is_some_and(|n| n != pop_args.len()),
                        args: pop_args.iter().map(format_operand_raw).collect(),
                    });
                }
            }
            Instruction::Primary { opcode: 30, .. } => {
                stack.pop();
            }
            _ => {}
        }
    }

    let unique: BTreeSet<(u16, u16)> = sequence.iter().map(|e| (e.category, e.index)).collect();
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (cat, idx) in &unique {
        let key = lookup_sig(*cat, *idx)
            .map(|s| status_name(s.decompiler_status))
            .unwrap_or("Unknown");
        *counts.entry(key).or_default() += 1;
    }

    Ok(ExtcallReport {
        total_opcode_table: count_ext_opcode_table(),
        total_known_table: all_signatures().len() + auto_signatures().len(),
        unique_reachable: unique.len(),
        reversed: unique
            .iter()
            .filter(|(cat, idx)| lookup_sig(*cat, *idx).is_some())
            .count(),
        verified: *counts.get("Verified").unwrap_or(&0),
        blocked: *counts.get("Blocked").unwrap_or(&0),
        partial: *counts.get("Partial").unwrap_or(&0),
        heuristic: 0,
        auto_fallback: 0,
        stack_discipline_only: *counts.get("StackDisciplineOnly").unwrap_or(&0),
        stub: *counts.get("Stub").unwrap_or(&0)
            + *counts.get("StackDisciplineOnly").unwrap_or(&0)
            + *counts.get("WrongOrSuspicious").unwrap_or(&0),
        unknown: *counts.get("Unknown").unwrap_or(&0),
        stack_mismatch: sequence.iter().filter(|e| e.stack_mismatch).count(),
        static_arg_unresolved: sequence.iter().filter(|e| e.static_arg_unresolved).count(),
        sequence,
    })
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

fn static_count_arg(args: &[Argument]) -> Option<usize> {
    let op = match args.first()? {
        Argument::Operand(op) => op,
        _ => return None,
    };
    (op.kind == pal_script::OperandKind::Immediate)
        .then_some(op.raw as i32)
        .filter(|count| *count >= 0)
        .map(|count| count as usize)
}

fn compare_runtime_trace(decompiler: &[ExtcallEvent], trace_text: &str) -> TraceCompare {
    let runtime = parse_runtime_extcalls(trace_text);

    let mut matched_prefix = 0;
    let mut ordinal_mismatches = 0;
    let mut mismatches = Vec::new();
    for (i, rt) in runtime.iter().enumerate() {
        let Some(dc) = decompiler.get(i) else {
            ordinal_mismatches += 1;
            continue;
        };
        if (dc.category, dc.index) == (rt.category, rt.index) {
            matched_prefix += 1;
        } else {
            ordinal_mismatches += 1;
        }
    }

    let by_pc: BTreeMap<u32, &ExtcallEvent> =
        decompiler.iter().map(|event| (event.pc, event)).collect();
    let mut pc_matched = 0usize;
    let mut pc_mismatches = 0usize;
    for (i, rt) in runtime.iter().enumerate() {
        let Some(pc) = rt.pc else {
            pc_mismatches += 1;
            mismatches.push(format!(
                "pc runtime #{i}: no opcode pc for ext_{:04X}_{:04X}",
                rt.category, rt.index
            ));
            continue;
        };
        match by_pc.get(&pc) {
            Some(dc) if (dc.category, dc.index) == (rt.category, rt.index) => pc_matched += 1,
            Some(dc) => {
                pc_mismatches += 1;
                mismatches.push(format!(
                    "pc runtime #{i}: pc=0x{pc:08X} ext_{:04X}_{:04X} != decompiler ext_{:04X}_{:04X}",
                    rt.category, rt.index, dc.category, dc.index
                ));
            }
            None => {
                pc_mismatches += 1;
                mismatches.push(format!(
                    "pc runtime #{i}: pc=0x{pc:08X} ext_{:04X}_{:04X} has no static extcall",
                    rt.category, rt.index
                ));
            }
        }
    }

    TraceCompare {
        decompiler_count: decompiler.len(),
        runtime_count: runtime.len(),
        matched_prefix,
        ordinal_mismatches,
        pc_matched,
        pc_mismatches,
        mismatches,
    }
}

fn parse_runtime_extcalls(trace_text: &str) -> Vec<RuntimeExtcallEvent> {
    let mut current_pc = None;
    let mut out = Vec::new();
    for line in trace_text.lines() {
        if line.contains("opcode pc=0x") {
            current_pc = parse_hex_after(line, "opcode pc=0x");
            continue;
        }
        let Some(marker) = line.find(" op extcall ") else {
            continue;
        };
        let rest = &line[marker..];
        let Some(ext_pos) = rest.find("ext_") else {
            continue;
        };
        let Some(raw) = rest.get(ext_pos + 4..ext_pos + 13) else {
            continue;
        };
        let Some((cat, idx)) = raw.split_once('_') else {
            continue;
        };
        let (Ok(category), Ok(index)) =
            (u16::from_str_radix(cat, 16), u16::from_str_radix(idx, 16))
        else {
            continue;
        };
        out.push(RuntimeExtcallEvent {
            pc: current_pc,
            category,
            index,
        });
    }
    out
}

fn parse_hex_after(line: &str, marker: &str) -> Option<u32> {
    let start = line.find(marker)? + marker.len();
    let hex = line.get(start..start + 8)?;
    u32::from_str_radix(hex, 16).ok()
}

fn count_ext_opcode_table() -> usize {
    let mut count = 0usize;
    for category in 0..=23u16 {
        for index in 0..=0xFFFFu16 {
            if ext_opcode(category, index).is_some() {
                count += 1;
            }
        }
    }
    count
}

fn format_operand_raw(op: &Operand) -> String {
    format!("0x{:08X}", op.raw)
}

fn status_name(status: ImplStatus) -> &'static str {
    match status {
        ImplStatus::Verified => "Verified",
        ImplStatus::Partial => "Partial",
        ImplStatus::Blocked => "Blocked",
        ImplStatus::StackDisciplineOnly => "StackDisciplineOnly",
        ImplStatus::Stub => "Stub",
        ImplStatus::Unknown => "Unknown",
        ImplStatus::WrongOrSuspicious => "WrongOrSuspicious",
    }
}

fn return_kind_name(kind: ReturnKind) -> &'static str {
    match kind {
        ReturnKind::Void => "Void",
        ReturnKind::Integer => "Integer",
        ReturnKind::Bool => "Bool",
        ReturnKind::Handle => "Handle",
        ReturnKind::StringId => "StringId",
        ReturnKind::Status => "Status",
        ReturnKind::PointId => "PointId",
        ReturnKind::UnsupportedUnknown => "UnsupportedUnknown",
    }
}

fn json_escape(s: &str) -> String {
    s.chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect(),
            '\n' => "\\n".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\t' => "\\t".chars().collect(),
            c if c.is_control() => format!("\\u{:04X}", c as u32).chars().collect(),
            c => vec![c],
        })
        .collect()
}

impl ExtcallReport {
    fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n  \"coverage\": {\n");
        out.push_str(&format!(
            "    \"total_opcode_table\": {},\n    \"total_known_table\": {},\n    \"unique_reachable\": {},\n    \"reversed\": {},\n    \"verified\": {},\n    \"blocked\": {},\n    \"partial\": {},\n    \"heuristic\": {},\n    \"auto_fallback\": {},\n    \"stack_discipline_only\": {},\n    \"stub\": {},\n    \"unknown\": {},\n    \"stack_mismatch\": {},\n    \"static_arg_unresolved\": {}\n",
            self.total_opcode_table,
            self.total_known_table,
            self.unique_reachable,
            self.reversed,
            self.verified,
            self.blocked,
            self.partial,
            self.heuristic,
            self.auto_fallback,
            self.stack_discipline_only,
            self.stub,
            self.unknown,
            self.stack_mismatch,
            self.static_arg_unresolved
        ));
        out.push_str("  },\n  \"sequence\": [\n");
        for (i, e) in self.sequence.iter().enumerate() {
            if i > 0 {
                out.push_str(",\n");
            }
            out.push_str(&format!(
                "    {{\"ordinal\":{},\"pc\":\"0x{:08X}\",\"category\":{},\"index\":{},\"name\":\"{}\",\"arg_count\":{},\"expected_arg_count\":{},\"return_kind\":\"{}\",\"status\":\"{}\",\"stack_mismatch\":{},\"static_arg_unresolved\":{},\"args\":[{}]}}",
                e.ordinal,
                e.pc,
                e.category,
                e.index,
                json_escape(&e.name),
                e.arg_count,
                e.expected_arg_count
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "null".to_owned()),
                json_escape(&e.return_kind),
                json_escape(&e.status),
                e.stack_mismatch,
                e.static_arg_unresolved,
                e.args
                    .iter()
                    .map(|arg| format!("\"{}\"", json_escape(arg)))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
        out.push_str("\n  ]\n}\n");
        out
    }
}

impl TraceCompare {
    fn to_json(&self) -> String {
        format!(
            "{{\n  \"decompiler_count\": {},\n  \"runtime_count\": {},\n  \"matched_prefix\": {},\n  \"ordinal_mismatches\": {},\n  \"pc_matched\": {},\n  \"pc_mismatches\": {},\n  \"mismatches\": [{}]\n}}\n",
            self.decompiler_count,
            self.runtime_count,
            self.matched_prefix,
            self.ordinal_mismatches,
            self.pc_matched,
            self.pc_mismatches,
            self.mismatches
                .iter()
                .map(|m| format!("\"{}\"", json_escape(m)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use pal_asset::Nls;
    use pal_script::{Argument, Instruction, Operand, OperandKind, ScriptImage, SCRIPT_CODE_BASE};

    use crate::cfg::{build_cfg, Block};
    use crate::codegen::{emit_lua, emit_single_instr_lua, emit_single_instr_lua_full};
    use crate::dat::{parse_file_dat, parse_text_dat};
    use crate::decompile::DecompileContext;
    use crate::strings::{lua_escape, resolve_resource_str, resolve_string_id, resolve_text_str};

    // ── helpers ───────────────────────────────────────────────────────────────

    fn make_primary(pc: u32, opcode: u16, args: Vec<Argument>) -> Instruction {
        let n_words = 1u32 + args.len() as u32;
        Instruction::Primary {
            pc,
            end_pc: pc + n_words * 4,
            word: 0x0001_0000 | opcode as u32,
            opcode,
            meta: pal_script::opcodes::primary_opcode(opcode),
            args,
        }
    }

    fn imm_arg(n: i32) -> Argument {
        Argument::Operand(Operand {
            raw: n as u32,
            kind: OperandKind::Immediate,
            lo: (n & 0xFFFF) as u16,
            bank: 0,
        })
    }

    fn var_arg(slot: u16) -> Argument {
        Argument::Operand(Operand {
            raw: 0x4000_0000 | slot as u32,
            kind: OperandKind::VariableSlot,
            lo: slot,
            bank: 0,
        })
    }

    fn point_arg(id: u32, target_pc: Option<u32>) -> Argument {
        Argument::Point { id, target_pc }
    }

    fn imm_op(n: i32) -> Operand {
        Operand {
            raw: n as u32,
            kind: OperandKind::Immediate,
            lo: (n & 0xFFFF) as u16,
            bank: 0,
        }
    }

    fn literal_slot_op(raw: u32) -> Operand {
        Operand {
            raw,
            kind: OperandKind::LiteralSlot,
            lo: (raw & 0xFFFF) as u16,
            bank: ((raw >> 16) & 0x0FFF) as u16,
        }
    }

    /// Build a minimal synthetic FILE.DAT with `slots` entries.
    /// slots: list of (slot_index, name) pairs.
    fn make_file_dat(slots: &[(usize, &str)]) -> Vec<u8> {
        const HEADER: usize = 0x10;
        const SLOT_SIZE: usize = 0x20;
        let max_slot = slots.iter().map(|(i, _)| *i).max().unwrap_or(0);
        let n = max_slot + 1;
        let mut dat = vec![0u8; HEADER + n * SLOT_SIZE];
        for (idx, name) in slots {
            let offset = HEADER + idx * SLOT_SIZE;
            let bytes = name.as_bytes();
            let len = bytes.len().min(SLOT_SIZE - 1);
            dat[offset..offset + len].copy_from_slice(&bytes[..len]);
        }
        dat
    }

    /// Build a minimal synthetic TEXT.DAT with the given entries.
    /// entries: list of (key, string) pairs.
    fn make_text_dat(entries: &[(u32, &str)]) -> Vec<u8> {
        let mut dat = Vec::new();
        dat.extend_from_slice(b"$TEXT_LIST__");
        let count = entries.len() as u32;
        dat.extend_from_slice(&count.to_le_bytes());
        for (key, s) in entries {
            dat.extend_from_slice(&key.to_le_bytes());
            dat.extend_from_slice(s.as_bytes());
            dat.push(0); // NUL terminator
        }
        dat
    }

    fn make_extcall_instr(pc: u32, cat: u16, idx: u16, dst_slot: u32) -> Instruction {
        let ext_arg = Argument::ExtCall {
            raw: ((cat as u32) << 16) | idx as u32,
            category: cat,
            index: idx,
            name: pal_script::opcodes::ext_opcode(cat, idx).and_then(|e| e.name),
        };
        let dst_arg = Argument::DestinationSlot {
            slot: dst_slot,
            raw: dst_slot,
        };
        Instruction::Primary {
            pc,
            end_pc: pc + 12,
            word: 0x0001_0000 | 23u32,
            opcode: 23,
            meta: pal_script::opcodes::primary_opcode(23),
            args: vec![ext_arg, dst_arg],
        }
    }

    // ── test_lua_escape ───────────────────────────────────────────────────────

    #[test]
    fn test_lua_escape() {
        assert_eq!(lua_escape("hello"), "hello");
        assert_eq!(lua_escape("say \"hi\""), r#"say \"hi\""#);
        assert_eq!(lua_escape("back\\slash"), r"back\\slash");
        assert_eq!(lua_escape("line\nfeed"), r"line\nfeed");
        assert_eq!(lua_escape("tab\there"), r"tab\there");
        // non-printable byte
        let s = lua_escape("\x01");
        assert_eq!(s, "\\1");
    }

    // ── test_file_dat_slot_rule ───────────────────────────────────────────────

    #[test]
    fn test_file_dat_slot_rule() {
        let dat = make_file_dat(&[(0, "EMPTY"), (1, "BGM01"), (2, "BK000D")]);
        let entries = parse_file_dat(&dat, Nls::ShiftJis).unwrap();
        assert_eq!(entries[0], "EMPTY");
        assert_eq!(entries[1], "BGM01");
        assert_eq!(entries[2], "BK000D");
    }

    // ── test_file_dat_boundary ────────────────────────────────────────────────

    #[test]
    fn test_file_dat_boundary() {
        let dat = vec![0u8; 0x10 + 2 * 0x20]; // only 2 entries
        let entries = parse_file_dat(&dat, Nls::ShiftJis).unwrap();
        // index 5 is out of range → should fail gracefully in resolve_string_id
        let result = resolve_string_id(5, &entries, &[], None);
        // Out-of-range id returns a fallback string
        assert!(result.contains("file_str") || result.contains("5"));
    }

    // ── test_empty_string_id ─────────────────────────────────────────────────

    #[test]
    fn test_empty_string_id() {
        let s = resolve_string_id(0x0FFFFFFF, &[], &[], None);
        assert_eq!(s, "\"\"");
    }

    // ── test_dynamic_string ───────────────────────────────────────────────────

    #[test]
    fn test_dynamic_string() {
        let s = resolve_string_id(0x10000001, &[], &[], None);
        assert_eq!(s, "dynamic_string(0x10000001)");
    }

    // ── test_dynamic_jump ─────────────────────────────────────────────────────

    #[test]
    fn test_dynamic_jump() {
        // opcode 9 with point_id=0 (no static target) -> dynamic jump helper.
        let instr = make_primary(
            SCRIPT_CODE_BASE,
            9,
            vec![point_arg(0, None)], // id=0 means dynamic / unresolved
        );
        let output = emit_single_instr_lua(&instr, &[], &[], None, &[]);
        assert!(
            output.contains("dynamic_jump"),
            "expected dynamic_jump, got: {output}"
        );
    }

    #[test]
    fn test_dynamic_jump_operand_is_explicit() {
        let instr = make_primary(SCRIPT_CODE_BASE, 9, vec![var_arg(12)]);
        let output = emit_single_instr_lua(&instr, &[], &[], None, &[]);
        assert!(
            output.contains("dynamic_jump("),
            "expected explicit dynamic_jump, got: {output}"
        );
        assert!(
            !output.contains("-- jmp dynamic"),
            "dynamic jump must not be comment-only, got: {output}"
        );
    }

    // ── test_if_pattern ───────────────────────────────────────────────────────

    #[test]
    fn test_if_pattern() {
        let mut bytes = vec![0u8; 56];
        bytes[0..4].copy_from_slice(b"Sv20");
        bytes[8..12].copy_from_slice(&12u32.to_le_bytes());

        fn w(opcode: u32, hi: u32) -> [u8; 4] {
            ((hi << 16) | opcode).to_le_bytes()
        }
        fn imm(n: i32) -> [u8; 4] {
            (n as u32).to_le_bytes()
        }
        fn var(slot: u16) -> [u8; 4] {
            (0x4000_0000u32 | slot as u32).to_le_bytes()
        }
        fn point_word(id: u32) -> [u8; 4] {
            id.to_le_bytes()
        }

        let mut pos = 12usize;
        bytes[pos..pos + 4].copy_from_slice(&w(12, 1));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&var(0));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&imm(1));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(10, 1));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&point_word(1));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&var(0));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(1, 1));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&var(1));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&imm(99));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(22, 1));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(21, 1));

        let mut pt_bytes = vec![0u8; 4];
        pt_bytes[0..4].copy_from_slice(&36u32.to_le_bytes());

        let pt = pal_script::PointTable::parse(&pt_bytes).unwrap();
        let script = ScriptImage::parse(&bytes).unwrap();
        let options = pal_script::DisassembleOptions {
            start: 12,
            end: None,
            point_table: Some(&pt),
        };
        let instructions = pal_script::disassemble_script(&script, options).unwrap();

        let cfg = build_cfg(&instructions, script.entry_pc());
        let ctx = DecompileContext::new_simple(cfg, &pt, &[], &[], None);
        let lua = emit_lua(&ctx, script.entry_pc());

        assert!(
            lua.contains("if ") || lua.contains("-- if"),
            "expected if pattern, got:\n{lua}"
        );
        assert!(
            lua.contains("v1"),
            "expected v1 assignment in then-body, got:\n{lua}"
        );
    }

    // ── test_while_pattern ────────────────────────────────────────────────────

    #[test]
    fn test_while_pattern() {
        let mut bytes = vec![0u8; 12 + 40];
        bytes[0..4].copy_from_slice(b"Sv20");
        bytes[8..12].copy_from_slice(&12u32.to_le_bytes());

        fn w(opcode: u32) -> [u8; 4] {
            ((1u32 << 16) | opcode).to_le_bytes()
        }
        fn var(slot: u16) -> [u8; 4] {
            (0x4000_0000u32 | slot as u32).to_le_bytes()
        }
        fn imm(n: i32) -> [u8; 4] {
            (n as u32).to_le_bytes()
        }
        fn pt(id: u32) -> [u8; 4] {
            id.to_le_bytes()
        }

        let mut pos = 12usize;
        bytes[pos..pos + 4].copy_from_slice(&w(12));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&var(0));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&imm(0));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(10));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&pt(1));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&var(0));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(9));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&pt(2));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(21));

        let mut pt_bytes = vec![0u8; 8];
        pt_bytes[0..4].copy_from_slice(&0u32.to_le_bytes());
        pt_bytes[4..8].copy_from_slice(&32u32.to_le_bytes());

        let pt = pal_script::PointTable::parse(&pt_bytes).unwrap();
        let script = ScriptImage::parse(&bytes).unwrap();
        let options = pal_script::DisassembleOptions {
            start: 12,
            end: None,
            point_table: Some(&pt),
        };
        let instructions = pal_script::disassemble_script(&script, options).unwrap();

        let cfg = build_cfg(&instructions, script.entry_pc());
        let ctx = DecompileContext::new_simple(cfg, &pt, &[], &[], None);
        let lua = emit_lua(&ctx, script.entry_pc());
        assert!(
            lua.contains("while ") || lua.contains("-- while"),
            "expected while pattern, got:\n{lua}"
        );
    }

    // ── test_gosub_function ───────────────────────────────────────────────────

    #[test]
    fn test_gosub_function() {
        let mut bytes = vec![0u8; 12 + 20];
        bytes[0..4].copy_from_slice(b"Sv20");
        bytes[8..12].copy_from_slice(&12u32.to_le_bytes());

        fn w(opcode: u32) -> [u8; 4] {
            ((1u32 << 16) | opcode).to_le_bytes()
        }
        fn pt(id: u32) -> [u8; 4] {
            id.to_le_bytes()
        }

        let mut pos = 12usize;
        bytes[pos..pos + 4].copy_from_slice(&w(11));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&pt(1));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(21));
        pos += 4;

        bytes[pos..pos + 4].copy_from_slice(&w(24));

        let mut pt_bytes = vec![0u8; 4];
        pt_bytes[0..4].copy_from_slice(&12u32.to_le_bytes());

        let pt = pal_script::PointTable::parse(&pt_bytes).unwrap();
        let script = ScriptImage::parse(&bytes).unwrap();
        let options = pal_script::DisassembleOptions {
            start: 12,
            end: None,
            point_table: Some(&pt),
        };
        let instructions = pal_script::disassemble_script(&script, options).unwrap();

        let cfg = build_cfg(&instructions, script.entry_pc());
        let ctx = DecompileContext::new_simple(cfg, &pt, &[], &[], None);
        let lua = emit_lua(&ctx, script.entry_pc());

        assert!(
            lua.contains("local function proc_"),
            "expected local function proc_*, got:\n{lua}"
        );
        assert!(lua.contains("proc_"), "expected proc_ call, got:\n{lua}");
    }

    // ── test_extcall_name ─────────────────────────────────────────────────────

    #[test]
    fn test_extcall_name() {
        use pal_script::opcodes::ext_opcode;

        let ext = ext_opcode(0, 0);
        let (cat, idx, expected_name) = if let Some(e) = ext {
            if let Some(name) = e.name {
                (e.category, e.index, name.to_owned())
            } else {
                (0u16, 0u16, format!("ext_{:04X}_{:04X}", 0, 0))
            }
        } else {
            (0u16, 0u16, format!("ext_{:04X}_{:04X}", 0u16, 0u16))
        };

        let ext_call_arg = Argument::ExtCall {
            raw: ((cat as u32) << 16) | idx as u32,
            category: cat,
            index: idx,
            name: pal_script::opcodes::ext_opcode(cat, idx).and_then(|e| e.name),
        };
        let dst_arg = Argument::DestinationSlot { slot: 0, raw: 0 };

        let instr = make_primary(SCRIPT_CODE_BASE, 23, vec![ext_call_arg, dst_arg]);
        let output = emit_single_instr_lua(&instr, &[], &[], None, &[]);

        assert!(
            !output.contains("pal.extcall"),
            "should NOT contain pal.extcall, got: {output}"
        );
        assert!(
            output.contains(&expected_name),
            "expected function name '{expected_name}' in output, got: {output}"
        );
    }

    // ── test_extcall_args_reversed ────────────────────────────────────────────

    #[test]
    fn test_extcall_args_reversed() {
        // push A(10), push B(20), push C(30), extcall (cat=99,idx=0, unknown sig)
        // After reversal: args[0]=C=30, args[1]=B=20, args[2]=A=10 in pop order
        // Output should list them as 30, 20, 10 (pop order, since no sig for cat=99)
        let pending = vec![imm_op(10), imm_op(20), imm_op(30)]; // push order: A, B, C

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 99, 0, 0);
        let output = emit_single_instr_lua_full(&instr, &[], &[], None, &pending, &[], &[]);

        assert!(
            !output.contains("argument order unresolved"),
            "unknown extcall should not claim unresolved argument order, got: {output}"
        );
        // Pop order: last pushed (30) first, then 20, then 10
        let pos_30 = output.find("30").expect("should contain 30");
        let pos_20 = output.find("20").expect("should contain 20");
        let pos_10 = output.find("10").expect("should contain 10");
        assert!(
            pos_30 < pos_20 && pos_20 < pos_10,
            "expected pop order 30,20,10, got: {output}"
        );
    }

    // ── test_sp_set_ex_arg_order ──────────────────────────────────────────────

    #[test]
    fn test_sp_set_ex_arg_order() {
        // sp_set_ex: pop=5
        // Push order (push sequence): name(9), slot(40), x(0), y(0), z(0)
        // Pop order (reversed):       z(0), y(0), x(0), slot(40), name(9)
        //   → pop[0]=z=0, pop[1]=y=0, pop[2]=x=0, pop[3]=slot=40, pop[4]=name=9
        // Wait — re-read the spec: push A, push B, push C, push D, push E → pop[0]=E(last)
        // So push order name=9, slot=40, x=0, y=0, z=0:
        //   → args_pop_order[0]=z=0, [1]=y=0, [2]=x=0, [3]=slot=40, [4]=name=9
        // sig params: slot=pop[1]=y=0? No…
        //
        // Re-read spec for sp_set_ex:
        //   params: (pop[1]=slot:Slot, pop[0]=name:ResourceStringFromFileDat,
        //            pop[2]=x:Coordinate, pop[3]=y:Coordinate, pop[4]=z:Integer)
        //
        // So if we push: push name(9), push slot(40), push x(0), push y(0), push z(0)
        // Then pop_order: [0]=last=z=0, [1]=y=0, [2]=x=0, [3]=slot=40, [4]=name=9
        //   → pop[0]=name? No, pop[0] should be name per spec.
        //
        // The spec says pop[0]=name and pop[1]=slot.
        // If pop[0]=name → name was pushed LAST.
        // Push order: slot(40), x(0), y(0), z(0), name(9) → pop[0]=name(9), pop[1]=z=0?
        //
        // From the spec instruction description:
        //   "(pop[1]=slot:Slot, pop[0]=name:ResourceStringFromFileDat, ...)"
        // Display: sp_set_ex(slot, name, x, y, z) = sp_set_ex(pop[1], pop[0], pop[2], pop[3], pop[4])
        //
        // For expected output sp_set_ex(40, "#FFFFFFFF", 0, 0, 0):
        //   slot=40=pop[1], name=9→"#FFFFFFFF"=pop[0], x=0=pop[2], y=0=pop[3], z=0=pop[4]
        //
        // So pop_order: [0]=9(name), [1]=40(slot), [2]=0(x), [3]=0(y), [4]=0(z)
        // Pop_order[i] = push_order[N-1-i] where N=5
        // push_order[4]=9 → pop[0]=9 ✓ (name is pushed last)
        // push_order[3]=40 → pop[1]=40 ✓ (slot is 2nd-to-last push)
        // push_order[2]=0 → pop[2]=0 ✓ (x)
        // push_order[1]=0 → pop[3]=0 ✓ (y)
        // push_order[0]=0 → pop[4]=0 ✓ (z)
        //
        // So push order = [z=0, y=0, x=0, slot=40, name=9] for pop[0]=name=9

        // Build a file_dat with slot 9 = "#FFFFFFFF"
        // Actually 0x0FFF_FFFF is the empty string sentinel → "\"\"" is expected for name=9?
        // But the test says Expected: sp_set_ex(40, "#FFFFFFFF", 0, 0, 0)
        // That means file_dat slot 9 = "#FFFFFFFF"
        let file_dat = make_file_dat(&[(9, "#FFFFFFFF")]);
        let nls = Nls::ShiftJis;

        // push order: [z=0, y=0, x=0, slot=40, name=9]
        let pending = vec![
            imm_op(0),  // z (pushed first → pop[4])
            imm_op(0),  // y (pushed 2nd → pop[3])
            imm_op(0),  // x (pushed 3rd → pop[2])
            imm_op(40), // slot (pushed 4th → pop[1])
            imm_op(9),  // name (pushed last → pop[0])
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 3, 3, 0);
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &file_dat, &[]);

        // Expected: sp_set_ex(40, "#FFFFFFFF", 0, 0, 0)
        assert!(
            output.contains("sp_set_ex"),
            "expected sp_set_ex, got: {output}"
        );
        assert!(
            output.contains("40"),
            "expected slot=40 in output, got: {output}"
        );
        assert!(
            output.contains("#FFFFFFFF") || output.contains("\\\"#FFFFFFFF\\\""),
            "expected name resolved from file_dat slot 9, got: {output}"
        );
        // Slot should appear before name in output (display order)
        let pos_slot = output.find("40").unwrap();
        let pos_name = output
            .find('#')
            .unwrap_or_else(|| output.find('"').unwrap());
        assert!(
            pos_slot < pos_name,
            "slot(40) should appear before name in output, got: {output}"
        );
    }

    // ── test_bgm_play_resource_id ─────────────────────────────────────────────

    #[test]
    fn test_bgm_play_resource_id() {
        // bgm_play: pop=7
        // params: slot=pop[0], unknown=pop[1], name=pop[2], flags=pop[3], volume=pop[4], ...
        // For name=69 → File.dat slot 69 = "BGM01"
        // Push order: [unk3=0, unk2=0, vol=100, flags=0, name=69, unknown=0, slot=-1]
        // (last pushed = slot = pop[0])
        // Pop: [0]=slot=-1, [1]=unknown=0, [2]=name=69, [3]=flags=0, [4]=vol=100, [5]=unk2=0, [6]=unk3=0

        let file_dat = make_file_dat(&[(69, "BGM01")]);
        let nls = Nls::ShiftJis;

        let pending = vec![
            imm_op(0),   // unk3 (push first → pop[6])
            imm_op(0),   // unk2 (→ pop[5])
            imm_op(100), // volume (→ pop[4])
            imm_op(0),   // flags (→ pop[3])
            imm_op(69),  // name (→ pop[2])
            imm_op(0),   // unknown (→ pop[1])
            imm_op(-1),  // slot=-1 (push last → pop[0])
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 4, 0, 0);
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &file_dat, &[]);

        assert!(
            output.contains("bgm_play"),
            "expected bgm_play, got: {output}"
        );
        assert!(
            output.contains("BGM01"),
            "expected BGM01 resolved from file_dat slot 69, got: {output}"
        );
    }

    // ── test_voice_play_resource_id ───────────────────────────────────────────

    #[test]
    fn test_voice_play_resource_id() {
        // voice_play: pop=4
        // params: slot=pop[0], name=pop[1], flags=pop[2], volume=pop[3]
        // name=88 → File.dat slot 88 = "VO01_SYS01"
        // Push order: [vol=100, flags=0, name=88, slot=0]
        // Pop: [0]=slot=0, [1]=name=88, [2]=flags=0, [3]=vol=100

        let file_dat = make_file_dat(&[(88, "VO01_SYS01")]);
        let nls = Nls::ShiftJis;

        let pending = vec![
            imm_op(100), // volume (push first → pop[3])
            imm_op(0),   // flags (→ pop[2])
            imm_op(88),  // name (→ pop[1])
            imm_op(0),   // slot (push last → pop[0])
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 13, 0, 0);
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &file_dat, &[]);

        assert!(
            output.contains("voice_play"),
            "expected voice_play, got: {output}"
        );
        assert!(
            output.contains("VO01_SYS01"),
            "expected VO01_SYS01 resolved from file_dat slot 88, got: {output}"
        );
    }

    // ── test_file_dat_immediate_resource_resolution ───────────────────────────

    #[test]
    fn test_file_dat_immediate_resource_resolution() {
        // ParamKind::ResourceStringFromFileDat with Immediate(102) → resolves to "BK000D"
        let file_dat = make_file_dat(&[(102, "BK000D")]);
        let resolved = resolve_resource_str(&file_dat, 102, Nls::ShiftJis);
        assert_eq!(resolved, "\"BK000D\"", "got: {resolved}");
    }

    // ── test_integer_not_resolved_as_resource ─────────────────────────────────

    #[test]
    fn test_integer_not_resolved_as_resource() {
        // ParamKind::Integer with Immediate(102) → stays as 102, not "BK000D"
        // We test this via the codegen render_param path indirectly:
        // Use a known sig that has Integer kind (e.g., sp_set_pos z param = pop[3]=Integer)
        // sp_set_pos: slot=pop[0], x=pop[1], y=pop[2], z=pop[3]
        // Push order: [z=102, y=0, x=0, slot=0]
        // Pop: [0]=slot=0, [1]=x=0, [2]=y=0, [3]=z=102
        let file_dat = make_file_dat(&[(102, "BK000D")]);
        let nls = Nls::ShiftJis;

        let pending = vec![
            imm_op(102), // z (push first → pop[3])
            imm_op(0),   // y
            imm_op(0),   // x
            imm_op(0),   // slot (push last → pop[0])
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 3, 4, 0); // sp_set_pos
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &file_dat, &[]);

        // z=102 should appear as integer 102, not as "BK000D"
        assert!(
            output.contains("102"),
            "expected integer 102 in output, got: {output}"
        );
        assert!(
            !output.contains("BK000D"),
            "Integer param should NOT resolve as resource name, got: {output}"
        );
    }

    // ── test_getprivateprofileint_arg_order ───────────────────────────────────

    #[test]
    fn test_getprivateprofileint_arg_order() {
        // getprivateprofileint: pop=4
        // params: section=pop[0], key=pop[1], default=pop[2], filename=pop[3]
        // Using TextStringFromTextDat for section, key, filename
        // Build a minimal TEXT.DAT:
        //   offset 0 (after header): entries start at byte 16
        //   entry 0: key=0, string="SectionName\0"  → string at byte 20
        //   entry 1: key=1, string="KeyName\0"      → string at byte 32
        //   entry 2: key=2, string="config.ini\0"   → string at byte 44

        let text_dat = make_text_dat(&[(0, "SectionName"), (1, "KeyName"), (2, "config.ini")]);

        // The text IDs passed as push args should be byte offsets into text_dat.
        // After the 16-byte header, entries are:
        //   offset 16: key=0 (4 bytes) + "SectionName\0" → string at offset 20
        //   offset 32: key=1 (4 bytes) + "KeyName\0" → string at offset 36
        //   offset 44: key=2 (4 bytes) + "config.ini\0" → string at offset 48
        // The resolve_text_str function tries offset+4 first.
        // So passing value=16 → reads text_dat[20..] = "SectionName"
        //    passing value=32 → reads text_dat[36..] = "KeyName"
        //    passing value=44 → reads text_dat[48..] = "config.ini"

        // Find the actual byte offsets
        // Header = 16 bytes, then entries
        // entry 0 starts at 16: [u32 key=0][SectionName\0] → string at 20
        // "SectionName" = 11 bytes + NUL = 12 bytes → entry 0 is 16 bytes total
        // entry 1 starts at 32: [u32 key=1][KeyName\0] → string at 36
        // "KeyName" = 7 bytes + NUL = 8 bytes → entry 1 is 12 bytes total
        // entry 2 starts at 44: [u32 key=2][config.ini\0] → string at 48

        let nls = Nls::ShiftJis;

        // Verify our offsets work
        let s0 = resolve_text_str(&text_dat, 16, nls);
        let s1 = resolve_text_str(&text_dat, 32, nls);
        let s2 = resolve_text_str(&text_dat, 44, nls);
        assert!(s0.contains("SectionName"), "section: got {s0}");
        assert!(s1.contains("KeyName"), "key: got {s1}");
        assert!(s2.contains("config.ini"), "filename: got {s2}");

        // Now build the extcall:
        // getprivateprofileint returns a value, dst_slot=5
        // params: section=pop[0], key=pop[1], default=pop[2]=1280, filename=pop[3]
        // push order: [filename_id=44, default=1280, key_id=32, section_id=16]
        // pop: [0]=section=16, [1]=key=32, [2]=default=1280, [3]=filename=44
        let pending = vec![
            imm_op(44),   // filename (push first → pop[3])
            imm_op(1280), // default (→ pop[2])
            imm_op(32),   // key (→ pop[1])
            imm_op(16),   // section (push last → pop[0])
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 18, 37, 5);
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &[], &text_dat);

        assert!(
            output.contains("getprivateprofileint"),
            "expected getprivateprofileint, got: {output}"
        );
        assert!(
            output.contains("SectionName"),
            "expected SectionName resolved, got: {output}"
        );
        assert!(
            output.contains("KeyName"),
            "expected KeyName resolved, got: {output}"
        );
        assert!(
            output.contains("1280") || output.contains("0x500"),
            "expected default=1280 in output, got: {output}"
        );
        assert!(
            output.contains("config.ini"),
            "expected config.ini resolved, got: {output}"
        );
    }

    // ── test_unknown_extcall_raw_args ─────────────────────────────────────────

    #[test]
    fn test_unknown_extcall_raw_args() {
        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 99, 0, 0);
        let pending = vec![imm_op(1), imm_op(2)];
        let output = emit_single_instr_lua_full(&instr, &[], &[], None, &pending, &[], &[]);
        assert!(
            output.contains("ext_0063_0000(2, 1)"),
            "expected raw extcall with raw pop-order args, got: {output}"
        );
        assert!(
            !output.contains("argument order unresolved"),
            "unknown extcall should not claim unresolved argument order, got: {output}"
        );
    }

    // ── test_dynamic_jump_not_staticized ──────────────────────────────────────

    #[test]
    fn test_dynamic_jump_not_staticized() {
        // opcode 9 with Point { id=0, target_pc=None } — dynamic jump (id=0 = unresolved)
        let instr = make_primary(
            SCRIPT_CODE_BASE,
            9,
            vec![Argument::Point {
                id: 0,
                target_pc: None,
            }],
        );
        let output = emit_single_instr_lua(&instr, &[], &[], None, &[]);
        assert!(
            output.contains("dynamic_jump"),
            "expected dynamic_jump for dynamic jump, got: {output}"
        );
        // Should not contain a static label reference
        assert!(
            !output.contains("L_"),
            "should not contain static label, got: {output}"
        );
    }

    // ── test_while_condition_inside_loop ─────────────────────────────────────

    #[test]
    fn test_while_condition_inside_loop() {
        // While loop where header block has: [eq_instr(v0, imm(5)), jf_instr(exit, v0)]
        // The eq_instr is a non-terminal; the header has computation before the jf.
        // Expected output: "while true do" and the eq_instr INSIDE the while body.
        //
        // Layout:
        //   PC 12:  eq v0, imm(5)        (3 words: 12 bytes)
        //   PC 24:  jf p1(exit=44), v0   (3 words: 12 bytes)
        //   PC 36:  jmp p2(back=PC 12)   (2 words: 8 bytes)
        //   PC 44:  end                  (1 word: 4 bytes)

        let mut bytes = vec![0u8; 12 + 40];
        bytes[0..4].copy_from_slice(b"Sv20");
        bytes[8..12].copy_from_slice(&12u32.to_le_bytes());

        fn w(opcode: u32) -> [u8; 4] {
            ((1u32 << 16) | opcode).to_le_bytes()
        }
        fn var(slot: u16) -> [u8; 4] {
            (0x4000_0000u32 | slot as u32).to_le_bytes()
        }
        fn imm(n: i32) -> [u8; 4] {
            (n as u32).to_le_bytes()
        }
        fn pt(id: u32) -> [u8; 4] {
            id.to_le_bytes()
        }

        let mut pos = 12usize;
        // PC 12: eq v0, imm(5) — opcode 12 = eq
        bytes[pos..pos + 4].copy_from_slice(&w(12));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&var(0));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&imm(5));
        pos += 4;

        // PC 24: jf p1 (exit=44), v0
        bytes[pos..pos + 4].copy_from_slice(&w(10));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&pt(1)); // p1 → exit
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&var(0)); // condition v0
        pos += 4;

        // PC 36: jmp p2 (back to PC 12)
        bytes[pos..pos + 4].copy_from_slice(&w(9));
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&pt(2)); // p2 → PC 12
        pos += 4;

        // PC 44: end
        bytes[pos..pos + 4].copy_from_slice(&w(21));

        // PointTable:
        // p1 → PC 44 (exit): offset = 44-12 = 32
        // p2 → PC 12 (loop): offset = 12-12 = 0
        // entries=2, offsets[2-1]=offsets[1]=32, offsets[2-2]=offsets[0]=0
        let mut pt_bytes = vec![0u8; 8];
        pt_bytes[0..4].copy_from_slice(&0u32.to_le_bytes()); // p2 → PC 12
        pt_bytes[4..8].copy_from_slice(&32u32.to_le_bytes()); // p1 → PC 44

        let pt_table = pal_script::PointTable::parse(&pt_bytes).unwrap();
        let script = ScriptImage::parse(&bytes).unwrap();
        let options = pal_script::DisassembleOptions {
            start: 12,
            end: None,
            point_table: Some(&pt_table),
        };
        let instructions = pal_script::disassemble_script(&script, options).unwrap();

        let cfg = build_cfg(&instructions, script.entry_pc());
        let ctx = DecompileContext::new_simple(cfg, &pt_table, &[], &[], None);
        let lua = emit_lua(&ctx, script.entry_pc());

        // The header block (PC 12) has non-terminal eq instruction → complex while
        assert!(
            lua.contains("while true do"),
            "expected 'while true do' for complex while, got:\n{lua}"
        );
        // The break condition should be inside the while
        assert!(
            lua.contains("break"),
            "expected break inside while, got:\n{lua}"
        );
        // The eq instruction should be inside the while (before the break check)
        let while_pos = lua.find("while true do").unwrap();
        let eq_pos = lua
            .find("v0 =")
            .unwrap_or_else(|| lua.find("v0=").unwrap_or(usize::MAX));
        // eq should appear after "while true do"
        assert!(
            eq_pos > while_pos,
            "eq instruction should be inside the while loop, got:\n{lua}"
        );
    }

    // ── test_function_boundary_no_next_gosub_cutoff ───────────────────────────

    #[test]
    fn test_function_boundary_no_next_gosub_cutoff() {
        // Two-function script:
        //   PC 12: gosub p1 → PC 32 (subroutine)
        //   PC 20: ... (some instructions in main)
        //   PC 24: end  ← main ends here (ret at PC 24, well before the next gosub target)
        //   PC 28: (padding)
        //   PC 32: ret  ← subroutine
        //
        // With the old code (next_func_pc cutoff): main would be cut off at PC 32.
        // With the new code (exit_pc=None): main emits until it hits end at PC 24.
        // The subroutine at PC 32 should be emitted separately and contain "return".

        let mut bytes = vec![0u8; 12 + 24];
        bytes[0..4].copy_from_slice(b"Sv20");
        bytes[8..12].copy_from_slice(&12u32.to_le_bytes());

        fn w(opcode: u32) -> [u8; 4] {
            ((1u32 << 16) | opcode).to_le_bytes()
        }
        fn pt(id: u32) -> [u8; 4] {
            id.to_le_bytes()
        }

        let mut pos = 12usize;

        // PC 12: gosub p1 → PC 32 (2 words)
        bytes[pos..pos + 4].copy_from_slice(&w(11)); // gosub
        pos += 4;
        bytes[pos..pos + 4].copy_from_slice(&pt(1)); // p1
        pos += 4;

        // PC 20: nop (1 word)
        bytes[pos..pos + 4].copy_from_slice(&w(22)); // nop
        pos += 4;

        // PC 24: end (1 word)
        bytes[pos..pos + 4].copy_from_slice(&w(21)); // end
        pos += 4;

        // PC 28: nop (1 word) — dead code between functions
        bytes[pos..pos + 4].copy_from_slice(&w(22)); // nop
        pos += 4;

        // PC 32: ret (1 word)
        bytes[pos..pos + 4].copy_from_slice(&w(24)); // ret

        // PointTable: p1 → PC 32 (offset = 32-12 = 20)
        let mut pt_bytes = vec![0u8; 4];
        pt_bytes[0..4].copy_from_slice(&20u32.to_le_bytes());

        let pt_table = pal_script::PointTable::parse(&pt_bytes).unwrap();
        let script = ScriptImage::parse(&bytes).unwrap();
        let options = pal_script::DisassembleOptions {
            start: 12,
            end: None,
            point_table: Some(&pt_table),
        };
        let instructions = pal_script::disassemble_script(&script, options).unwrap();

        let cfg = build_cfg(&instructions, script.entry_pc());
        let ctx = DecompileContext::new_simple(cfg, &pt_table, &[], &[], None);
        let lua = emit_lua(&ctx, script.entry_pc());

        // Both functions should be emitted
        assert!(
            lua.contains("local function"),
            "expected local function definitions, got:\n{lua}"
        );
        // Main function should emit a return (from the end at PC 24)
        // Subroutine at PC 32 should also have a return
        assert!(
            lua.contains("return"),
            "expected return in output, got:\n{lua}"
        );
        // There should be two separate function definitions
        let func_count = lua.matches("local function").count();
        assert!(
            func_count >= 2,
            "expected at least 2 local functions, got {func_count} in:\n{lua}"
        );
    }

    // ── test_no_pal_extcall_output ────────────────────────────────────────────

    #[test]
    fn test_no_pal_extcall_output() {
        // Verify the output never contains "pal.extcall("
        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 7, 0, 0); // wait
        let pending = vec![imm_op(500), imm_op(1)];
        let output = emit_single_instr_lua_full(&instr, &[], &[], None, &pending, &[], &[]);
        assert!(
            !output.contains("pal.extcall"),
            "output must not contain 'pal.extcall', got: {output}"
        );
    }

    // ── test_literal_slot_0xffffffff_slot_renders_minus1 ─────────────────────

    #[test]
    fn test_literal_slot_0xffffffff_slot_renders_minus1() {
        // bgm_play pop[0]=LiteralSlot(0xFFFFFFFF) = slot=-1 (sentinel for "use default slot 0")
        // ParamKind::Slot → should render as -1 (integer), NOT as dynamic_string(0xFFFFFFFF)
        // Push order (7 args): unk3, unk2, vol, flags, name, unknown, slot=-1
        // pop[0]=slot=-1 (LiteralSlot(0xFFFFFFFF))
        let file_dat = make_file_dat(&[(69, "BGM01")]);
        let nls = Nls::ShiftJis;

        let pending = vec![
            imm_op(0),                    // unk3 → pop[6]
            imm_op(0),                    // unk2 → pop[5]
            imm_op(100),                  // vol  → pop[4]
            imm_op(0),                    // flags → pop[3]
            imm_op(69),                   // name=69 → pop[2]
            imm_op(0),                    // unknown → pop[1]
            literal_slot_op(0xFFFF_FFFF), // slot=-1 → pop[0] (last pushed)
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 4, 0, 0); // bgm_play
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &file_dat, &[]);

        assert!(
            output.contains("bgm_play"),
            "expected bgm_play, got: {output}"
        );
        // Slot=-1 should appear as integer -1, not as dynamic_string(0xFFFFFFFF)
        assert!(
            output.contains("-1"),
            "slot=-1 should render as -1, got: {output}"
        );
        assert!(
            !output.contains("dynamic_string(0xFFFFFFFF)"),
            "slot param should NOT produce dynamic_string(0xFFFFFFFF), got: {output}"
        );
    }

    // ── test_0xffffffff_resource_not_dynamic_string ──────────────────────────

    #[test]
    fn test_0xffffffff_resource_not_dynamic_string() {
        // LiteralSlot(0xFFFFFFFF) as ResourceStringFromFileDat param
        // → should render as nil --[[no_resource]], not dynamic_string(0xFFFFFFFF)
        use crate::strings::resolve_resource_str;
        let result = resolve_resource_str(&[], 0xFFFF_FFFF, Nls::ShiftJis);
        assert!(
            result.contains("nil"),
            "0xFFFFFFFF should resolve to nil sentinel, got: {result}"
        );
        assert!(
            !result.contains("dynamic_string"),
            "0xFFFFFFFF should NOT be dynamic_string, got: {result}"
        );
    }

    // ── test_invalid_static_gosub_not_silent ─────────────────────────────────

    #[test]
    fn test_invalid_static_gosub_not_silent() {
        // opcode 11 (gosub) with Point { id=999, target_pc=None } — id > 0 but no target
        // Should emit an INVALID comment, not silently skip.
        //
        // Build: PC 12: gosub_point p999 (invalid), PC 20: end
        let mut bytes = vec![0u8; 12 + 12];
        bytes[0..4].copy_from_slice(b"Sv20");
        bytes[8..12].copy_from_slice(&12u32.to_le_bytes()); // entry_pc=12

        fn w(op: u32) -> [u8; 4] {
            ((1u32 << 16) | op).to_le_bytes()
        }
        // PC 12: gosub_point 999
        bytes[12..16].copy_from_slice(&w(11));
        bytes[16..20].copy_from_slice(&999u32.to_le_bytes());
        // PC 20: end
        bytes[20..24].copy_from_slice(&w(21));

        // PointTable with 1 entry (id=1 → PC 12), so id=999 is out of range
        let mut pt_bytes = vec![0u8; 4];
        pt_bytes[0..4].copy_from_slice(&0u32.to_le_bytes()); // id=1 → offset=0 → PC 12
        let pt_table = pal_script::PointTable::parse(&pt_bytes).unwrap();

        let script = pal_script::ScriptImage::parse(&bytes).unwrap();
        // Disassemble WITHOUT point_table to avoid PointIdOutOfRange for id=999.
        // resolve_static_points then silently skips out-of-range IDs.
        let options = pal_script::DisassembleOptions {
            start: 12,
            end: None,
            point_table: None,
        };
        let mut instructions = pal_script::disassemble_script(&script, options).unwrap();

        // resolve_static_points: id=999 > entries=1 → Err → target_pc stays None
        crate::decompile::resolve_static_points(&mut instructions, &pt_table);

        let cfg = build_cfg(&instructions, script.entry_pc());
        let ctx = DecompileContext::new_simple(cfg, &pt_table, &[], &[], None);
        let lua = emit_lua(&ctx, script.entry_pc());

        // Must emit an error annotation, not silently skip the gosub
        assert!(
            lua.contains("INVALID") || lua.contains("invalid") || lua.contains("gosub_invalid"),
            "invalid gosub target must produce error annotation, got:\n{lua}"
        );
    }

    // ── test_getprivateprofileint_non_ini_filename_annotation ─────────────────

    #[test]
    fn test_getprivateprofileint_non_ini_filename_annotation() {
        // getprivateprofileint with filename="HIP_BACK_G" (not *.ini)
        // Should annotate with "runtime uses SYSTEM.INI"
        //
        // Build TEXT.DAT:
        //   offset 16: key=0, "graphics\0"      → string at offset 20
        //   offset 29: key=1, "DEF_CG_WIDTH\0"  → string at offset 33
        //   offset 50: key=2, "HIP_BACK_G\0"    → string at offset 54
        //   (offset 50 = 29 + 4 + 12 + 5 = ?)
        // Let's compute:
        //   entry0: 16 + [4 bytes key] + "graphics\0"(9) = 16+4+9=29, entry1 starts at 29
        //   entry1: 29 + [4] + "DEF_CG_WIDTH\0"(13) = 29+4+13=46, entry2 starts at 46
        //   entry2: 46 + [4] + "HIP_BACK_G\0"(11) = 46+4+11=61

        let text_dat = make_text_dat(&[(0, "graphics"), (1, "DEF_CG_WIDTH"), (2, "HIP_BACK_G")]);

        // Compute byte offsets (record starts, not string starts)
        // Header=16 bytes
        // entry0 at 16: key(4) + "graphics\0"(9) = 13 bytes → entry1 at 29
        // entry1 at 29: key(4) + "DEF_CG_WIDTH\0"(13) = 17 bytes → entry2 at 46
        // entry2 at 46: HIP_BACK_G
        let nls = Nls::ShiftJis;

        // Verify text_dat strings at expected offsets
        let s0 = resolve_text_str(&text_dat, 16, nls);
        let s1 = resolve_text_str(&text_dat, 29, nls);
        let s2 = resolve_text_str(&text_dat, 46, nls);
        assert!(s0.contains("graphics"), "s0={s0}");
        assert!(s1.contains("DEF_CG_WIDTH"), "s1={s1}");
        assert!(s2.contains("HIP_BACK_G"), "s2={s2}");

        // Push order: [filename=46, default=1280, key=29, section=16]
        let pending = vec![
            imm_op(46),   // filename(HIP_BACK_G) → pop[3]
            imm_op(1280), // default → pop[2]
            imm_op(29),   // key → pop[1]
            imm_op(16),   // section → pop[0]
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 18, 37, 1);
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &[], &text_dat);

        assert!(
            output.contains("HIP_BACK_G"),
            "filename should be in output, got: {output}"
        );
        // Non-.ini filename must be annotated
        assert!(
            output.contains("SYSTEM.INI") || output.contains("runtime"),
            "non-.ini filename must have SYSTEM.INI annotation, got: {output}"
        );
    }

    // ── test_sp_set_ex_no_double_reverse ─────────────────────────────────────

    #[test]
    fn test_sp_set_ex_no_double_reverse() {
        // Verify there is NO double-reverse: push order [z, y, x, slot, name]
        // should give pop order [name, slot, x, y, z]
        // sig: slot=pop[1], name=pop[0] → output: sp_set_ex(slot, name, ...)
        // i.e., sp_set_ex(slot_val, name_val, x_val, y_val, z_val)
        //
        // Concrete: push(z=5), push(y=4), push(x=3), push(slot=2), push(name_id=1)
        //   pop[0]=name_id=1 → file_dat slot 1 = "SLOT1_NAME"
        //   pop[1]=slot=2
        //   output: sp_set_ex(2, "SLOT1_NAME", 3, 4, 5)

        let file_dat = make_file_dat(&[(0, "SLOT0_NAME"), (1, "SLOT1_NAME")]);
        let nls = Nls::ShiftJis;

        // Push order: z=5, y=4, x=3, slot=2, name=1 (name pushed last → pop[0]=1)
        let pending = vec![
            imm_op(5), // z → pop[4]
            imm_op(4), // y → pop[3]
            imm_op(3), // x → pop[2]
            imm_op(2), // slot → pop[1]
            imm_op(1), // name_id → pop[0] (last pushed)
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 3, 3, 0); // sp_set_ex
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &file_dat, &[]);

        // Expected: sp_set_ex(2, "SLOT1_NAME", 3, 4, 5)
        assert!(output.contains("sp_set_ex"), "got: {output}");
        assert!(
            output.contains("SLOT1_NAME"),
            "name should be resolved, got: {output}"
        );
        assert!(
            output.contains('2'),
            "slot=2 should be in output, got: {output}"
        );

        // Verify slot(2) appears before name("SLOT1_NAME") — no double-reverse
        let pos_slot = output.find('2').expect("should have 2");
        let pos_name = output.find("SLOT1_NAME").expect("should have SLOT1_NAME");
        assert!(
            pos_slot < pos_name,
            "slot(2) must appear before name in display order (no double-reverse), got: {output}"
        );

        // And name appears before x(3), y(4), z(5)
        let pos_x = output.rfind('3').expect("should have 3");
        assert!(pos_name < pos_x, "name before x in output, got: {output}");
    }

    // ── test_getprivateprofileint_ini_extension_no_annotation ────────────────

    #[test]
    fn test_getprivateprofileint_ini_extension_no_annotation() {
        // When filename ends in ".ini", no annotation should be added
        let text_dat = make_text_dat(&[(0, "graphics"), (1, "DEF_CG_WIDTH"), (2, "SYSTEM.INI")]);

        // Compute offsets
        // entry0 at 16: "graphics\0"(9) + key(4) = 13 → entry1 at 29
        // entry1 at 29: "DEF_CG_WIDTH\0"(13) + key(4) = 17 → entry2 at 46
        let nls = Nls::ShiftJis;

        let pending = vec![
            imm_op(46),   // filename=SYSTEM.INI → pop[3]
            imm_op(1280), // default → pop[2]
            imm_op(29),   // key → pop[1]
            imm_op(16),   // section → pop[0]
        ];

        let instr = make_extcall_instr(SCRIPT_CODE_BASE, 18, 37, 1);
        let output =
            emit_single_instr_lua_full(&instr, &[], &[], Some(nls), &pending, &[], &text_dat);

        assert!(
            output.contains("SYSTEM.INI"),
            "should contain SYSTEM.INI, got: {output}"
        );
        // *.ini filename should NOT add the "runtime uses SYSTEM.INI" annotation
        assert!(
            !output.contains("runtime uses SYSTEM.INI"),
            "*.ini filename should not be annotated, got: {output}"
        );
    }
}
