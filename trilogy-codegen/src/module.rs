use super::{write_procedure, Context};
use trilogy_ir::ir;
use trilogy_vm::ProgramBuilder;

pub fn write_module(builder: &mut ProgramBuilder, module: &ir::Module) {
    let mut context = Context::new(builder, module.location().to_owned());

    context
        .write_label(module.location().to_owned())
        .expect("each module has a unique location and is only written once");

    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(..) => {}
            ir::DefinitionItem::Function(..) => {}
            ir::DefinitionItem::Rule(..) => {}
            ir::DefinitionItem::Procedure(procedure) => write_procedure(&mut context, procedure),
            ir::DefinitionItem::Alias(..) => {}
            ir::DefinitionItem::Test(..) => {}
        }
    }
}
