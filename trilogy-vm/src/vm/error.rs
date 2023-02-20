// I am aware these names are not all that ergonomic, but they line up
// with what they are documented as.
//
// Maybe `Error` is not the best name for this enum, will revisit later.
#[allow(clippy::enum_variant_names)]
pub enum Error {
    RuntimeTypeError,
    AssertionError,
    ExecutionFizzledError,
    UnhandledEffectError,
    InternalRuntimeError,
}
