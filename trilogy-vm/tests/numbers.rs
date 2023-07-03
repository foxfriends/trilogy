use trilogy_vm::{Number, Value, VirtualMachine};

#[test]
fn test_add() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 14
    ADD
    EXIT
    "#;

    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Number(Number::from(26)));
}
