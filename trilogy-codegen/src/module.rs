use crate::entrypoint::{ProgramContext, StaticMember};
use crate::preamble::{END, RETURN};
use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use trilogy_ir::{ir, visitor::HasBindings, Id};
use trilogy_vm::{Instruction, Value};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Mode {
    Program,
    Module,
    Document,
}

pub(crate) fn write_module_definitions(
    context: &mut ProgramContext,
    module: &ir::Module,
    statics: &HashMap<Id, StaticMember>,
    mode: Mode,
) {
    // Here's where all the definitions follow.
    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Module(definition) if definition.module.as_module().is_some() => {
                context.write_module(statics.clone(), definition);
            }
            ir::DefinitionItem::Function(function) => {
                context.write_function(statics, function);
            }
            ir::DefinitionItem::Rule(rule) => {
                context.write_rule(statics, rule);
            }
            ir::DefinitionItem::Procedure(procedure) => {
                if mode == Mode::Program && procedure.name.id.name() == Some("main") {
                    // When this is the entrypoint of the program, the entrypoint of the chunk
                    // is the main function, which we have just found here.
                    //
                    // If there is no main... we'll have to raise some error I suppose.
                    context.entrypoint();
                    context.label("trilogy:__entrypoint__");
                }
                context.write_procedure(statics, procedure);
            }
            ir::DefinitionItem::Alias(..) => todo!(),
            ir::DefinitionItem::Test(..) => todo!(),
            // Imported modules are not written
            ir::DefinitionItem::Module(..) => {}
        }
    }
}

pub(crate) fn write_module_prelude(
    context: &mut Context,
    module: &ir::Module,
) -> HashMap<Id, StaticMember> {
    // Start by extracting all the parameters. Declare them all up front so
    // that we can be sure about their ordering.
    let module_parameters = module
        .parameters
        .iter()
        .flat_map(|param| param.bindings())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    context.declare_variables(module_parameters.iter().cloned());
    for _ in 1..module.parameters.len() {
        context.close(RETURN);
    }
    for (i, parameter) in module.parameters.iter().enumerate() {
        context.instruction(Instruction::LoadLocal(i as u32));
        write_pattern_match(context, parameter, END);
    }
    // Then save the extracted bindings into an array which will be stored in the
    // module object. Functions will be able to reference these variables by pulling
    // them from the array.
    context.instruction(Instruction::Const(Vec::<Value>::new().into()));
    for _ in &module_parameters {
        context
            .instruction(Instruction::Swap)
            .instruction(Instruction::Insert);
    }

    // For definitions to actually access these parameters, they're defined as 0-arity
    // functions. That are aware of the module convention.
    let mut statics_for_later = HashMap::new();
    for (i, var) in module_parameters.iter().rev().enumerate() {
        let label = context.labeler.for_id(var);
        context.label(&label);
        context
            .instruction(Instruction::LoadRegister(1))
            .instruction(Instruction::Const(i.into()))
            .instruction(Instruction::Access)
            .instruction(Instruction::Return);
        statics_for_later.insert(var.clone(), StaticMember::Label(label));
    }
    context.undeclare_variables(module_parameters, false);

    // Next a closure is created that defines the exports of this module. This function
    // is the public reification of the module.
    //
    // The current module's parameters are stored into register 1 such that when a function
    // is called, its parameters can be located. Through this public interface that
    // invariant is upheld.

    let current_module = context.scope.intermediate();
    // Put in a case for each public method. The name of the method will be expected
    // as an atom.
    context.close(RETURN);
    for def in module.definitions() {
        if def.is_exported {
            let next_export = context.labeler.unique_hint("next_export");
            let name = def
                .name()
                .unwrap()
                .name()
                .expect("ids of definitions have names");
            let atom = context.atom(name);
            context
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(atom.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&next_export)
                .instruction(Instruction::Pop);

            match &def.item {
                ir::DefinitionItem::Function(_func) => todo!("support exported functions"),
                ir::DefinitionItem::Procedure(proc) => {
                    let Some(StaticMember::Label(proc_label)) =
                        context.scope.lookup_static(&proc.name.id).cloned()
                    else {
                        unreachable!("definitions will be found");
                    };
                    // Procedure only has one overload. All overloads would have the same arity anyway.
                    let arity = proc.overloads[0].parameters.len();
                    context
                        .close(RETURN)
                        .instruction(Instruction::LoadRegister(1));
                    context
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::SetLocal(current_module))
                        .write_procedure_reference(proc_label)
                        .instruction(Instruction::Slide(arity as u32))
                        .instruction(Instruction::Call(arity as u32))
                        .instruction(Instruction::LoadRegister(1))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::SetLocal(current_module))
                        .instruction(Instruction::Return);
                }
                ir::DefinitionItem::Module(_submod) => {}
                ir::DefinitionItem::Alias(_alias) => todo!("support exported aliases"),
                ir::DefinitionItem::Rule(_rule) => todo!("support exported rules"),
                ir::DefinitionItem::Test(..) => unreachable!("tests cannot be exported"),
            }
            context.label(next_export);
        }
    }

    context.jump(END);
    context.scope.end_intermediate();
    statics_for_later
}
