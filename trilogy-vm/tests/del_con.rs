use trilogy_vm::{Number, Value, VirtualMachine};

#[test]
fn test_noop() {
    const PROGRAM: &str = r#"
    SHIFT 1
    RESET
    CONST unit
    CALL 1
    EXIT
    "#;

    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Unit);
}

#[test]
fn test_basic() {
    const PROGRAM: &str = r#"
    SHIFT 7
    CONST 1
    ADD
    RESET
    CONST 3
    CALL 1
    EXIT
    "#;

    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Number(Number::from(4)));
}

#[test]
fn test_reenter() {
    const PROGRAM: &str = r#"
    SHIFT 7
    CONST 1
    ADD
    RESET
    COPY
    CONST 3
    CALL 1
    CALL 1
    EXIT
    "#;

    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Number(Number::from(5)));
}

#[test]
fn test_capture() {
    const PROGRAM: &str = r#"
    CONST 1
    SHIFT 17
    LOAD 2
    ADD
    SET 1
    LOAD 1
    RESET
    COPY
    CONST 1
    CALL 1
    CALL 1
    EXIT
    "#;

    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Number(Number::from(4)));
}
