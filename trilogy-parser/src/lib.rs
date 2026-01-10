//! The parser for the Trilogy Programming Language.

#![cfg_attr(test, expect(incomplete_features))]
#![cfg_attr(test, feature(deref_patterns))]

#[macro_use]
extern crate trilogy_parser_derive;

#[cfg(test)]
#[macro_use]
mod test;

mod parse;
mod parser;
mod spanned;
pub mod syntax;
mod token_pattern;

pub use parse::Parse;
pub use parser::Parser;
pub use spanned::Spanned;

pub(crate) use token_pattern::TokenPattern;
