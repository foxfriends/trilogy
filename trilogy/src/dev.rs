use clap::Subcommand;
use pretty::{DocAllocator, RcAllocator};
use std::path::PathBuf;
use trilogy_parser::{Parser, PrettyPrintSExpr};
use trilogy_scanner::Scanner;

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Scan a file, printing out its tokens.
    Scan { file: PathBuf },
    /// Parse a file, printing out its AST.
    Parse {
        file: PathBuf,
        #[arg(short, long, default_value = "80")]
        width: usize,
        #[arg(short, long)]
        verbose: bool,
    },
}

pub fn run(command: Command) -> std::io::Result<()> {
    match command {
        Command::Scan { file } => {
            let contents = std::fs::read_to_string(file)?;
            let scanner = Scanner::new(&contents);
            for token in scanner {
                println!("{token:?}");
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
    }

    Ok(())
}
