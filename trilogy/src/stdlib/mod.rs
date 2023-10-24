#[trilogy_derive::module(crate_name=crate)]
pub mod std {
    #[trilogy_derive::module(crate_name=crate)]
    pub mod num {
        use crate::{Number, Result, Runtime, Value};

        /// Converts an arbitrary value to a number, if possible. The conversion
        /// depends on the type of input:
        /// * Number: Unchanged
        /// * Char: The value of the character's unicode code point
        /// * String: Attempt to parse the string as a number
        /// * Bool: true = 1, false = 0
        /// * Unit: always 0
        /// * Bits: Interpret the bits as a unsigned integer of arbitrary size
        ///
        /// When the conversion is not possible (for other types, or the string
        /// does not represent a valid number) yields `'NAN`.
        #[trilogy_derive::proc(crate_name=crate)]
        pub fn cast(rt: Runtime, value: Value) -> Result<()> {
            let nan = rt.atom("NAN");
            match value {
                Value::Number(n) => rt.r#return(n),
                Value::String(s) => match s.parse::<Number>() {
                    Ok(num) => rt.r#return(num),
                    Err(..) => rt.r#yield(nan, |rt, val| rt.r#return(val)),
                },
                Value::Char(ch) => rt.r#return(ch as u32),
                Value::Bool(true) => rt.r#return(1),
                Value::Bool(false) => rt.r#return(0),
                Value::Unit => rt.r#return(0),
                Value::Bits(bits) => rt.r#return(Number::from(bits)),
                _ => rt.r#yield(nan, |rt, val| rt.r#return(val)),
            }
        }
    }

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
}
