use crate::entrypoint::ProgramContext;
use std::collections::HashMap;
use trilogy_ir::{ir, Id};

pub(crate) fn write_module_inner(
    context: &mut ProgramContext,
    module: &ir::Module,
    mut statics: HashMap<Id, String>,
    mut modules: HashMap<Id, String>,
    is_entrypoint: bool,
) {
    statics.extend(
        module
            .definitions()
            .iter()
            .filter_map(|def| match &def.item {
                ir::DefinitionItem::Function(func) => Some(func.name.id.clone()),
                ir::DefinitionItem::Rule(rule) => Some(rule.name.id.clone()),
                ir::DefinitionItem::Procedure(proc) => Some(proc.name.id.clone()),
                // TODO: this is wrong for aliases, they are more of a compile-time transform.
                // Maybe they should be resolved at the IR phase so they can be omitted here.
                ir::DefinitionItem::Alias(alias) => Some(alias.name.id.clone()),
                ir::DefinitionItem::Module(module) if module.module.as_module().is_some() => {
                    Some(module.name.id.clone())
                }
                ir::DefinitionItem::Module(..) => None,
                ir::DefinitionItem::Test(..) => None,
            })
            .map(|id| {
                let label = context.labeler.for_id(&id);
                (id, label)
            }),
    );
    modules.extend(
        module
            .definitions()
            .iter()
            .filter_map(|def| match &def.item {
                ir::DefinitionItem::Module(module) => {
                    let path = module.module.as_external()?;
                    Some((module.name.id.clone(), path.to_owned()))
                }
                _ => None,
            }),
    );
    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(definition) => {
                context.write_module(statics.clone(), modules.clone(), definition);
            }
            ir::DefinitionItem::Function(function) => {
                context.write_function(&statics, &modules, function);
            }
            ir::DefinitionItem::Rule(rule) => {
                context.write_rule(&statics, &modules, rule);
            }
            ir::DefinitionItem::Procedure(procedure) => {
                if is_entrypoint && procedure.name.id.name() == Some("main") {
                    context.entrypoint();
                }
                context.write_procedure(&statics, &modules, procedure);
            }
            ir::DefinitionItem::Alias(..) => todo!(),
            ir::DefinitionItem::Test(..) => todo!(),
        }
    }
}
