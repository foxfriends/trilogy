#![allow(clippy::never_loop)]

#[trilogy_derive::module(crate_name=crate)]
pub mod std {
    use crate::{Runtime, Struct, Value};

    /// Prints a value to stdout. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn print(_: Runtime, value: Value) {
        match value {
            Value::String(s) => println!("{s}"),
            _ => println!("{value}"),
        }
    }

    /// Prints a value to stderr. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn eprint(_: Runtime, value: Value) {
        match value {
            Value::String(s) => eprintln!("{s}"),
            _ => eprintln!("{value}"),
        }
    }

    /// Reads a full line from stdin, as a string.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn readline(runtime: Runtime) -> Value {
        use std::io;
        let mut s = String::new();
        match io::stdin().read_line(&mut s) {
            Ok(0) => {
                for val in runtime.y(runtime.atom("EOF")) {
                    return val;
                }
                Value::Unit
            }
            Err(error) => {
                let atom = runtime.atom("ERR");
                let error = Struct::new(atom, error.to_string());
                for val in runtime.y(error) {
                    return val;
                }
                Value::Unit
            }
            _ => Value::String(s),
        }
    }
}
