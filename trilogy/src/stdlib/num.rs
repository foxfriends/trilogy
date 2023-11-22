#[trilogy_derive::module(crate_name=crate)]
pub mod num {
    use crate::{Number, Result, Runtime, Value};

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

    #[trilogy_derive::func(crate_name=crate)]
    pub fn im(rt: Runtime, value: Value) -> Result<()> {
        let nan = rt.atom("NAN");
        match value {
            Value::Number(n) => rt.r#return(n.im()),
            _ => rt.r#yield(nan, |rt, val| rt.r#return(val)),
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn re(rt: Runtime, value: Value) -> Result<()> {
        let nan = rt.atom("NAN");
        match value {
            Value::Number(n) => rt.r#return(n.re()),
            _ => rt.r#yield(nan, |rt, val| rt.r#return(val)),
        }
    }
}
