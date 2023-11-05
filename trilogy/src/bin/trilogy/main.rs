use clap::Parser as _;
use num::{bigint::Sign, BigInt};
use std::io::stdin;
use std::path::PathBuf;
use trilogy::{RuntimeError, Trilogy};
use trilogy_vm::Value;

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
    Run {
        /// The path to the Trilogy source file containing the `main!()` procedure.
        file: PathBuf,
        /// Run without including the standard library.
        #[arg(short = 'S', long)]
        no_std: bool,
        /// Print the exit value instead of using it as the exit code.
        #[arg(short, long)]
        print: bool,
    },
    /// Runs a pre-compiled Trilogy program by interfacing with the VM directly.
    Vm {
        /// A path to a pre-compiled Trilogy ASM file.
        ///
        /// Omit to read from STDIN.
        file: Option<PathBuf>,
        #[arg(short = 'S', long)]
        /// Run without the standard library.
        no_std: bool,
        /// Print the exit value instead of using it as the exit code.
        #[arg(short, long)]
        print: bool,
    },
    /// Compile a Trilogy program, printing the ASM it compiles to.
    /// Redirect to a file is recommended.
    ///
    /// Expects a single path in which the `main!()` procedure is found.
    Compile { file: PathBuf },
    /// Check the syntax and warnings of a Trilogy program.
    Check {
        /// The path to the Trilogy source file containing the `main!()` procedure.
        file: PathBuf,
        /// Check without including the standard library.
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
    /// Commands for assistance when developing Trilogy.
    #[cfg(feature = "dev")]
    #[command(subcommand)]
    Dev(dev::Command),
}

fn handle(result: Result<Value, RuntimeError>, print: bool) {
    match result {
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

fn run(trilogy: Trilogy, print: bool) {
    handle(trilogy.run(), print)
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
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
        Command::Vm {
            file: Some(path),
            print,
            no_std: _,
        } => match Trilogy::from_asm(&mut std::fs::File::open(path)?) {
            Ok(trilogy) => run(trilogy, print),
            Err(errors) => {
                eprintln!("{errors}");
                std::process::exit(1);
            }
        },
        Command::Vm {
            file: None,
            print,
            no_std: _,
        } => match Trilogy::from_asm(&mut stdin()) {
            Ok(trilogy) => run(trilogy, print),
            Err(errors) => {
                eprintln!("{errors}");
                std::process::exit(1);
            }
        },
        Command::Compile { file } => match Trilogy::from_file(file) {
            Ok(trilogy) => match trilogy.compile() {
                Ok(chunk) => println!("{}", chunk),
                Err(error) => eprintln!("{error}"),
            },
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
        Command::Check { file, no_std: _ } => {
            if let Err(report) = Trilogy::from_file(file) {
                report.eprint();
                std::process::exit(1);
            }
        }
        Command::Test { file } => match Trilogy::from_file(file) {
            Ok(trilogy) => match trilogy.run_tests() {
                Ok(true) => {}
                Ok(false) => std::process::exit(1),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            },
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
        #[cfg(feature = "dev")]
        Command::Dev(dev_command) => {
            dev::run(dev_command)?;
        }
        _ => todo!("not yet implemented"),
    }

    Ok(())
}
