#[trilogy_derive::module(crate_name=crate)]
pub mod str {
    use trilogy_vm::Array;

    use crate::{Result, Runtime, Value};

    /// Converts a value to its string representation. This is the same representation
    /// that is used when printing the value with `print`.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn cast(rt: Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => rt.r#return(s),
            Value::Char(ch) => rt.r#return(ch.to_string()),
            _ => rt.r#return(value.to_string()),
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn slice(rt: Runtime, start: Value, len: Value, string: Value) -> Result<()> {
        let start = rt.typecheck::<usize>(start)?;
        let len = rt.typecheck::<usize>(len)?;
        let string = rt.typecheck::<String>(string)?;
        rt.r#return(string.chars().skip(start).take(len).collect::<String>())
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn length(rt: Runtime, string: Value) -> Result<()> {
        let string = rt.typecheck::<String>(string)?;
        rt.r#return(string.len())
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn reverse(rt: Runtime, string: Value) -> Result<()> {
        let string = rt.typecheck::<String>(string)?;
        rt.r#return(string.chars().rev().collect::<String>())
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn replace(rt: Runtime, needle: Value, replacement: Value, string: Value) -> Result<()> {
        let replacement = rt.typecheck::<String>(replacement)?;
        let string = rt.typecheck::<String>(string)?;

        match needle {
            Value::String(needle) => rt.r#return(string.replace(&needle, &replacement)),
            Value::Char(needle) => rt.r#return(string.replace(needle, &replacement)),
            _ => Err(rt.runtime_type_error(needle)),
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn replace_n(
        rt: Runtime,
        n: Value,
        needle: Value,
        replacement: Value,
        string: Value,
    ) -> Result<()> {
        let n = rt.typecheck::<usize>(n)?;
        let replacement = rt.typecheck::<String>(replacement)?;
        let string = rt.typecheck::<String>(string)?;

        match needle {
            Value::String(needle) => rt.r#return(string.replacen(&needle, &replacement, n)),
            Value::Char(needle) => rt.r#return(string.replacen(needle, &replacement, n)),
            _ => Err(rt.runtime_type_error(needle)),
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn split(rt: Runtime, separator: Value, string: Value) -> Result<()> {
        let string = rt.typecheck::<String>(string)?;

        match separator {
            Value::String(separator) => rt.r#return(
                string
                    .split(&separator)
                    .map(|val| val.into())
                    .collect::<Array>(),
            ),
            Value::Char(separator) => rt.r#return(
                string
                    .split(separator)
                    .map(|val| val.into())
                    .collect::<Array>(),
            ),
            _ => Err(rt.runtime_type_error(separator)),
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn split_n(rt: Runtime, n: Value, separator: Value, string: Value) -> Result<()> {
        let n = rt.typecheck::<usize>(n)?;
        let string = rt.typecheck::<String>(string)?;

        match separator {
            Value::String(separator) => rt.r#return(
                string
                    .splitn(n + 1, &separator)
                    .map(|val| val.into())
                    .collect::<Array>(),
            ),
            Value::Char(separator) => rt.r#return(
                string
                    .splitn(n + 1, separator)
                    .map(|val| val.into())
                    .collect::<Array>(),
            ),
            _ => Err(rt.runtime_type_error(separator)),
        }
    }
}
