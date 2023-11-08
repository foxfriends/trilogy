#[trilogy_derive::module(crate_name=crate)]
pub mod str {
    use crate::{Result, Runtime, Value};

    /// Converts a value to its string representation. This is the same representation
    /// that is used when printing the value with `print`.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn cast(rt: Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => rt.r#return(s),
            _ => rt.r#return(value.to_string()),
        }
    }
}