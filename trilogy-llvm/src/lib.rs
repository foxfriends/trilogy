use codegen::Codegen;
use inkwell::{context::Context, OptimizationLevel};
use std::collections::HashMap;
use trilogy_ir::ir;

mod codegen;
mod expression;
mod pattern_match;
mod procedure;
mod runtime;
mod scope;

#[repr(C)]
#[derive(Debug, Default)]
#[allow(dead_code, reason = "WIP")]
struct TrilogyValue {
    tag: u8,
    value: [u8; 8],
}

type Entrypoint = unsafe extern "C" fn() -> u8;

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
        codegen.module.link_in_module(submodule.module).unwrap();
    }

    codegen.compile_entrypoint(entrymodule, entrypoint);

    let ee = codegen
        .module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let result = unsafe {
        let tri_main = ee.get_function::<Entrypoint>("main").unwrap();
        tri_main.call()
    };
    println!("{result}");
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
            submodule.compile_entrypoint(entrymodule, entrypoint);
        }
        compiled.insert(file, submodule.module.to_string());
    }
    compiled
}

pub fn compile_and_link(
    modules: HashMap<String, &ir::Module>,
    entrymodule: &str,
    entrypoint: &str,
) -> String {
    let context = Context::create();
    let codegen = Codegen::new(&context);
    for (file, module) in modules {
        let submodule = codegen.compile_module(&file, module);
        codegen.module.link_in_module(submodule.module).unwrap();
    }
    codegen.compile_entrypoint(entrymodule, entrypoint);
    codegen.module.to_string()
}
