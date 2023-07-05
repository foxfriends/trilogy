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
fn test_yield_invert() {
    // exit with 1 + yield 1
    //     when n invert n |> resume |> resume |> cancel
    const PROGRAM: &str = r#"
    # with (cancel)
    SHIFT 1
        EXIT
    # when (yield)
    SHIFT 22
        # 2 cancel
        # 1 resume
        # 0 n
        LOADR 1
        SWAP
        # resume
        CALL 1
        # resume
        CALL 1
        # cancel
        CALL 1
        FIZZLE
    SWAP
    # pre-yield (resume)
    SHIFT 7
        CONST 1
        ADD
        RESET
    CONST 1
    # yield
    CALL 3
    "#;
    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Number(Number::from(3)));
}

#[test]
fn test_same_stack_twice() {
    const PROGRAM: &str = r#"
    CONST 1
    SHIFT 28
    JUMPF 17
    CONST 2
    SWAP
    COPY
    CONST false
    CALL 2
    LOADR 2
    EXIT
    COPY
    CONST true
    CALL 2
    "#;
    let mut vm = VirtualMachine::load(PROGRAM.parse().unwrap());
    assert_eq!(vm.run().unwrap(), Value::Number(Number::from(1)));
}
