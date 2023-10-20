#[trilogy_derive::module(crate_name=crate)]
pub mod std {
    use trilogy_vm::Value;

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn print(value: Value) {
        match value {
            Value::String(s) => println!("{s}"),
            _ => println!("{value}"),
        }
    }
}
