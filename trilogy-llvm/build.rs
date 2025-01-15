use std::{fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=core/trilogy.h");
    println!("cargo::rerun-if-changed=core/trilogy.c");
    let core = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap()
        .parse::<PathBuf>()
        .unwrap()
        .join("core");
    fs::remove_file(core.join("trilogy.ll")).ok();
    fs::remove_file(core.join("trilogy.bc")).ok();
    let output = Command::new("clang")
        .args(["-S", "-emit-llvm", "trilogy.c"])
        .current_dir(&core)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    if !output.status.success() {
        println!("cargo::error={}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    let output = Command::new("llvm-as")
        .args(["trilogy.ll", "-o", "trilogy.bc"])
        .current_dir(&core)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    if !output.status.success() {
        println!("cargo::error={}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}
