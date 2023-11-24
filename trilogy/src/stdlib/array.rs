#[trilogy_derive::module(crate_name=crate)]
pub mod array {
    use crate::{Result, Runtime};
    use trilogy_vm::{Array, Value};

    #[trilogy_derive::func(crate_name=crate)]
    pub fn length(rt: Runtime, array: Value) -> Result<()> {
        let array = rt.typecheck::<Array>(array)?;
        rt.r#return(array.len())
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn slice(rt: Runtime, start: Value, len: Value, array: Value) -> Result<()> {
        let start = rt.typecheck::<usize>(start)?;
        let len = rt.typecheck::<usize>(len)?;
        let array = rt.typecheck::<Array>(array)?;
        rt.r#return(array.into_iter().skip(start).take(len).collect::<Array>())
    }
}
