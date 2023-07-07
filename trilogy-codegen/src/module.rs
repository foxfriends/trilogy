use super::{write_procedure, Labeler};
use trilogy_ir::ir;
use trilogy_vm::ProgramBuilder;

pub fn write_module(builder: &mut ProgramBuilder, module: &ir::Module) {
    let mut labeler = Labeler::new(module.location().to_owned());
    builder
        .write_label(module.location().to_owned())
        .expect("each module has a unique location and is only written once");

    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(..) => {}
            ir::DefinitionItem::Function(..) => {}
            ir::DefinitionItem::Rule(..) => {}
            ir::DefinitionItem::Procedure(procedure) => {
                write_procedure(&mut labeler, builder, procedure)
            }
            ir::DefinitionItem::Alias(..) => {}
            ir::DefinitionItem::Test(..) => {}
        }
    }
}
