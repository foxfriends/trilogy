mod analyzer;
mod error;
pub mod ir;
mod scope;
mod symbol;
pub mod visitor;

pub use analyzer::{Analyzer, Resolver};
pub use error::Error;
pub use symbol::Id;
