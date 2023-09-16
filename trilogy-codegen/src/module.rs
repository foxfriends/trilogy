use std::collections::HashMap;

use crate::program::ProgramContext;
use trilogy_ir::{ir, Id};

pub(crate) fn write_module(
    context: &mut ProgramContext,
    module: &ir::Module,
    parent_statics: Option<&HashMap<Id, String>>,
    is_entrypoint: bool,
) {
    let mut statics = module
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
        .collect::<HashMap<_, _>>();
    if let Some(parent_statics) = parent_statics {
        statics.extend(parent_statics.clone());
    }
    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(definition) => {
                context.write_module(Some(&statics), definition);
            }
            ir::DefinitionItem::Function(function) => {
                context.write_function(&statics, function);
            }
            ir::DefinitionItem::Rule(rule) => {
                context.write_rule(&statics, rule);
            }
            ir::DefinitionItem::Procedure(procedure) => {
                if is_entrypoint && procedure.name.id.name() == Some("main") {
                    context.label("main");
                }
                context.write_procedure(&statics, procedure);
            }
            ir::DefinitionItem::Alias(..) => todo!(),
            ir::DefinitionItem::Test(..) => todo!(),
        }
    }
}
