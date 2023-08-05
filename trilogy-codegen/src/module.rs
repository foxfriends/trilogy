use crate::program::ProgramContext;
use trilogy_ir::ir;

pub(crate) fn write_module(context: &mut ProgramContext, module: &ir::Module, is_entrypoint: bool) {
    context.write_label(module.location().to_owned());

    let statics = module
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
        .map(|id| {
            let label = context.labeler.for_id(&id);
            (id, label)
        })
        .collect();

    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(..) => {}
            ir::DefinitionItem::Function(function) => {
                context.write_function(&statics, function);
            }
            ir::DefinitionItem::Rule(rule) => {
                context.write_rule(&statics, rule);
            }
            ir::DefinitionItem::Procedure(procedure) => {
                if is_entrypoint && procedure.name.id.name() == Some("main") {
                    context.write_label("main".to_owned());
                }
                context.write_procedure(&statics, procedure);
            }
            ir::DefinitionItem::Alias(..) => {}
            ir::DefinitionItem::Test(..) => {}
        }
    }
}
