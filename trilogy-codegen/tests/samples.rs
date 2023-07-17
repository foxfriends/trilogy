use std::path::PathBuf;

use trilogy_vm::{Value, VirtualMachine};

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
