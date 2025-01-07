use std::collections::HashMap;

use codegen::Codegen;
use inkwell::{context::Context, OptimizationLevel};
use trilogy_ir::ir;

mod codegen;
mod expression;
mod pattern_match;
mod procedure;
mod scope;

#[repr(C)]
#[derive(Debug, Default)]
#[allow(dead_code, reason = "WIP")]
struct TrilogyValue {
    tag: u8,
    value: [u8; 8],
}

type MainProcedure = unsafe extern "C" fn() -> i64;

pub fn evaluate(
    modules: HashMap<String, &ir::Module>,
    entrymodule: &str,
    entrypoint: &str,
    _parameters: Vec<String>,
) -> String {
    let context = Context::create();
    let codegen = Codegen::new(&context);
    for (file, module) in modules {
        let submodule = codegen.compile_module(&file, module);
        if file == entrymodule {
            submodule.compile_entrypoint(entrypoint);
        }
        codegen.module.link_in_module(submodule.module).unwrap();
    }

    let ee = codegen
        .module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    unsafe {
        let tri_main = ee.get_function::<MainProcedure>("__#trilogy_main").unwrap();
        tri_main.call();
    };
    "Ok".to_owned()
}

pub fn compile(
    modules: HashMap<String, &ir::Module>,
    entrymodule: &str,
    entrypoint: &str,
) -> HashMap<String, String> {
    let context = Context::create();
    let codegen = Codegen::new(&context);
    let mut compiled = HashMap::with_capacity(modules.len() + 1);
    compiled.insert("trilogy:runtime".to_owned(), codegen.module.to_string());
    for (file, module) in modules {
        let submodule = codegen.compile_module(&file, module);
        if file == entrymodule {
            submodule.compile_entrypoint(entrypoint);
        }
        compiled.insert(file, submodule.module.to_string());
    }
    compiled
}
