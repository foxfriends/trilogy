mod static_program;

use static_program::StaticProgram;
use trilogy_vm::{Value, VirtualMachine};

#[test]
fn test_call_fn() {
    const PROGRAM: &str = r#"
    JUMP &main
some_fn:
    CONST 1
    ADD
    RETURN
main:
    CONST &some_fn
    CONST 2
    CALL 1
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(3));
}

#[test]
fn test_recursion() {
    const PROGRAM: &str = r#"
    JUMP &main
some_fn:
    LOADL 0
    CONST 0
    VALEQ
    JUMPF &recurse
    RETURN
recurse:
    CONST &some_fn
    LOADL 0
    CONST 1
    SUB
    LOADL 0
    LOADL 1
    ADD
    CALL 2
    RETURN
main:
    CONST &some_fn
    CONST 5
    CONST 0
    CALL 2
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(15));
}

#[test]
fn test_tail_recursion() {
    const PROGRAM: &str = r#"
    JUMP &main
some_fn:
    LOADL 0
    CONST 0
    VALEQ
    JUMPF &recurse
    RETURN
recurse:
    CONST &some_fn
    LOADL 0
    CONST 1
    SUB
    LOADL 0
    LOADL 1
    ADD
    BECOME 2
main:
    CONST &some_fn
    CONST 5
    CONST 0
    CALL 2
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&StaticProgram(PROGRAM)).unwrap(), Value::from(15));
}
