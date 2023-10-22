//! Bridges the runtime of the Trilogy Virtual Machine to the host program.

mod array;
pub(crate) mod atom;
mod bits;
pub(crate) mod callable;
mod eq;
mod number;
mod record;
mod set;
mod r#struct;
mod tuple;
mod value;

pub use array::Array;
pub use atom::Atom;
pub use bits::Bits;
pub use callable::{Callable, Native, NativeFunction};
pub use eq::{ReferentialEq, StructuralEq};
pub use number::Number;
pub use r#struct::Struct;
pub use record::Record;
pub use set::Set;
pub use tuple::Tuple;
pub use value::Value;
