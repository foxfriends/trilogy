#[macro_use]
extern crate trilogy_parser_derive;

#[cfg(test)]
#[macro_use]
mod test;

mod parse;
mod parser;
pub mod syntax;

pub use parse::Parse;
pub use parser::Parser;

// These things probably belong in some internal prelude...
#[doc(hidden)]
pub use format::{PrettyPrintSExpr, PrettyPrinted, PrettyPrinter};

mod format;
mod spanned;
mod token_pattern;
pub use spanned::Spanned;
pub(crate) use token_pattern::TokenPattern;
