use clap::Parser as _;
use std::path::PathBuf;
use trilogy::{Builder, Trilogy};

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
    /// Run a Trilogy program.
    Run {
        /// The path to the Trilogy source file containing the `main!()` procedure.
        file: PathBuf,
        /// Run without including the standard library.
        #[arg(short = 'S', long)]
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
        /// Compile a library instead of a program.
        ///
        /// In this case, no `proc main!()` is required, but the resulting library cannot be used
        /// for much.
        #[arg(long = "lib")]
        library: bool,
        /// Compile the tests instead of the application.
        ///
        /// The resulting binary will run the test suite and print its result when executed.
        #[arg(long = "test")]
        test: bool,
        /// Only tests in modules whose path matches a listed prefix will be run.
        ///
        /// By default, tests in all locally defined modules are run.
        ///
        /// This flag is only relevant when compiling tests.
        #[arg(long = "prefix", short = 'p', default_values_t = [String::from("file:")])]
        filter_prefix: Vec<String>,
    },
    /// Check the syntax and warnings of a Trilogy program.
    Check {
        /// The path to the Trilogy source file containing the `main!()` procedure.
        file: PathBuf,
    },
    /// Runs all tests found in the given module and all its submodules.
    ///
    /// The provided path is not required to define a `main` function as
    /// entrypoint, as it will not be called.
    Test {
        file: PathBuf,
        /// Only tests in modules whose path matches a listed prefix will be run.
        ///
        /// By default, tests in all locally defined modules are run.
        #[arg(long = "prefix", short = 'p', default_values_t = [String::from("file:")])]
        filter_prefix: Vec<String>,
    },
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
    /// Print the version information.
    Version,
    /// Commands for assistance when developing Trilogy.
    #[cfg(feature = "dev")]
    #[command(subcommand)]
    Dev(dev::Command),
}

fn run(trilogy: Trilogy, print: bool, debug: bool) {
    let result = trilogy.run();
    match result {
        Ok(value) if print => {
            println!("{value:?}");
        }
        Ok(value) => {
            // NOTE: for now, we're printing no matter what...
            println!("{value:?}");
        }
        Err(error) if debug => {
            eprintln!("{error:?}");
            std::process::exit(255);
        }
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(255);
        }
    }
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    let args = Cli::parse();

    match args.command {
        Command::Run { file, print, debug } => match Builder::std().build_from_source(file) {
            Ok(trilogy) => run(trilogy, print, debug),
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
        Command::Compile {
            file,
            library,
            test,
            filter_prefix,
        } => match Builder::std()
            .is_library(library || test)
            .build_from_source(file)
        {
            Ok(trilogy) => {
                print!(
                    "{}",
                    if test {
                        trilogy.compile_test(&filter_prefix)
                    } else {
                        trilogy.compile()
                    }
                );
            }
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
        Command::Check { file, .. } => {
            if let Err(report) = Trilogy::from_file(file) {
                report.eprint();
                std::process::exit(1);
            }
        }
        Command::Test {
            file,
            filter_prefix,
        } => match Builder::std().is_library(true).build_from_source(file) {
            Ok(trilogy) => {
                trilogy.test(&filter_prefix);
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
        Command::Version => {
            println!(
                "{} {} -- Trilogy Language Compiler",
                env!("CARGO_CRATE_NAME"),
                env!("CARGO_PKG_VERSION")
            )
        }
        _ => todo!("not yet implemented"),
    }

    Ok(())
}
