use crate::{
    context::{Labeler, Scope},
    prelude::*,
};
use std::collections::HashMap;
use trilogy_ir::{ir, Id};
use trilogy_vm::{Instruction, ProgramBuilder};

struct ModuleContext<'a> {
    builder: &'a mut ProgramBuilder,
    statics: HashMap<Id, String>,
    labeler: Labeler,
}

impl<'a> ModuleContext<'a> {
    fn new(builder: &'a mut ProgramBuilder, location: String) -> Self {
        Self {
            builder,
            labeler: Labeler::new(location),
            statics: HashMap::default(),
        }
    }
}

pub fn write_module(builder: &mut ProgramBuilder, module: &ir::Module, is_entrypoint: bool) {
    let mut context = ModuleContext::new(builder, module.location().to_owned());

    context
        .builder
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
        context.statics.insert(id, label);
    }

    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(..) => {}
            ir::DefinitionItem::Function(..) => {}
            ir::DefinitionItem::Rule(..) => {}
            ir::DefinitionItem::Procedure(procedure) => {
                if is_entrypoint && procedure.name.id.name() == Some("main") {
                    context.builder.write_label("main".to_owned()).unwrap();
                }
                context.write_procedure(procedure);
            }
            ir::DefinitionItem::Alias(..) => {}
            ir::DefinitionItem::Test(..) => {}
        }
    }
}

impl ModuleContext<'_> {
    fn write_procedure(&mut self, procedure: &ir::ProcedureDefinition) {
        let beginning = self.labeler.begin_procedure(&procedure.name);
        self.builder.write_label(beginning).unwrap();
        let for_id = self.labeler.for_id(&procedure.name.id);
        self.builder.write_label(for_id).unwrap();
        for overload in &procedure.overloads {
            let on_fail = self.labeler.unique();
            let context = self.begin(overload.parameters.len());
            write_procedure(context, overload, &on_fail);
            self.builder.write_label(on_fail).unwrap();
        }
        self.builder.write_instruction(Instruction::Fizzle);
    }

    fn begin(&mut self, parameters: usize) -> Context<'_> {
        let scope = Scope::new(&self.statics, parameters);
        Context::new(self.builder, &mut self.labeler, scope)
    }
}
