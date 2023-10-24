use crate::{ChunkBuilder, Value};

/// A program that can be run on this VM.
///
/// The `Program` trait encapsulates the module resolution portion of a
/// particular language's runtime, allowing the relationship between
/// modules to be defined by the language.
///
/// # Examples
///
/// ```
/// # use trilogy_vm::{Program, VirtualMachine, Value, ChunkBuilder};
/// struct NoopProgram;
///
/// impl Program for NoopProgram {
///     fn chunk(&self, input: &Value, builder: &mut ChunkBuilder) {}
///
///     fn entrypoint(&self, builder: &mut ChunkBuilder) {
///         builder.parse(r#"
///             CONST 0
///             EXIT
///         "#);
///     }
/// }
///
/// # let vm = VirtualMachine::new();
/// assert_eq!(vm.run(&NoopProgram).unwrap(), Value::from(0));
/// ```
pub trait Program {
    /// Retrieve another chunk of code as described by a given value. The interpretation
    /// of the value (and production of the value) is at the language runtime's definition.
    fn chunk(&self, input: &Value, builder: &mut ChunkBuilder);

    /// Compute the initial chunk to execute when the virtual machine is provided with
    /// a new program.
    fn entrypoint(&self, builder: &mut ChunkBuilder);
}
