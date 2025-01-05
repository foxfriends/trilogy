use clap::Subcommand;
use colored::*;
use pretty::{DocAllocator, RcAllocator};
use std::path::PathBuf;
#[cfg(feature = "tvm")]
use trilogy::Trilogy;
use trilogy_parser::{Parser, PrettyPrintSExpr};
use trilogy_scanner::{Scanner, TokenType, TokenValue};

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Scan a file, printing out its tokens.
    Scan {
        file: PathBuf,
        #[arg(short, long)]
        debug: bool,
    },
    /// Parse a file, printing out its AST.
    Parse {
        file: PathBuf,
        #[arg(short, long, default_value = "80")]
        width: usize,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Parse a file, printing out the IR.
    Ir { file: PathBuf },
    /// Parse a file, printing out the ASM.
    #[cfg(feature = "tvm")]
    Asm { file: PathBuf },
}

pub fn run(command: Command) -> std::io::Result<()> {
    match command {
        Command::Scan { file, debug } => {
            let contents = std::fs::read_to_string(file)?;
            let scanner = Scanner::new(&contents);
            for token in scanner {
                if debug {
                    println!("{token:?}");
                    continue;
                }
                print!(
                    "{}   {}",
                    token.span.to_string().cyan(),
                    format!("{:?}", token.token_type).yellow()
                );
                match token.token_type {
                    TokenType::Identifier => {
                        let Some(TokenValue::String(value)) = token.value else {
                            unreachable!()
                        };
                        print!(" ({})", value.blue());
                    }
                    TokenType::Numeric
                    | TokenType::Bits
                    | TokenType::Character
                    | TokenType::String => match token.value {
                        Some(TokenValue::Bits(bits)) => {
                            print!(" ({})", bits.to_string().bright_yellow())
                        }
                        Some(TokenValue::String(s)) => print!(" ({})", s.green()),
                        Some(TokenValue::Char(ch)) => print!(" ({})", ch.to_string().green()),
                        Some(TokenValue::Number(n)) => {
                            print!(" ({})", n.to_string().bright_yellow())
                        }
                        _ => unreachable!(),
                    },
                    _ => {}
                }
                println!();
            }
        }
        Command::Parse {
            file,
            width,
            verbose,
        } => {
            let contents = std::fs::read_to_string(file)?;
            let scanner = Scanner::new(&contents);
            let parser = Parser::new(scanner);
            let parse = parser.parse();

            if verbose {
                println!("{:#?}", parse.ast());
            } else {
                let allocator = RcAllocator;
                let doc = parse
                    .ast()
                    .pretty_print_sexpr(&allocator)
                    .append(allocator.hardline());
                doc.render(width, &mut std::io::stdout())?;
            }

            if parse.has_warnings() {
                println!("Encountered {} warnings:", parse.warnings().len());
                println!("{:#?}", parse.warnings());
            }
            if parse.has_errors() {
                println!("Encountered {} errors:", parse.errors().len());
                println!("{:#?}", parse.errors());
            }
        }
        Command::Ir { .. } => todo!(),
        #[cfg(feature = "tvm")]
        Command::Asm { file } => match Trilogy::from_file(file) {
            Ok(trilogy) => match trilogy.compile_debug() {
                Ok(chunk) => println!("{:?}", chunk),
                Err(error) => eprintln!("{error}"),
            },
            Err(report) => {
                report.eprint();
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
