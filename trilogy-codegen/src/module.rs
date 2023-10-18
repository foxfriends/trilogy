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
            ir::DefinitionItem::Test(..) => todo!(),
            // Imported modules are not written
            ir::DefinitionItem::Module(..) => {}
        }
    }
}

pub(crate) fn write_module_prelude(
    context: &mut Context,
    module: &ir::Module,
    mode: Mode,
) -> HashMap<Id, StaticMember> {
    // The parameters were all declared ahead of time +1 for the module member
    for _ in 0..module.parameters.len() {
        context.close(RETURN);
    }
    // Collect the module parameters before declaring them so we can be sure of their
    // ordering.
    let module_parameters = module
        .parameters
        .iter()
        .flat_map(|param| param.bindings())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    context.declare_variables(module_parameters.iter().cloned());
    for (i, parameter) in module.parameters.iter().enumerate() {
        context.instruction(Instruction::LoadLocal(i as u32));
        write_pattern_match(context, parameter, END);
    }
    // Then save the extracted bindings into an array which will be stored in the
    // module object. Functions will be able to reference these variables by pulling
    // them from the array.
    if mode == Mode::Document {
        // Documents do not have access to their parent module's parameters, so they
        // get a fresh context array.
        context.instruction(Instruction::Const(Vec::<Value>::new().into()));
    } else {
        // Submodules do, and they are prefixed to the parameters. It so happens
        // that the parent's statics will already cover these, so the only consideration
        // is that the new module's parameter accessors need to know how many parameters
        // are in the parent module such that they know which index to start pulling from.
        context
            .instruction(Instruction::LoadRegister(1))
            .instruction(Instruction::Clone);
    }
    for _ in &module_parameters {
        context
            .instruction(Instruction::Swap)
            .instruction(Instruction::Insert);
    }

    context.undeclare_variables(module_parameters.iter().cloned(), false);

    // The current module's parameters are stored into register 1 such that when a function
    // is called, its parameters can be located. Through this public interface that
    // invariant is upheld.
    let current_module = context.scope.intermediate();
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
                .instruction(Instruction::LoadLocal(module.parameters.len() as u32))
                .instruction(Instruction::Const(atom.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&next_export);

            match &def.item {
                ir::DefinitionItem::Function(func) => {
                    // All overloads must have the same arity, so get from the first one.
                    let function_arity = func.overloads[0].parameters.len();
                    let Some(StaticMember::Label(label)) =
                        context.scope.lookup_static(&func.name.id).cloned()
                    else {
                        unreachable!("definitions will be found as a local label");
                    };
                    // Capture all the parameters up front.
                    for _ in 0..function_arity {
                        context.close(RETURN);
                    }
                    // Once all parameters are located, set up the context register
                    // and call the function by applying the parameters one by one.
                    context
                        .instruction(Instruction::LoadRegister(1))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(1))
                        .write_procedure_reference(label);
                    for i in 0..function_arity {
                        context
                            .instruction(Instruction::LoadLocal(current_module + i as u32 + 1))
                            .instruction(Instruction::Call(1));
                    }
                    // After every parameter was passed, then we have the return value which is
                    // no longer subject to the context rules, so return the context register.
                    context
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::Return);
                }
                ir::DefinitionItem::Procedure(proc) => {
                    let Some(StaticMember::Label(proc_label)) =
                        context.scope.lookup_static(&proc.name.id).cloned()
                    else {
                        unreachable!("definitions will be found as a local label");
                    };
                    // Procedure only has one overload. All overloads would have the same arity anyway.
                    let arity = proc.overloads[0].parameters.len();
                    context
                        .close(RETURN)
                        .instruction(Instruction::LoadRegister(1))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::Slide(arity as u32))
                        .write_procedure_reference(proc_label)
                        .instruction(Instruction::Slide(arity as u32))
                        .instruction(Instruction::Call(arity as u32))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::Return);
                }
                ir::DefinitionItem::Module(submodule) => {
                    let static_member = context
                        .scope
                        .lookup_static(&submodule.name.id)
                        .unwrap()
                        .clone();
                    match static_member {
                        StaticMember::Chunk(path) => {
                            context
                                .instruction(Instruction::Chunk(path.into()))
                                .instruction(Instruction::Return);
                        }
                        StaticMember::Label(label) => {
                            let submodule_arity =
                                submodule.module.as_module().unwrap().parameters.len();
                            // Capture all the parameters up front.
                            for _ in 0..submodule_arity + 1 {
                                context.close(RETURN);
                            }
                            // Once all parameters are located, set up the context register
                            // and call the module by applying the parameters one by one.
                            context
                                .instruction(Instruction::LoadRegister(1))
                                .instruction(Instruction::LoadLocal(current_module))
                                .instruction(Instruction::SetRegister(1))
                                .write_procedure_reference(label);
                            for i in 0..submodule_arity + 1 {
                                // Call with each arg, in order
                                context
                                    .instruction(Instruction::LoadLocal(
                                        current_module + i as u32 + 1,
                                    ))
                                    .instruction(Instruction::Call(1));
                            }
                            // After every parameter was passed, then we have the return value which is
                            // no longer subject to the context rules, so replace the context register.
                            context
                                .instruction(Instruction::Swap)
                                .instruction(Instruction::SetRegister(1))
                                .instruction(Instruction::Return);
                        }
                        StaticMember::Context(..) => unreachable!(),
                    }
                }
                ir::DefinitionItem::Rule(rule) => {
                    let Some(StaticMember::Label(static_member)) =
                        context.scope.lookup_static(&rule.name.id).cloned()
                    else {
                        unreachable!("definitions will be found as a local label");
                    };
                    let arity = rule.overloads[0].parameters.len();
                    // It's a rule, so it will be immediately called to perform setup.
                    // That setup does not require the context... I think. But it's easy
                    // to pass anyway and better safe than sorry, I suppose.
                    context
                        .close(RETURN)
                        .instruction(Instruction::LoadRegister(1))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(1))
                        .write_procedure_reference(static_member)
                        .instruction(Instruction::Call(0))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(1))
                        // That will produce the rule's closure, but we need to re-close that
                        // over the module scope
                        .close(RETURN);
                    let closure = context.scope.intermediate();
                    context
                        // When called again, it's to pass the parameters. Again, might not need
                        // the context, but let's pass it anyway.
                        .instruction(Instruction::LoadRegister(1))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::Slide(arity as u32))
                        .instruction(Instruction::LoadLocal(closure))
                        .instruction(Instruction::Slide(arity as u32))
                        .instruction(Instruction::Call(arity as u32))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(1))
                        // That needs to be closed over to keep passing context too
                        .close(RETURN);
                    let iterator = context.scope.intermediate();
                    context
                        .instruction(Instruction::LoadRegister(1))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::LoadLocal(iterator))
                        .instruction(Instruction::Call(0))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(1))
                        .instruction(Instruction::Return);
                }
                ir::DefinitionItem::Test(..) => unreachable!("tests cannot be exported"),
            }
            context.label(next_export);
        }
    }

    context.jump(END);
    context.scope.end_intermediate();

    // For definitions to actually access the module parameters, they're defined as
    // 0-arity functions that are aware of the module convention.
    let mut statics_for_later = HashMap::new();
    let base = context.scope.context_size();
    for (i, var) in module_parameters.iter().rev().enumerate() {
        let label = context.labeler.for_id(var);
        context.label(&label);
        context
            .instruction(Instruction::LoadRegister(1))
            .instruction(Instruction::Const((base + i).into()))
            .instruction(Instruction::Access)
            .instruction(Instruction::Return);
        statics_for_later.insert(var.clone(), StaticMember::Context(label));
    }
    statics_for_later
}
