use trilogy_vm::Value;

#[trilogy_derive::func(crate_name=crate, name=regex)]
pub struct Regex(Value);

#[trilogy_derive::module(crate_name=crate)]
impl Regex {}
