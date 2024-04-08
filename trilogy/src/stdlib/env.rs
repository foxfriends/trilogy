#[trilogy_derive::module(crate_name=crate)]
pub mod env {
    use crate::{Array, Record, Result, Runtime, Value};

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn vars(rt: Runtime) -> Result<()> {
        rt.r#return(
            std::env::vars()
                .map(|(k, v)| (Value::from(k), Value::from(v)))
                .collect::<Record>(),
        )
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn var(rt: Runtime, name: Value) -> Result<()> {
        let name = rt.typecheck::<String>(name)?;
        rt.r#return(std::env::var(name).ok())
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn args(rt: Runtime) -> Result<()> {
        rt.r#return(std::env::args().map(Value::from).collect::<Array>())
    }
}
