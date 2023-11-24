#[trilogy_derive::module(crate_name=crate)]
pub mod atom {
    use crate::{Result, Runtime};
    use trilogy_vm::Value;

    #[trilogy_derive::func(crate_name=crate)]
    pub fn of(rt: Runtime, value: Value) -> Result<()> {
        let value = rt.typecheck::<String>(value)?;
        let atom = rt.atom(&value);
        rt.r#return(atom)
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn make(rt: Runtime, value: Value) -> Result<()> {
        let value = rt.typecheck::<String>(value)?;
        let atom = rt.atom_anon(&value);
        rt.r#return(atom)
    }
}
