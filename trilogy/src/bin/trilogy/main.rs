use clap::Parser as _;
#[cfg(feature = "tvm")]
use num::{bigint::Sign, BigInt};
#[cfg(feature = "tvm")]
use std::io::stdin;
use std::path::PathBuf;
use trilogy::{Builder, Trilogy};

#[cfg(feature = "tvm")]
use trilogy_vm::Value;

#[cfg(feature = "dev")]
mod dev;

#[cfg(all(feature = "std", feature = "tvm"))]
mod test_reporter;

/// Trilogy Programming Language
#[derive(clap::Parser, Clone, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Clone, Debug)]
enum Command {
    /// Run a Trilogy program.
    Run {
        /// The path to the Trilogy source file containing the `main!()` procedure.
        file: PathBuf,
        /// Run without including the standard library.
        #[cfg(feature = "std")]
        #[arg(short = 'S', long)]
        no_std: bool,
        /// Print the exit value instead of using it as the exit code.
        #[arg(short, long)]
        print: bool,
        /// Print the debug trace instead of the regular stack trace on error.
        #[arg(long)]
        debug: bool,
    },
    /// Runs a pre-compiled Trilogy program by interfacing with the VM directly.
    #[cfg(feature = "tvm")]
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
        /// Print the debug trace instead of the regular stack trace on error.
        #[arg(long)]
        debug: bool,
    },
    /// Compile a Trilogy program, printing the ASM it compiles to.
    /// Redirect to a file is recommended.
    ///
    /// Expects a single path in which the `main!()` procedure is found.
    Compile {
        file: PathBuf,
        #[arg(long = "lib")]
        library: bool,
    },
    /// Check the syntax and warnings of a Trilogy program.
    Check {
        /// The path to the Trilogy source file containing the `main!()` procedure.
        file: PathBuf,
        /// Check without including the standard library.
        #[cfg(feature = "std")]
        #[arg(short = 'S', long)]
        no_std: bool,
    },
    #[cfg(all(feature = "tvm", feature = "std"))]
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

fn run(trilogy: Trilogy, print: bool, debug: bool) {
    let result = trilogy.run();
    #[cfg(feature = "stats")]
    {
        log::info!("stats\n{}", trilogy.stats());
        log::info!("stats\n{}", trilogy_vm::GLOBAL_STATS);
    }

    match result {
        #[cfg(feature = "tvm")]
        Ok(value) if print => {
            println!("{}", value);
        }
        #[cfg(feature = "llvm")]
        Ok(value) if print => {
            println!("{:?}", value);
        }
        #[cfg(feature = "llvm")]
        Ok(value) => {
            // NOTE: for now, we're printing no matter what...
            println!("{:?}", value);
        }
        #[cfg(feature = "tvm")]
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
        #[cfg(feature = "tvm")]
        Ok(..) => std::process::exit(255),
        Err(error) if debug => {
            eprintln!("{error:?}");
            std::process::exit(255);
        }
        #[cfg(feature = "tvm")]
        Err(error) => {
            error.eprint();
            std::process::exit(255);
        }
        #[cfg(feature = "llvm")]
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(255);
        }
    }
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    tokio::task::spawn_blocking(main_sync).await?
}

#[cfg(not(feature = "async"))]
fn main() -> std::io::Result<()> {
    main_sync()
}

fn main_sync() -> std::io::Result<()> {
    pretty_env_logger::init();
    let args = Cli::parse();

    match args.command {
        Command::Run {
            file,
            print,
            debug,
            #[cfg(feature = "std")]
                no_std: true,
        } => match Builder::new().build_from_source(file) {
            Ok(trilogy) => run(trilogy, print, debug),
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
        #[cfg(feature = "std")]
        Command::Run {
            file, print, debug, ..
        } => match Trilogy::from_file(file) {
            Ok(trilogy) => run(trilogy, print, debug),
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
        #[cfg(feature = "tvm")]
        Command::Vm {
            file: Some(path),
            print,
            no_std: _,
            debug,
        } => match Builder::default().build_from_asm(&mut std::fs::File::open(path)?) {
            Ok(trilogy) => run(trilogy, print, debug),
            Err(errors) => {
                eprintln!("{errors}");
                std::process::exit(1);
            }
        },
        #[cfg(feature = "tvm")]
        Command::Vm {
            file: None,
            print,
            no_std: _,
            debug,
        } => match Builder::default().build_from_asm(&mut stdin()) {
            Ok(trilogy) => run(trilogy, print, debug),
            Err(errors) => {
                eprintln!("{errors}");
                std::process::exit(1);
            }
        },
        #[cfg(all(feature = "tvm", feature = "std"))]
        Command::Compile { file, library, .. } => {
            match Builder::std().is_library(library).build_from_source(file) {
                Ok(trilogy) => match trilogy.compile() {
                    Ok(chunk) => println!("{}", chunk),
                    Err(error) => eprintln!("{error}"),
                },
                Err(report) => {
                    report.eprint();
                    std::process::exit(1);
                }
            }
        }
        #[cfg(all(feature = "llvm", feature = "std"))]
        Command::Compile { file, library } => {
            match Builder::std().is_library(library).build_from_source(file) {
                Ok(trilogy) => {
                    print!("{}", trilogy.compile());
                }
                Err(report) => {
                    report.eprint();
                    std::process::exit(1);
                }
            }
        }
        #[cfg(feature = "tvm")]
        Command::Check {
            file,
            #[cfg(feature = "std")]
                no_std: true,
        } => {
            if let Err(report) = Builder::new().build_from_source(file) {
                report.eprint();
                std::process::exit(1);
            }
        }
        #[cfg(feature = "std")]
        Command::Check { file, .. } => {
            if let Err(report) = Trilogy::from_file(file) {
                report.eprint();
                std::process::exit(1);
            }
        }
        #[cfg(all(feature = "std", feature = "tvm"))]
        Command::Test { file } => match Builder::std().is_library(true).build_from_source(file) {
            Ok(trilogy) => {
                let mut reporter = test_reporter::Stdout::default();
                trilogy.run_tests(&mut reporter);
                if reporter.is_error() {
                    std::process::exit(1);
                }
            }
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
