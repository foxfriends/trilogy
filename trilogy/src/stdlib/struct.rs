#[trilogy_derive::module(crate_name=crate)]
pub mod r#struct {
    use crate::{Result, Runtime};
    use trilogy_vm::{Atom, Struct, Value};

    #[trilogy_derive::func(crate_name=crate)]
    pub fn construct(rt: Runtime, tag: Value, value: Value) -> Result<()> {
        let tag = rt.typecheck::<Atom>(tag)?;
        rt.r#return(Struct::new(tag, value))
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn destruct(rt: Runtime, value: Value) -> Result<()> {
        let value = rt.typecheck::<Struct>(value)?;
        rt.r#return(value.destruct())
    }
}
