//! A cactus stack.
//!
//! This is the stack implementation that backs the Trilogy VM, where branches
//! are used to represent continuations and closures that share a parent stack
//! but have differing active stacks.
//!
//! It is not a general purpose cactus implementation, but one specialized to
//! managing the stack in a garbage collected environment. Elements of the cactus
//! will not be freed immediately.

mod branch;
#[allow(clippy::module_inception)]
mod cactus;
mod pointer;
mod range_map;
mod slice;

pub use branch::Branch;
pub use cactus::Cactus;
pub use pointer::Pointer;
pub use range_map::RangeMap;
pub use slice::Slice;
