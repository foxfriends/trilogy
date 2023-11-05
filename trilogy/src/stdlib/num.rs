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
