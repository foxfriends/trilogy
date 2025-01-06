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
#[derive(Debug)]
struct TrilogyValue {
    tag: u8,
    value: [u8; 8],
}

type MainProcedure = unsafe extern "C" fn() -> TrilogyValue;

pub fn evaluate(
    modules: HashMap<String, &ir::Module>,
    path: Vec<&str>,
    parameters: Vec<String>,
) -> String {
    let context = Context::create();
    let codegen = Codegen::new(&context);
    for (file, module) in modules {
        codegen.compile_module(&file, module);
    }

    let ee = codegen
        .module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    let result = unsafe {
        let tri_main = ee.get_function::<MainProcedure>("main").unwrap();
        tri_main.call()
    };
    println!("{:?}", result);
    "Ok".to_owned()
}
