use super::{write_procedure, Context};
use trilogy_ir::ir;
use trilogy_vm::ProgramBuilder;

pub fn write_module(builder: &mut ProgramBuilder, module: &ir::Module, is_entrypoint: bool) {
    let mut context = Context::new(builder, module.location().to_owned());

    context
        .write_label(module.location().to_owned())
        .expect("each module has a unique location and is only written once");

    for id in module
        .definitions()
        .iter()
        .filter_map(|def| match &def.item {
            ir::DefinitionItem::Module(module) => Some(module.name.id.clone()),
            ir::DefinitionItem::Function(func) => Some(func.name.id.clone()),
            ir::DefinitionItem::Rule(rule) => Some(rule.name.id.clone()),
            ir::DefinitionItem::Procedure(proc) => Some(proc.name.id.clone()),
            // TODO: this is wrong for aliases, they are more of a compile-time transform.
            // Maybe they should be resolved at the IR phase so they can be omitted here.
            ir::DefinitionItem::Alias(alias) => Some(alias.name.id.clone()),
            ir::DefinitionItem::Test(..) => None,
        })
    {
        let label = context.labeler.for_id(&id);
        context.scope.declare_label(id, label);
    }

    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(..) => {}
            ir::DefinitionItem::Function(..) => {}
            ir::DefinitionItem::Rule(..) => {}
            ir::DefinitionItem::Procedure(procedure) => {
                if is_entrypoint && procedure.name.id.name() == Some("main") {
                    context.write_label("main".to_owned()).unwrap();
                }
                write_procedure(&mut context, procedure);
            }
            ir::DefinitionItem::Alias(..) => {}
            ir::DefinitionItem::Test(..) => {}
        }
    }
}
