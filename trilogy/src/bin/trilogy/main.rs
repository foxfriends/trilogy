use clap::Parser as _;
use num::{bigint::Sign, BigInt};
use std::path::PathBuf;
use trilogy::Trilogy;
use trilogy_vm::{Program, Value};

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
    Run {
        file: PathBuf,
        #[arg(short = 'S', long)]
        no_std: bool,
        #[arg(short, long)]
        print: bool,
    },
    /// Compile a Trilogy program into a bytecode file which can be run
    /// later on the VM directly.
    ///
    /// Expects a single path in which the `main!()` procedure is found.
    Compile { file: PathBuf },
    /// Check the syntax and warnings of a Trilogy program.
    ///
    /// Expects a single path, from which all imported modules will be
    /// checked.
    Check {
        file: PathBuf,
        #[arg(short = 'S', long)]
        no_std: bool,
    },
    /// Runs all tests found in the given module and all its submodules.
    ///
    /// The provided path is not required to define a `main` function as
    /// entrypoint, as it will not be called.
    Test { file: PathBuf },
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
    /// Run precompiled bytecode directly.
    Vm {
        file: PathBuf,
        #[arg(short = 'S', long)]
        no_std: bool,
        #[arg(short, long)]
        print: bool,
    },
    /// Commands for assistance when developing Trilogy.
    #[cfg(feature = "dev")]
    #[command(subcommand)]
    Dev(dev::Command),
}

fn run(mut trilogy: Trilogy, print: bool) {
    match trilogy.run() {
        Ok(value) if print => {
            println!("{}", value);
        }
        Ok(Value::Number(number)) if number.is_integer() => {
            let output = number.as_integer().unwrap();
            // Truly awful
            if BigInt::from(i32::MIN) <= output && output <= BigInt::from(i32::MAX) {
                let (sign, digits) = output.to_u32_digits();
                let exit = if sign == Sign::Minus {
                    -(digits[0] as i32)
                } else if sign == Sign::Plus {
                    digits[0] as i32
                } else {
                    0
                };
                std::process::exit(exit);
            }
            std::process::exit(255)
        }
        Ok(..) => std::process::exit(255),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(255);
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Run {
            file,
            print,
            no_std: _,
        } => match Trilogy::from_file(file) {
            Ok(trilogy) => run(trilogy, print),
            Err(errors) => {
                eprintln!("{errors}");
                std::process::exit(1);
            }
        },
        Command::Compile { file } => match Trilogy::from_file(file) {
            Ok(trilogy) => {
                println!("{}", trilogy);
            }
            Err(errors) => {
                eprintln!("{errors}");
                std::process::exit(1);
            }
        },
        Command::Check { file, no_std: _ } => match Trilogy::from_file(file) {
            Ok(..) => {}
            Err(errors) => {
                eprintln!("{errors}");
                std::process::exit(1);
            }
        },

        #[cfg(feature = "dev")]
        Command::Dev(dev_command) => {
            dev::run(dev_command)?;
        }
        Command::Vm {
            file,
            print,
            no_std: _,
        } => {
            let asm = std::fs::read_to_string(file)?;
            let program: Program = match asm.parse() {
                Ok(program) => program,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            };
            run(Trilogy::from(program), print)
        }
        _ => unimplemented!("This feature is not yet built"),
    }

    Ok(())
}
