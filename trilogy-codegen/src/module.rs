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
            ir::DefinitionItem::Function(function) => {
                context.write_function(function);
            }
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
        let beginning = self.labeler.begin(&procedure.name);
        self.builder.write_label(beginning).unwrap();
        let for_id = self.labeler.for_id(&procedure.name.id);
        self.builder.write_label(for_id).unwrap();
        assert!(procedure.overloads.len() == 1);
        let overload = &procedure.overloads[0];
        let context = self.begin(overload.parameters.len());
        write_procedure(context, overload);
    }

    fn write_function(&mut self, function: &ir::FunctionDefinition) {
        let beginning = self.labeler.begin(&function.name);
        self.builder.write_label(beginning).unwrap();
        let for_id = self.labeler.for_id(&function.name.id);
        self.builder.write_label(for_id).unwrap();
        let mut context = self.begin(1);

        let ret = context.labeler.unique_hint("func_return");
        let res = context.labeler.unique_hint("func_reset");
        let arity = function.overloads[0].parameters.len();
        for i in 1..arity {
            context.shift(if i == 1 { &ret } else { &res });
            context.scope.closure(1);
        }

        for overload in &function.overloads {
            write_function(&mut context, overload);
        }

        context
            .write_instruction(Instruction::Fizzle)
            .write_label(ret)
            .unwrap()
            .write_instruction(Instruction::Return)
            .write_label(res)
            .unwrap()
            .write_instruction(Instruction::Reset);
    }

    fn begin(&mut self, parameters: usize) -> Context<'_> {
        let scope = Scope::new(&self.statics, parameters);
        Context::new(self.builder, &mut self.labeler, scope)
    }
}
