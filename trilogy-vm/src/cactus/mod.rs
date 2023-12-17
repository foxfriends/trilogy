//! A cactus stack.
//!
//! This is the stack implementation that backs the Trilogy VM, where branches
//! are used to represent continuations and closures that share a parent stack
//! but have differing active stacks.

mod branch;
mod cactus;
mod slice;

pub use branch::Branch;
pub use cactus::Cactus;
pub use slice::Slice;
