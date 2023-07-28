use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use trilogy_vm::{StructuralEq, Value, VirtualMachine};

const TEST_DIR: &str = "../samples";

macro_rules! include_tri {
    ($path:literal) => {{
        let loader = trilogy_loader::Loader::new(PathBuf::from(TEST_DIR).join($path));
        let binder = loader.load().unwrap();
        if binder.has_errors() {
            panic!(
                "Failed to compile: {:?}",
                binder.errors().collect::<Vec<_>>()
            );
        }
        let program = match binder.analyze() {
            Ok(program) => program,
            Err(errors) => {
                panic!("Failed to compile: {:?}", errors);
            }
        };
        program.generate_code()
    }};
}

#[test]
fn sample_simple() {
    let program = include_tri!("simple.tri");
    assert_eq!(VirtualMachine::load(program).run().unwrap(), Value::from(4));
}

#[test]
fn sample_call() {
    let program = include_tri!("call.tri");
    assert_eq!(VirtualMachine::load(program).run().unwrap(), Value::from(7));
}

#[test]
fn sample_tuple() {
    let program = include_tri!("tuple.tri");
    assert_eq!(VirtualMachine::load(program).run().unwrap(), Value::from(7));
}

#[test]
fn sample_disj() {
    let program = include_tri!("disj.tri");
    assert_eq!(
        VirtualMachine::load(program).run().unwrap(),
        Value::from(14)
    );
}

#[test]
fn sample_if_else() {
    let program = include_tri!("if_else.tri");
    assert_eq!(
        VirtualMachine::load(program).run().unwrap(),
        Value::from(11)
    );
}

#[test]
fn sample_record() {
    let program = include_tri!("record.tri");
    let mut map = HashMap::new();
    map.insert(Value::from(3), Value::from(5));
    map.insert(Value::from(1), Value::from(2));
    map.insert(Value::from(4), Value::from(5));
    map.insert(Value::from("hello"), Value::from("world"));
    assert!(StructuralEq::eq(
        &VirtualMachine::load(program).run().unwrap(),
        &Value::from(map)
    ));
}

#[test]
fn sample_do_closure() {
    let program = include_tri!("do_closure.tri");
    assert_eq!(VirtualMachine::load(program).run().unwrap(), Value::from(7));
}

#[test]
fn sample_array() {
    let program = include_tri!("array.tri");
    assert!(StructuralEq::eq(
        &VirtualMachine::load(program).run().unwrap(),
        &Value::from(vec![
            Value::from(1),
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into()
        ])
    ));
}

#[test]
fn sample_set() {
    let program = include_tri!("set.tri");
    let mut set = HashSet::<Value>::new();
    set.insert(1.into());
    set.insert(2.into());
    set.insert(3.into());
    set.insert(4.into());
    set.insert(5.into());
    assert!(StructuralEq::eq(
        &VirtualMachine::load(program).run().unwrap(),
        &Value::from(set)
    ));
}

#[test]
fn sample_while() {
    let program = include_tri!("while.tri");
    assert_eq!(
        VirtualMachine::load(program).run().unwrap(),
        Value::from(32)
    );
}

#[test]
fn sample_func() {
    let program = include_tri!("func.tri");
    assert_eq!(
        VirtualMachine::load(program).run().unwrap(),
        Value::from(14)
    );
}

#[test]
fn sample_fn() {
    let program = include_tri!("fn.tri");
    assert_eq!(
        VirtualMachine::load(program).run().unwrap(),
        Value::from(12)
    );
}

#[test]
fn neg_pattern() {
    let program = include_tri!("negpattern.tri");
    assert_eq!(VirtualMachine::load(program).run().unwrap(), Value::from(0));
}

#[test]
fn r#match() {
    let program = include_tri!("match.tri");
    assert_eq!(
        VirtualMachine::load(program).run().unwrap(),
        Value::from(21)
    );
}

#[test]
fn glue() {
    let program = include_tri!("glue.tri");
    assert_eq!(
        VirtualMachine::load(program).run().unwrap(),
        Value::from("worldworld")
    );
}

#[test]
fn sample_array_pattern() {
    let program = include_tri!("array_pattern.tri");
    assert!(StructuralEq::eq(
        &VirtualMachine::load(program).run().unwrap(),
        &Value::from(vec![
            Value::from(2),
            3.into(),
            4.into(),
            5.into(),
            2.into(),
            3.into(),
            4.into(),
            1.into(),
            2.into(),
            3.into(),
            4.into(),
        ])
    ));
}

#[test]
fn sample_record_pattern() {
    let program = include_tri!("record_pattern.tri");
    let mut map = HashMap::new();
    map.insert(Value::from(2), Value::from("b"));
    assert!(StructuralEq::eq(
        &VirtualMachine::load(program).run().unwrap(),
        &Value::from(map)
    ));
}
