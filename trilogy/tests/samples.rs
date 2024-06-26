use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use trilogy::Builder;
use trilogy_vm::{Struct, StructuralEq, Value};

const TEST_DIR: &str = "../samples";

macro_rules! include_tri {
    ($path:literal) => {{
        Builder::new()
            .build_from_source(PathBuf::from(TEST_DIR).join($path))
            .unwrap()
    }};
}

#[test]
fn sample_simple() {
    let program = include_tri!("simple.tri");
    assert_eq!(program.run().unwrap(), Value::from(4));
}

#[test]
fn sample_call() {
    let program = include_tri!("call.tri");
    assert_eq!(program.run().unwrap(), Value::from(7));
}

#[test]
fn sample_tuple() {
    let program = include_tri!("tuple.tri");
    assert_eq!(program.run().unwrap(), Value::from(7));
}

#[test]
fn sample_disj() {
    let program = include_tri!("disj.tri");
    assert_eq!(program.run().unwrap(), Value::from(14));
}

#[test]
fn sample_if_else() {
    let program = include_tri!("if_else.tri");
    assert_eq!(program.run().unwrap(), Value::from(11));
}

#[test]
fn sample_record() {
    let program = include_tri!("record.tri");
    let mut map = HashMap::new();
    map.insert(Value::from(3), Value::from(5));
    map.insert(Value::from(1), Value::from(2));
    map.insert(Value::from(4), Value::from(5));
    map.insert(Value::from("hello"), Value::from("world"));
    assert!(StructuralEq::eq(&program.run().unwrap(), &Value::from(map)));
}

#[test]
fn sample_do_closure() {
    let program = include_tri!("do_closure.tri");
    assert_eq!(program.run().unwrap(), Value::from(7));
}

#[test]
fn sample_array() {
    let program = include_tri!("array.tri");
    assert!(StructuralEq::eq(
        &program.run().unwrap(),
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
    assert!(StructuralEq::eq(&program.run().unwrap(), &Value::from(set)));
}

#[test]
fn sample_while() {
    let program = include_tri!("while.tri");
    assert_eq!(program.run().unwrap(), Value::from(32));
}

#[test]
fn sample_func() {
    let program = include_tri!("func.tri");
    assert_eq!(program.run().unwrap(), Value::from(14));
}

#[test]
fn sample_fn() {
    let program = include_tri!("fn.tri");
    assert_eq!(program.run().unwrap(), Value::from(12));
}

#[test]
fn sample_negpattern() {
    let program = include_tri!("negpattern.tri");
    assert_eq!(program.run().unwrap(), Value::from(0));
}

#[test]
fn sample_match() {
    let program = include_tri!("match.tri");
    assert_eq!(program.run().unwrap(), Value::from(21));
}

#[test]
fn sample_glue() {
    let program = include_tri!("glue.tri");
    assert_eq!(program.run().unwrap(), Value::from("worldworldworld"));
}

#[test]
fn sample_array_pattern() {
    let program = include_tri!("array_pattern.tri");
    assert!(StructuralEq::eq(
        &program.run().unwrap(),
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
    assert!(StructuralEq::eq(&program.run().unwrap(), &Value::from(map)));
}

#[test]
fn sample_set_pattern() {
    let program = include_tri!("set_pattern.tri");
    let mut set = HashSet::<Value>::new();
    set.insert(3.into());
    set.insert(4.into());
    assert!(StructuralEq::eq(&program.run().unwrap(), &Value::from(set)));
}

#[test]
fn sample_compose() {
    let program = include_tri!("compose.tri");
    assert_eq!(
        program.run().unwrap(),
        Value::from((
            Struct::new(program.atom("b"), Struct::new(program.atom("a"), 1)),
            Struct::new(program.atom("a"), Struct::new(program.atom("b"), 1))
        ))
    );
}

#[test]
fn sample_op_ref() {
    let program = include_tri!("op_ref.tri");
    assert_eq!(program.run().unwrap(), Value::from(3));
}

#[test]
fn sample_while_break_continue() {
    let program = include_tri!("while_break_continue.tri");
    assert_eq!(program.run().unwrap(), Value::from(22));
}

#[test]
fn sample_while_break_continue_higher_order() {
    let program = include_tri!("while_break_continue_higher_order.tri");
    assert_eq!(program.run().unwrap(), Value::from(22));
}

#[test]
fn sample_mut_closure() {
    let program = include_tri!("mut_closure.tri");
    assert_eq!(program.run().unwrap(), Value::from(3));
}

#[test]
fn sample_first_class_return() {
    let program = include_tri!("first_class_return.tri");
    assert_eq!(program.run().unwrap(), Value::from(3));
}

#[test]
fn sample_first_class_return_closure() {
    let program = include_tri!("first_class_return_closure.tri");
    assert_eq!(program.run().unwrap(), Value::from(4));
}

#[test]
fn sample_first_class_return_returned() {
    let program = include_tri!("first_class_return_returned.tri");
    assert_eq!(program.run().unwrap(), Value::from(3));
}

#[test]
fn sample_iterator_query() {
    let program = include_tri!("iterator_query.tri");
    assert_eq!(program.run().unwrap(), Value::from(3));
}

#[test]
fn sample_iterator_collection() {
    let program = include_tri!("iterator_collection.tri");
    assert_eq!(program.run().unwrap(), Value::from(91));
}

#[test]
fn sample_collect_iterator() {
    let program = include_tri!("collect_iterator.tri");
    assert_eq!(program.run().unwrap(), Value::from(75));
}

#[test]
fn sample_handler() {
    let program = include_tri!("handler.tri");
    assert_eq!(program.run().unwrap(), Value::from(14));
}

#[test]
fn sample_first_class_resume() {
    let program = include_tri!("first_class_resume.tri");
    assert_eq!(program.run().unwrap(), Value::from(2));
}

#[test]
fn sample_query_conjunction() {
    let program = include_tri!("query_conjunction.tri");
    assert_eq!(program.run().unwrap(), Value::from(20));
}

#[test]
fn sample_query_disjunction() {
    let program = include_tri!("query_disjunction.tri");
    assert_eq!(program.run().unwrap(), Value::from(15));
}

#[test]
fn sample_for_failure() {
    let program = include_tri!("for_failure.tri");
    assert_eq!(program.run().unwrap(), Value::from(10));
}

#[test]
fn sample_query_alternative() {
    let program = include_tri!("query_alternative.tri");
    assert_eq!(program.run().unwrap(), Value::from(10));
}

#[test]
fn sample_query_implication() {
    let program = include_tri!("query_implication.tri");
    assert_eq!(program.run().unwrap(), Value::from(13));
}

#[test]
fn sample_query_not() {
    let program = include_tri!("query_not.tri");
    assert_eq!(program.run().unwrap(), Value::from(1));
}

#[test]
fn sample_reused_variable() {
    let program = include_tri!("reused_variable.tri");
    assert_eq!(program.run().unwrap(), Value::from(0));
}

#[test]
fn sample_rule() {
    let program = include_tri!("rule.tri");
    assert_eq!(program.run().unwrap(), Value::from(6));
}

#[test]
fn sample_rule_fancy() {
    let program = include_tri!("rule_fancy.tri");
    assert_eq!(program.run().unwrap(), Value::from(14));
}

#[test]
fn sample_rule_rec() {
    let program = include_tri!("rule_rec.tri");
    assert_eq!(program.run().unwrap(), Value::from(6));
}

#[test]
#[ignore = "not yet working"]
fn sample_occurs_check() {
    let program = include_tri!("occurs_check.tri");
    assert_eq!(program.run().unwrap(), Value::from(0));
}

#[test]
fn sample_modules_main() {
    let program = include_tri!("modules/main.tri");
    assert_eq!(program.run().unwrap(), Value::from(10));
}

#[test]
fn sample_module_params() {
    let program = include_tri!("module_params.tri");
    assert_eq!(program.run().unwrap(), Value::from(8));
}

#[test]
fn sample_module_private_member() {
    let program = include_tri!("module_private_member.tri");
    assert_eq!(program.run().unwrap(), Value::from(10));
}

#[test]
fn sample_export_submodule() {
    let program = include_tri!("export_submodule.tri");
    assert_eq!(program.run().unwrap(), Value::from(5));
}

#[test]
fn sample_submodule_parameters() {
    let program = include_tri!("submodule_parameters.tri");
    assert_eq!(program.run().unwrap(), Value::from(5));
}

#[test]
fn sample_module_function() {
    let program = include_tri!("module_function.tri");
    assert_eq!(program.run().unwrap(), Value::from(17));
}

#[test]
fn sample_module_rule() {
    let program = include_tri!("module_rule.tri");
    assert_eq!(program.run().unwrap(), Value::from(5));
}

#[test]
fn sample_is_expr() {
    let program = include_tri!("is_expr.tri");
    assert_eq!(program.run().unwrap(), Value::from(true));
}

#[test]
fn sample_proc_auto_return_unit() {
    let program = include_tri!("proc_auto_return_unit.tri");
    assert_eq!(program.run().unwrap(), Value::from(()));
}

#[test]
fn sample_main_no_exit() {
    let program = include_tri!("main_no_exit.tri");
    assert_eq!(program.run().unwrap(), Value::from(0));
}

#[test]
fn sample_main_return() {
    let program = include_tri!("main_return.tri");
    assert_eq!(program.run().unwrap(), Value::from(3));
}

#[test]
fn sample_evaluate_unbound() {
    let program = include_tri!("evaluate_unbound.tri");
    assert_eq!(program.run().unwrap(), Value::from(0));
}

#[test]
fn sample_evaluate_unbound_iter() {
    let program = include_tri!("evaluate_unbound_iter.tri");
    assert_eq!(program.run().unwrap(), Value::from(0));
}

#[test]
fn sample_qy() {
    let program = include_tri!("qy.tri");
    assert_eq!(program.run().unwrap(), Value::from(6));
}

#[test]
fn sample_not_main() {
    let program = Builder::new()
        .is_library(true)
        .build_from_source(PathBuf::from(TEST_DIR).join("not_main.tri"))
        .unwrap();
    assert_eq!(program.call("not_main", vec![]).unwrap(), Value::from(6));
}

#[test]
fn sample_not_main_params() {
    let program = Builder::new()
        .is_library(true)
        .build_from_source(PathBuf::from(TEST_DIR).join("not_main_params.tri"))
        .unwrap();
    assert_eq!(
        program
            .call("not_main", vec![Value::from(1), Value::from(2)])
            .unwrap(),
        Value::from(3)
    );
}
