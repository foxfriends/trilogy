#[trilogy_derive::module(crate_name=crate)]
pub mod bits {
    use crate::{Result, Runtime, Value};
    use trilogy_vm::Bits;

    /// Reinterprets a value as bits. The exact structure of those bits differs based on
    /// the value being converted.
    /// * Arbitrarily large integers are written by their big endian sign-magnitude binary representation, with length rounded up to the nearest 8 bits (+ 1 for the sign).
    /// * Characters are represented by their UTF-8 codepoint.
    /// * Strings are represented by the UTF-8 codepoints of all their characters.
    /// * `true` is `0bb1` and `false` is `0bb0`
    /// * `unit` is `0bb` (the empty bit string)
    /// * Already bits values are untouched
    ///
    /// All other values (including non-integer numbers) are 'NoBits.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn cast(rt: Runtime, value: Value) -> Result<()> {
        let no_bits = rt.atom("NoBits");
        match value {
            Value::Number(n) if n.is_integer() => rt.r#return(Bits::from(n.as_integer().unwrap())),
            Value::String(s) => rt.r#return(Bits::from(s.as_str())),
            Value::Char(ch) => rt.r#return(Bits::from(ch)),
            Value::Bool(val) => rt.r#return(Bits::from(val)),
            Value::Unit => rt.r#return(Bits::new()),
            Value::Bits(bits) => rt.r#return(bits),
            _ => rt.r#yield(no_bits, |rt, val| rt.r#return(val)),
        }
    }

    /// Concatentate two bits values together.
    #[trilogy_derive::func(crate_name=crate)]
    pub fn concat(rt: Runtime, lhs: Value, rhs: Value) -> Result<()> {
        let lhs = rt.typecheck::<Bits>(lhs)?;
        let rhs = rt.typecheck::<Bits>(rhs)?;
        rt.r#return(lhs.concat(&rhs))
    }
}
