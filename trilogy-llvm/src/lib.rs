//! Generation of LLVM IR from Trilogy IR.
//!
//! Requires that the caller has provided all required source modules, including "trilogy:core".
//!
//! I guess in theory this means we can swap out the core library, but that's kind of weird. It
//! would probably be more reliable to include the core module from the trilogy-llvm crate directly,
//! but this is not convenient due to the compilation requirements, so it is not done.
use codegen::Codegen;
use inkwell::context::Context;
use std::{collections::HashMap, ffi::c_void};
use trilogy_ir::ir;

mod bare;
mod call;
mod codegen;
mod constant;
mod continue_in_scope;
mod core;
mod current_continuation;
mod entrypoint;
mod expression;
mod function;
mod module;
mod pattern_match;
mod procedure;
mod query;
mod rule;
mod test;
mod types;

type Entrypoint = unsafe extern "C" fn() -> c_void;

/// Parameters to rules/procedures/functions start at 5, due to return, yield, end, next, and done
const IMPLICIT_PARAMS: usize = 5;

#[repr(C)]
#[derive(Default, Debug)]
pub struct TrilogyValue {
    pub tag: u8,
    pub payload: u64,
}

fn compile<'a>(context: &'a Context, modules: &'a HashMap<String, &ir::Module>) -> Codegen<'a> {
    let mut codegen = Codegen::new(context, modules);

    log::debug!("beginning trilogy compilation");
    for (file, module) in modules {
        log::debug!("compiling module {file}");
        let submodule = codegen.compile_module(file, module, false);
        codegen.consume(submodule)
    }
    log::debug!("trilogy compilation finished");
    codegen
}

pub fn evaluate(
    modules: HashMap<String, &ir::Module>,
    entrymodule: &str,
    entrypoint: &str,
    _parameters: Vec<String>,
) -> TrilogyValue {
    let context = Context::create();
    let codegen = compile(&context, &modules);

    let mut output = TrilogyValue::default();
    codegen.compile_embedded(entrymodule, entrypoint, &mut output as *mut TrilogyValue);
    let (_module, ee) = codegen.finish();

    unsafe {
        log::debug!("locating main (compiling llvm)");
        let tri_main = ee.get_function::<Entrypoint>("main").unwrap();
        log::debug!("calling main");
        tri_main.call();
    };

    output
}

pub fn compile_to_llvm(
    modules: HashMap<String, &ir::Module>,
    entrymodule: &str,
    entrypoint: &str,
) -> String {
    let context = Context::create();
    let codegen = compile(&context, &modules);
    codegen.compile_standalone(entrymodule, entrypoint);
    let (module, _) = codegen.finish();
    module.to_string()
}

fn compile_tests<'a>(
    context: &'a Context,
    modules: &'a HashMap<String, &ir::Module>,
    filter_prefix: &[impl AsRef<str>],
) -> Codegen<'a> {
    let mut codegen = Codegen::new(context, modules);

    log::debug!("beginning trilogy compilation");
    for (file, module) in modules {
        log::debug!("compiling module {file}");
        let submodule = codegen.compile_module(file, module, true);
        codegen.consume(submodule);
    }

    codegen.compile_test_entrypoint(
        &codegen
            .tests
            .iter()
            .map(|name| name.as_str())
            .filter(|name| {
                filter_prefix
                    .iter()
                    .any(|prefix| name.starts_with(prefix.as_ref()))
            })
            .collect::<Vec<_>>(),
    );
    log::debug!("trilogy compilation finished");
    codegen
}

pub fn evaluate_tests(modules: HashMap<String, &ir::Module>, filter_prefix: &[impl AsRef<str>]) {
    let context = Context::create();
    let codegen = compile_tests(&context, &modules, filter_prefix);
    let (_module, ee) = codegen.finish();
    unsafe {
        let tri_main = ee.get_function::<Entrypoint>("main").unwrap();
        tri_main.call();
    };
}

pub fn compile_tests_to_llvm(
    modules: HashMap<String, &ir::Module>,
    filter_prefix: &[impl AsRef<str>],
) -> String {
    let context = Context::create();
    let codegen = compile_tests(&context, &modules, filter_prefix);
    let (module, _) = codegen.finish();
    module.to_string()
}
