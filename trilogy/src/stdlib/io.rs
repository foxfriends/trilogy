#[trilogy_derive::module(crate_name=crate)]
pub mod io {
    use crate::{Result, Runtime, Struct, Value};
    use std::io::Write;

    /// Prints a value to stdout. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn println(rt: Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => println!("{s}"),
            _ => println!("{value}"),
        }
        rt.r#return(())
    }

    /// Prints a value to stdout. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn print(rt: Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => print!("{s}"),
            _ => print!("{value}"),
        }
        std::io::stdout().flush().unwrap();
        rt.r#return(())
    }

    /// Prints a value to stderr. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn eprintln(rt: Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => eprintln!("{s}"),
            _ => eprintln!("{value}"),
        }
        rt.r#return(())
    }

    /// Prints a value to stderr. The value will be printed using its string representation.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn eprint(rt: Runtime, value: Value) -> Result<()> {
        match value {
            Value::String(s) => eprint!("{s}"),
            _ => eprint!("{value}"),
        }
        rt.r#return(())
    }

    /// Reads a full line from stdin, as a string.
    #[trilogy_derive::proc(crate_name=crate)]
    pub fn readline(rt: Runtime) -> Result<()> {
        use std::io;
        let mut s = String::new();
        match io::stdin().read_line(&mut s) {
            Ok(0) => {
                let atom = rt.atom("EOF");
                rt.r#yield(atom, |rt, val| rt.r#return(val))
            }
            Err(error) => {
                let atom = rt.atom("ERR");
                let error = Struct::new(atom, error.to_string());
                rt.r#yield(error, |rt, val| rt.r#return(val))
            }
            _ => rt.r#return(Value::String(s)),
        }
    }
}
