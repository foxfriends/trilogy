use crate::{ChunkBuilder, Value};

/// A program that can be run on this VM.
///
/// The `Program` trait encapsulates the module resolution portion of a
/// particular language's runtime, allowing the relationship between
/// modules to be defined by the language.
pub trait Program {
    /// Retrieve another chunk of code as described by a given value. The interpretation
    /// of the value (and production of the value) is at the language runtime's definition.
    fn chunk(&mut self, input: &Value, builder: &mut ChunkBuilder);

    /// Compute the initial chunk to execute when the virtual machine is provided with
    /// a new program.
    fn entrypoint(&mut self, builder: &mut ChunkBuilder);
}
