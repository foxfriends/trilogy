//! Bridges the runtime of the Trilogy Virtual Machine to the host program.

mod array;
pub(crate) mod atom;
mod bits;
mod continuation;
mod eq;
mod native;
mod number;
mod procedure;
mod record;
mod set;
mod r#struct;
mod tuple;
mod value;

pub use array::Array;
pub use atom::Atom;
pub use bits::Bits;
pub use continuation::Continuation;
pub use eq::{ReferentialEq, StructuralEq};
pub use native::{Native, NativeFunction};
pub use number::Number;
pub use procedure::Procedure;
pub use r#struct::Struct;
pub use record::Record;
pub use set::Set;
pub use tuple::Tuple;
pub use value::Value;
