#[trilogy_derive::module(crate_name=crate)]
pub mod num {
    use crate::{Number, Result, Runtime, Value};
    use num::{BigInt, Integer, Num};

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
    #[trilogy_derive::func(crate_name=crate)]
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

    /// Parse a string in a particular base as an integer.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn parse_int(rt: Runtime, base: Value, value: Value) -> Result<()> {
        let base = rt.typecheck::<u32>(base)?;
        let value = rt.typecheck::<String>(value)?;
        let parsed = BigInt::from_str_radix(&value, base);
        match parsed {
            Ok(number) => rt.r#return(number),
            Err(..) => {
                let nan = rt.atom("NAN");
                rt.r#yield(nan, |rt, val| rt.r#return(val))
            }
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn to_fixed(rt: Runtime, digits: Value, number: Value) -> Result<()> {
        let digits = rt.typecheck::<usize>(digits)?;
        let number = rt.typecheck::<Number>(number)?;
        if let Some(int) = number.as_integer() {
            rt.r#return(format!("{int}"))
        } else if let Ok(float) = f64::try_from(&number) {
            rt.r#return(format!("{float:.*}", digits))
        } else {
            Err(rt.runtime_error(rt.atom("Unimplemented")))
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn to_fixed_auto(rt: Runtime, number: Value) -> Result<()> {
        let number = rt.typecheck::<Number>(number)?;
        if let Some(int) = number.as_integer() {
            rt.r#return(format!("{int}"))
        } else if let Ok(float) = f64::try_from(&number) {
            rt.r#return(format!("{float}"))
        } else {
            Err(rt.runtime_error(rt.atom("Unimplemented")))
        }
    }

    /// Returns the magnitude of the imaginary portion of a number, discarding the real part.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn im(rt: Runtime, value: Value) -> Result<()> {
        let nan = rt.atom("NAN");
        match value {
            Value::Number(n) => rt.r#return(n.im()),
            _ => rt.r#yield(nan, |rt, val| rt.r#return(val)),
        }
    }

    /// Returns the real portion of a number, discarding any imaginary part.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn re(rt: Runtime, value: Value) -> Result<()> {
        let nan = rt.atom("NAN");
        match value {
            Value::Number(n) => rt.r#return(n.re()),
            _ => rt.r#yield(nan, |rt, val| rt.r#return(val)),
        }
    }

    /// Returns the lowest common multiple of two numbers. These numbers must be
    /// integers or it is an error.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn lcm(rt: Runtime, lhs: Value, rhs: Value) -> Result<()> {
        let lhs = rt.typecheck::<BigInt>(lhs)?;
        let rhs = rt.typecheck::<BigInt>(rhs)?;
        rt.r#return(lhs.lcm(&rhs))
    }
}
