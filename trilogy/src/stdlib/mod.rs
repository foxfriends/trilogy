use trilogy_vm::Value;

#[trilogy_derive::proc]
pub fn print(value: Value) {
    println!("{}", value);
}
