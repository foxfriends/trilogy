use trilogy_parser::syntax;

#[derive(Debug)]
pub enum Error {
    UnknownExport { name: syntax::Identifier },
}
