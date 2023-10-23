mod static_program;

use static_program::StaticProgram;
use trilogy_vm::{Number, Value, VirtualMachine};

#[test]
fn test_const() {
    const PROGRAM: &str = r#"
    CONST 12
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(
        vm.run(&mut StaticProgram(PROGRAM)).unwrap(),
        Value::from(12)
    );
}

#[test]
fn test_add() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 14
    ADD
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(
        vm.run(&mut StaticProgram(PROGRAM)).unwrap(),
        Value::from(26)
    );
}

#[test]
fn test_sub() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 14
    SUB
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(
        vm.run(&mut StaticProgram(PROGRAM)).unwrap(),
        Value::from(-2)
    );
}

#[test]
fn test_div() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 14
    DIV
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(
        vm.run(&mut StaticProgram(PROGRAM)).unwrap(),
        Value::Number(Number::rational(12, 14))
    );
}

#[test]
fn test_mul() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 14
    MUL
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(
        vm.run(&mut StaticProgram(PROGRAM)).unwrap(),
        Value::from(168)
    );
}

#[test]
fn test_intdiv() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 5
    INTDIV
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&mut StaticProgram(PROGRAM)).unwrap(), Value::from(2));
}

#[test]
fn test_rem() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 5
    REM
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(vm.run(&mut StaticProgram(PROGRAM)).unwrap(), Value::from(2));
}

#[test]
#[ignore = "not yet implemented"]
fn test_pow() {
    const PROGRAM: &str = r#"
    CONST 12
    CONST 5
    POW
    EXIT
    "#;

    let vm = VirtualMachine::new();
    assert_eq!(
        vm.run(&mut StaticProgram(PROGRAM)).unwrap(),
        Value::from(248832)
    );
}
