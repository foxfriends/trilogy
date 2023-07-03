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
    LOADR 2
    ADD
    SETR 1
    LOADR 1
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

#[test]
#[ignore = "incomplete"]
fn test_yield_invert() {
    const PROGRAM: &str = r#"
    # with
    SHIFT 1
        EXIT
    # when
    SHIFT 17
        COPY
        # resume
        CALL 1
        CALL 2
        # cancel
        RESET
    # yield
    SHIFT 1
        RESET
    SWAP
    CALL 3
    EXIT
    "#;
    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Number(Number::from(4)));
}
