use std::collections::HashMap;

use codegen::Codegen;
use inkwell::context::Context;
use trilogy_ir::ir;

mod codegen;
mod procedure;

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
    "Ok".to_owned()
}
