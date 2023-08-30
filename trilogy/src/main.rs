use clap::Parser as _;
use num::{bigint::Sign, BigInt};
use std::path::PathBuf;
use trilogy_loader::Loader;
use trilogy_vm::{Program, Value, VirtualMachine};

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
    Check { file: PathBuf },
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

fn print_errors(errors: impl IntoIterator<Item = impl std::fmt::Debug>) {
    for error in errors {
        eprintln!("{error:#?}");
    }
}

fn run(program: Program, print: bool, _no_std: bool) {
    match VirtualMachine::load(program.clone()).run() {
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
            eprintln!("Trace:\n{}", error.trace(&program));
            eprintln!("Dump:\n{}", error.dump());
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
            no_std,
        } => {
            let loader = Loader::new(file);
            let binder = loader.load().unwrap();
            if binder.has_errors() {
                print_errors(binder.errors());
                std::process::exit(1);
            }
            let program = match binder.analyze() {
                Ok(program) => program,
                Err(errors) => {
                    print_errors(errors);
                    std::process::exit(1);
                }
            };
            let program = program.generate_code();
            run(program, print, no_std)
        }
        Command::Compile { file } => {
            let loader = Loader::new(file);
            let binder = loader.load().unwrap();
            if binder.has_errors() {
                print_errors(binder.errors());
                std::process::exit(1);
            }
            let program = match binder.analyze() {
                Ok(program) => program,
                Err(errors) => {
                    print_errors(errors);
                    std::process::exit(1);
                }
            };
            let program = program.generate_code();
            print!("{}", program);
        }
        Command::Check { file } => {
            let loader = Loader::new(file);
            let binder = loader.load().unwrap();
            if binder.has_errors() {
                print_errors(binder.errors());
                std::process::exit(1);
            }
            if let Err(errors) = binder.analyze() {
                print_errors(errors);
                std::process::exit(1);
            };
        }
        #[cfg(feature = "dev")]
        Command::Dev(dev_command) => {
            dev::run(dev_command)?;
        }
        Command::Vm {
            file,
            print,
            no_std,
        } => {
            let asm = std::fs::read_to_string(file)?;
            let program: Program = match asm.parse() {
                Ok(program) => program,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            };
            run(program, print, no_std)
        }
        _ => unimplemented!("This feature is not yet built"),
    }

    Ok(())
}
