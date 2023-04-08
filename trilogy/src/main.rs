use clap::Parser as _;
use pretty::RcAllocator;
use std::path::PathBuf;
use trilogy_loader::Loader;
use trilogy_parser::{Parser, PrettyPrint};
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
    /// If one file is provided, the output is written to standard output
    /// unless the `--write` flag is passed. With multiple files, the `--write`
    /// flag must be passed.
    Fmt {
        /// The file, or files, to format.
        ///
        /// If no files are provided, formats from standard input to standard output.
        files: Vec<PathBuf>,
        #[arg(short, long)]
        /// Write formatted output directly to the file at the path from where it was read.
        write: bool,
    },
    /// Run the Trilogy language server.
    Lsp { files: Vec<PathBuf> },
    /// Commands for assistance when developing Trilogy.
    #[cfg(feature = "dev")]
    #[command(subcommand)]
    Dev(dev::Command),
}

fn print_errors(errors: impl IntoIterator<Item = impl std::fmt::Debug>) {
    for error in errors {
        eprintln!("{error:#?}");
    }
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Run { file } => {
            let loader = Loader::new(file);
            let binder = loader.load().unwrap();
            if binder.has_errors() {
                print_errors(binder.errors());
                std::process::exit(1);
            }
            match binder.analyze() {
                Ok(analyzed) => {
                    println!("{:#?}", analyzed);
                }
                Err(errors) => print_errors(errors),
            }
        }
        Command::Fmt { files, write } => {
            if files.is_empty() {
                let mut stdin = std::io::stdin();
                let contents = std::io::read_to_string(&mut stdin)?;
                let scanner = Scanner::new(&contents);
                let parser = Parser::new(scanner);
                let parse = parser.parse();
                if !parse.has_errors() {
                    let doc = parse.ast().pretty_print(&RcAllocator);
                    doc.render(100, &mut std::io::stdout())?;
                } else {
                    print_errors(parse.errors());
                    std::process::exit(1);
                }
            } else if files.len() == 1 && !write {
                let contents = std::fs::read_to_string(&files[0])?;
                let scanner = Scanner::new(&contents);
                let parser = Parser::new(scanner);
                let parse = parser.parse();
                if !parse.has_errors() {
                    let doc = parse.ast().pretty_print(&RcAllocator);
                    doc.render(100, &mut std::io::stdout())?;
                } else {
                    print_errors(parse.errors());
                    std::process::exit(1);
                }
            } else {
                let mut all_success = true;
                for file in files {
                    let contents = std::fs::read_to_string(&file)?;
                    let scanner = Scanner::new(&contents);
                    let parser = Parser::new(scanner);
                    let parse = parser.parse();
                    if !parse.has_errors() {
                        let doc = parse.ast().pretty_print(&RcAllocator);
                        doc.render(100, &mut std::fs::File::create(&file)?)?;
                    } else {
                        print_errors(parse.errors());
                        all_success = false;
                    }
                }
                if !all_success {
                    std::process::exit(1);
                }
            }
        }
        #[cfg(feature = "dev")]
        Command::Dev(dev_command) => {
            dev::run(dev_command)?;
        }
        _ => unimplemented!("This feature is not yet built"),
    }

    Ok(())
}
