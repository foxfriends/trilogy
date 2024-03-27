#[trilogy_derive::module(crate_name=crate)]
pub mod json {
    use crate::{Result, Runtime, Value};

    /// Converts a value to a valid JSON string that represents that value.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn stringify(rt: Runtime, value: Value) -> Result<()> {
        let string = serde_json::to_string(&value).map_err(|error| {
            rt.runtime_error(rt.r#struct(
                "JsonError",
                format!("value is not JSON serializable: {error}"),
            ))
        })?;
        rt.r#return(string)
    }

    /// Converts a value to a pretty-printed JSON string that represents that value.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn stringify_pretty(rt: Runtime, value: Value) -> Result<()> {
        let string = serde_json::to_string_pretty(&value).map_err(|error| {
            rt.runtime_error(rt.r#struct(
                "JsonError",
                format!("value is not JSON serializable: {error}"),
            ))
        })?;
        rt.r#return(string)
    }

    /// Parses a JSON string into a value.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn parse(rt: Runtime, value: Value) -> Result<()> {
        let string = rt.typecheck::<String>(value)?;
        let value: Value = serde_json::from_str(&string).map_err(|error| {
            rt.runtime_error(rt.r#struct("JsonError", format!("string is not valid JSON: {error}")))
        })?;
        rt.r#return(value)
    }
}
