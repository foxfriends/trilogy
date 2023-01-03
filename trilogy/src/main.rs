use clap::{Parser, Subcommand};
use std::path::PathBuf;
use trilogy_scanner::Scanner;

/// Trilogy Programming Language
#[derive(Parser, Clone, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    /// Start up the interactive Trilogy REPL.
    Repl,
    /// Run a Trilogy program.
    ///
    /// Expects a single path in which the `main!()` procedure is found.
    Run { file: PathBuf },
    /// Check the syntax and warnings of a Trilogy program.
    ///
    /// Expects a single path, from which all imported modules will be
    /// checked.
    Check { file: PathBuf },
    /// Format one or many Trilogy files.
    ///
    /// If no files are provided, formats from standard input to standard
    /// output.
    Fmt { files: Vec<PathBuf> },
    /// Run the Trilogy language server.
    Lsp { files: Vec<PathBuf> },
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Run { file } => {
            let contents = std::fs::read_to_string(file)?;
            let scanner = Scanner::new(&contents);
            for token in scanner {
                println!("{:?}", token);
            }
        }
        _ => unimplemented!("This feature is not yet built"),
    }

    Ok(())
}
