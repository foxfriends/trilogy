use std::{fs, path::PathBuf, process::Command};

fn try_command(command: &mut Command) {
    let output = command.spawn().unwrap().wait_with_output().unwrap();
    if !output.status.success() {
        println!("cargo::error={}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}

fn main() {
    let core = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap()
        .parse::<PathBuf>()
        .unwrap()
        .join("core");

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
        Command::new("clang")
            .args(["-S", "-emit-llvm"])
            .args(&sources)
            .current_dir(&core),
    );

    for file in &sources {
        let ll = file.with_extension("ll");
        let bc = file.with_extension("bc");
        try_command(
            Command::new("llvm-as")
                .arg(ll)
                .arg("-o")
                .arg(bc)
                .current_dir(&core),
        );
    }

    try_command(
        Command::new("llvm-link")
            .args(
                sources
                    .into_iter()
                    .map(|source| source.with_extension("bc")),
            )
            .args(["-o", "core.bc"])
            .current_dir(&core),
    );
}
