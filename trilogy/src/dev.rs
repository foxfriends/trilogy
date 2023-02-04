use clap::Subcommand;
use std::path::PathBuf;
use trilogy_parser::Parser;
use trilogy_scanner::Scanner;

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Scan a file, printing out its tokens.
    Scan { file: PathBuf },
    /// Parse a file, printing out its AST.
    Parse { file: PathBuf },
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
        Command::Parse { file } => {
            let contents = std::fs::read_to_string(file)?;
            let scanner = Scanner::new(&contents);
            let parser = Parser::new(scanner);
            let ast = parser.parse();
            println!("{:#?}", ast.ast);
        }
    }

    Ok(())
}
