//! The parser for the Trilogy Programming Language.

#[macro_use]
extern crate trilogy_parser_derive;

#[cfg(test)]
#[macro_use]
mod test;

mod format;
mod parse;
mod parser;
mod spanned;
pub mod syntax;
mod token_pattern;

// These things probably belong in some internal prelude...
#[doc(hidden)]
pub use format::{PrettyPrintSExpr, PrettyPrinted, PrettyPrinter};

pub use parse::Parse;
pub use parser::Parser;
pub use spanned::Spanned;

pub(crate) use token_pattern::TokenPattern;
