use codegen::Codegen;
use inkwell::context::Context;
use std::{collections::HashMap, ffi::c_void, rc::Rc};
use trilogy_ir::ir;

mod bare;
mod call;
mod codegen;
mod constant;
mod core;
mod debug_info;
mod definitions;
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
