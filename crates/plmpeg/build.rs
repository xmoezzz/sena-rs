fn main() {
    println!("cargo:rerun-if-changed=vendor/pl_mpeg.h");
    println!("cargo:rerun-if-changed=src/pl_mpeg_impl.c");

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        return;
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        println!("cargo:rustc-link-lib=m");
    }

    cc::Build::new()
        .file("src/pl_mpeg_impl.c")
        .include("vendor")
        .opt_level(2)
        .warnings(false)
        .compile("plmpeg");
}
