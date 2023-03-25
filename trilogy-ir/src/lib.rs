#![allow(dead_code)]

mod analyzer;
mod error;
pub mod ir;
mod scope;
mod symbol;

pub use analyzer::Analyzer;
pub use error::Error;
pub use symbol::Id;
