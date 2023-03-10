#![allow(dead_code)] // this is all just planning anyway

mod analysis;
mod analyzer;
pub mod ir;
mod lexical_error;

pub use analysis::Analysis;
pub use analyzer::Analyzer;
pub use lexical_error::LexicalError;
