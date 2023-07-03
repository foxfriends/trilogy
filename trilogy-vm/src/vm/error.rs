// I am aware these names are not all that ergonomic, but they line up
// with what they are documented as.
//
// Maybe `Error` is not the best name for this enum, will revisit later.
#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Debug)]
pub enum Error {
    RuntimeTypeError,
    AssertionError,
    ExecutionFizzledError,
    UnhandledEffectError,
    InternalRuntimeError(InternalRuntimeError),
}

#[derive(Copy, Clone, Debug)]
pub enum InternalRuntimeError {
    InvalidOpcode,
    InvalidOffset,
    ExpectedValue,
    ExpectedPointer,
    ExpectedStack,
}

impl From<InternalRuntimeError> for Error {
    fn from(value: InternalRuntimeError) -> Self {
        Self::InternalRuntimeError(value)
    }
}
