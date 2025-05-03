use std::{fs, path::PathBuf, process::Command};

fn try_command(command: &mut Command) {
    let output = command.spawn().unwrap().wait_with_output().unwrap();
    if !output.status.success() {
        println!("cargo::error={}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    if !output.stderr.is_empty() {
        println!("cargo::warning={}", String::from_utf8_lossy(&output.stderr));
    }
}

fn main() {
    let llvm_prefix = std::env::var("LLVM_SYS_181_PREFIX")
        .ok()
        .and_then(|s| s.parse::<PathBuf>().ok())
        .map(|p| p.join("bin"))
        .unwrap_or_default();
    let core = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap()
        .parse::<PathBuf>()
        .unwrap()
        .join("core");
    println!("cargo::rerun-if-env-changed=TRILOGY_CORE_DEFINES");
    let defines = std::env::var("TRILOGY_CORE_DEFINES")
        .map(|s| {
            s.split(",")
                .map(|def| format!("-D{def}"))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut sources = vec![];
    for file in fs::read_dir(&core).unwrap() {
        let Ok(file) = file else {
            continue;
        };
        match file.path().extension().and_then(|s| s.to_str()) {
            Some("h") => println!("cargo::rerun-if-changed={}", file.path().display()),
            Some("c") => {
                println!("cargo::rerun-if-changed={}", file.path().display());
                sources.push(file.path())
            }
            Some("ll" | "bc" | "tmp") => {
                fs::remove_file(file.path()).ok();
            }
            _ => {}
        }
    }

    try_command(
        Command::new(llvm_prefix.join("clang"))
            .args(["-g", "-S", "-emit-llvm", "-Wall"])
            .args(&defines)
            .args(&sources)
            .current_dir(&core),
    );

    for file in &sources {
        let ll = file.with_extension("ll");
        let bc = file.with_extension("bc");
        try_command(
            Command::new(llvm_prefix.join("llvm-as"))
                .arg(ll)
                .arg("-o")
                .arg(bc)
                .current_dir(&core),
        );
    }

    try_command(
        Command::new(llvm_prefix.join("llvm-link"))
            .args(
                sources
                    .into_iter()
                    .map(|source| source.with_extension("bc")),
            )
            .args(["-o", "core.bc"])
            .current_dir(&core),
    );
}
