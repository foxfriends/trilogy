use clap::Parser as _;
use std::path::PathBuf;
use trilogy_parser::Parser;
use trilogy_scanner::Scanner;

#[cfg(feature = "dev")]
mod dev;

/// Trilogy Programming Language
#[derive(clap::Parser, Clone, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Clone, Debug)]
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
    /// Commands for assistance when developing Trilogy.
    #[cfg(feature = "dev")]
    #[command(subcommand)]
    Dev(dev::Command),
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Run { file } => {
            let contents = std::fs::read_to_string(file)?;
            let scanner = Scanner::new(&contents);
            let parser = Parser::new(scanner);
            let ast = parser.parse();
            println!("{:#?}", ast.ast);
        }
        #[cfg(feature = "dev")]
        Command::Dev(dev_command) => {
            dev::run(dev_command)?;
        }
        _ => unimplemented!("This feature is not yet built"),
    }

    Ok(())
}
