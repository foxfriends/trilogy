mod static_program;

use static_program::StaticProgram;
use trilogy_vm::{Value, VirtualMachine};

#[test]
fn test_noop() {
    const PROGRAM: &str = r#"
    SHIFT &after
    RESET
    after: CONST unit
    CALL 1
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::Unit);
}

#[test]
fn test_basic() {
    const PROGRAM: &str = r#"
    SHIFT &after
    CONST 1
    ADD
    RESET
    after: CONST 3
    CALL 1
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(4));
}

#[test]
fn test_reenter() {
    const PROGRAM: &str = r#"
    SHIFT &after
    CONST 1
    ADD
    RESET
after:
    COPY
    CONST 3
    CALL 1
    CALL 1
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(5));
}

#[test]
fn test_capture() {
    const PROGRAM: &str = r#"
    CONST 1
    SHIFT &after
    LOADL 0
    ADD
    SETL 0
    LOADL 0
    RESET
    after: COPY
    CONST 1
    CALL 1
    CALL 1
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(4));
}

#[test]
fn test_yield_invert() {
    // exit with 1 + yield 1
    //     when n invert n |> resume |> resume |> cancel
    const PROGRAM: &str = r#"
    # with (cancel)
with:
    SHIFT &when         # 0 -> cancel
        EXIT
when:
    SHIFT &resume       # 1 -> when (reset)
        LOADL 2         # 2 -> cancel; 3 -> resume; 4 -> 1
        SWAP
        # resume
        CALL 1
        # resume
        CALL 1
        # cancel
        CALL 1
        FIZZLE
resume:
    SWAP                # 0 -> when; 1 -> cancel
    SHIFT &after        # 2 -> resume
        CONST 1
        ADD
        RESET
after:
    CONST 1             # 3 -> 1
    # yield
    CALL 3
    "#;
    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(3));
}

#[test]
fn test_same_stack_twice() {
    const PROGRAM: &str = r#"
    CONST 1
    SHIFT &after
    JUMPF &skip
    CONST 2
    SWAP
    COPY
    CONST false
    CALL 2
    skip:
    LOADL 0
    EXIT
    after: COPY
    CONST true
    CALL 2
    "#;
    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(1));
}
