use anyhow::Result;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const CARGO_CONTENT: &str = r#"
[package]
name = "binder_ndk"
authors = ["Android"]
version = "1.0.0"
edition = "2021"
rust-version = "1.67"

[lib]
crate-type = ["cdylib"]
"#;

fn build_stub() -> Result<()> {
    let symbols = std::fs::read_to_string("src/symbols.txt")?;
    let outdir = env::var("OUT_DIR")?;
    let project_path = PathBuf::from(&outdir).join("libbinder_ndk");
    if project_path.exists() {
        std::fs::remove_dir_all(&project_path)?;
    }
    std::fs::create_dir(&project_path)?;

    let project_cargo_path = project_path.join("Cargo.toml");
    std::fs::File::create(&project_cargo_path)?;
    std::fs::write(&project_cargo_path, CARGO_CONTENT)?;
    let src_path = project_path.join("src");
    std::fs::create_dir_all(&src_path)?;
    let mut f = std::fs::File::create(src_path.join("lib.rs"))?;
    for symbol in symbols.split('\n') {
        if !symbol.is_empty() {
            f.write_all(format!("#[no_mangle]\npub extern fn {}() {{}}\n", symbol).as_bytes())?;
        }
    }
    f.flush()?;

    let target = env::var("TARGET")?;
    Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg(&target)
        .arg("--manifest-path")
        .arg(project_cargo_path)
        .arg("--target-dir")
        .arg(&outdir)
        .current_dir(&project_path)
        .status()?;

    // we always use debug build for stub due to speed!
    println!(
        "cargo:rustc-link-search={}",
        format_args!("{}/{}/{}", outdir, target, "debug")
    );
    println!("cargo:rustc-link-lib=binder_ndk");

    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=src/symbols.txt");

    build_stub().unwrap();
}
