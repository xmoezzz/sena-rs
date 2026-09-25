use std::path::PathBuf;

use pal_asset::{AssetSource, LoadedAsset};
use pal_script::PointTable;
use pal_vm::{CoreAssets, RuntimeStatus, ScriptRuntime, ScriptRuntimeConfig};

fn asset(name: &str, bytes: Vec<u8>) -> LoadedAsset {
    LoadedAsset {
        name: name.to_owned(),
        bytes,
        source: AssetSource::Loose {
            path: PathBuf::from(name),
        },
    }
}

fn opcode(op: u16) -> [u8; 4] {
    (0x0001_0000u32 | op as u32).to_le_bytes()
}

fn word(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

fn var(slot: u16) -> u32 {
    0x4000_0000u32 | slot as u32
}

#[test]
fn runtime_executes_basic_variable_arithmetic() {
    let mut script = Vec::new();
    script.extend_from_slice(b"Sv20");
    script.extend_from_slice(&word(0));
    script.extend_from_slice(&word(12));
    script.extend_from_slice(&opcode(1));
    script.extend_from_slice(&word(var(1)));
    script.extend_from_slice(&word(1));
    script.extend_from_slice(&opcode(2));
    script.extend_from_slice(&word(var(1)));
    script.extend_from_slice(&word(2));
    script.extend_from_slice(&opcode(21));

    let assets = CoreAssets {
        script: asset("Script.src", script),
        file_dat: asset("File.dat", Vec::new()),
        text_dat: asset("Text.dat", Vec::new()),
        mem_dat: asset("Mem.dat", Vec::new()),
        point_dat: asset("Point.dat", Vec::new()),
        graphic_dat: None,
        extended_softpal: false,
        script_check_value: 0,
        script_entry_pc: 12,
        point_table: PointTable::parse(&[]).expect("empty Point.dat should parse"),
        graphic_index: None,
    };

    let config = ScriptRuntimeConfig::default();
    let mut runtime = ScriptRuntime::boot(assets.script_entry_pc, config.clone());
    let tick = runtime
        .run_frame(&assets, &config)
        .expect("runtime should execute fixture");

    assert_eq!(tick.status, RuntimeStatus::Halted { pc: 40 });
    assert_eq!(runtime.vars()[1], 3);
}
