#![allow(clippy::never_loop)]

#[trilogy_derive::module(crate_name=crate)]
pub mod std {
    use crate::{Result, Runtime, Struct, Value};

    /// Prints a value to stdout. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn print(_: &mut Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => println!("{s}"),
            _ => println!("{value}"),
        }
        Ok(())
    }

    /// Prints a value to stderr. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn eprint(_: &mut Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => eprintln!("{s}"),
            _ => eprintln!("{value}"),
        }
        Ok(())
    }

    /// Reads a full line from stdin, as a string.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn readline(runtime: &mut Runtime) -> Result<()> {
        use std::io;
        let mut s = String::new();
        match io::stdin().read_line(&mut s) {
            Ok(0) => runtime.r#yield(runtime.atom("EOF"), |rt, val| rt.r#return(val)),
            Err(error) => {
                let atom = runtime.atom("ERR");
                let error = Struct::new(atom, error.to_string());
                runtime.r#yield(error, |rt, val| rt.r#return(val))
            }
            _ => runtime.r#return(Value::String(s)),
        }
    }
}
