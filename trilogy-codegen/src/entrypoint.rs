use super::context::ProgramContext;
use super::module::Mode;
use crate::prelude::*;
use std::collections::HashMap;
use trilogy_ir::ir;
use trilogy_vm::{ChunkBuilder, ChunkWriter, Value};

pub fn write_program(
    file: &str,
    builder: &mut ChunkBuilder,
    module: &ir::Module,
    entry_path: &[&str],
) {
    let mut context = ProgramContext::new(file, builder);
    context.write_main(entry_path, 0.into());
    write_preamble(&mut context);

    let mut statics = HashMap::default();
    context.collect_static(module, &mut statics);

    context.label("trilogy:__entrymodule__");
    // Parameters len will be 0, but let's write it out anyway
    let mut precontext = context.begin(&mut statics, module.parameters.len());
    write_module_prelude(&mut precontext, module, Mode::Document);
    write_module_definitions(&mut context, module, &mut statics, Mode::Document);
}

pub fn write_module(file: &str, builder: &mut ChunkBuilder, module: &ir::Module) {
    let mut context = ProgramContext::new(file, builder);
    let mut statics = HashMap::default();
    context.collect_static(module, &mut statics);
    context.entrypoint();
    // Parameters len will be 0, but let's write it out anyway
    let mut precontext = context.begin(&mut statics, module.parameters.len());
    write_module_prelude(&mut precontext, module, Mode::Document);
    write_module_definitions(&mut context, module, &mut statics, Mode::Document);
}

pub fn write_test(
    file: &str,
    builder: &mut ChunkBuilder,
    module: &ir::Module,
    path: &[&str],
    test: &str,
) {
    let mut context = ProgramContext::new(file, builder);
    let mut full_path = path.to_vec();
    full_path.push("trilogy:__testentry__");
    context.write_main(&full_path, Value::Unit);
    write_preamble(&mut context);

    let mut statics = HashMap::default();
    context.collect_static(module, &mut statics);

    // Parameters len will be 0, but let's write it out anyway
    context.label("trilogy:__entrymodule__");
    let mut precontext = context.begin(&mut statics, module.parameters.len());
    write_module_prelude(&mut precontext, module, Mode::Test(path, test));
    write_module_definitions(&mut context, module, &mut statics, Mode::Test(path, test));
}
