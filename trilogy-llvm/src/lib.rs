//! Generation of LLVM IR from Trilogy IR.
//!
//! Requires that the caller has provided all required source modules, including "trilogy:core".
//! In particular the existence of `core::to_string` is assumed.
//!
//! I guess in theory this means we can swap out the core library, but that's kind of weird. It
//! would probably be more reliable to include the core module from the trilogy-llvm crate directly,
//! but this is not convenient due to the compilation requirements, so it is not done.
use codegen::Codegen;
use inkwell::context::Context;
use std::{collections::HashMap, ffi::c_void, rc::Rc};
use trilogy_ir::ir;

mod bare;
mod call;
mod codegen;
mod constant;
mod core;
mod current_continuation;
mod entrypoint;
mod expression;
mod module;
mod pattern_match;
mod procedure;
mod types;

type Entrypoint = unsafe extern "C" fn() -> c_void;

#[repr(C)]
#[derive(Default, Debug)]
pub struct TrilogyValue {
    pub tag: u32,
    pub payload: u64,
}

pub fn evaluate(
    modules: HashMap<String, &ir::Module>,
    entrymodule: &str,
    entrypoint: &str,
    _parameters: Vec<String>,
) -> TrilogyValue {
    let context = Context::create();
    let codegen = Codegen::new(&context, &modules);

    for (file, module) in &modules {
        let submodule = codegen.compile_module(file, module);
        submodule.di.builder.finalize();
        codegen
            .module
            .link_in_module(Rc::into_inner(submodule.module).unwrap())
            .unwrap();
    }

    let mut output = TrilogyValue::default();
    codegen.compile_embedded(entrymodule, entrypoint, &mut output as *mut TrilogyValue);
    let (_module, ee) = codegen.finish();

    unsafe {
        let tri_main = ee.get_function::<Entrypoint>("main").unwrap();
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
    let codegen = Codegen::new(&context, &modules);

    for (file, module) in &modules {
        let submodule = codegen.compile_module(file, module);
        submodule.di.builder.finalize();
        codegen
            .module
            .link_in_module(Rc::into_inner(submodule.module).unwrap())
            .unwrap();
    }

    codegen.compile_standalone(entrymodule, entrypoint);
    let (module, _) = codegen.finish();
    module.to_string()
}
