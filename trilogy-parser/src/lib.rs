#[macro_use]
extern crate trilogy_parser_derive;

mod parser;
pub mod syntax;

pub use parser::Parser;

// These things probably belong in some internal prelude...
mod spanned;
mod token_pattern;
pub(crate) use spanned::Spanned;
pub(crate) use token_pattern::TokenPattern;
