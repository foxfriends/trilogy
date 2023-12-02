use crate::context::{ProgramContext, StaticMember};
use crate::preamble::{END, RETURN};
use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use trilogy_ir::{ir, visitor::HasBindings, Id};
use trilogy_vm::{Instruction, Value};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Mode<'a> {
    Module,
    Document,
    Test(&'a [&'a str], &'a str),
}

impl Mode<'_> {
    fn child(self) -> Self {
        match self {
            Self::Document | Self::Module => Self::Module,
            Self::Test([], ..) => Self::Module,
            Self::Test(path, name) => Self::Test(&path[1..], name),
        }
    }
}

pub(crate) fn write_module_definitions(
    context: &mut ProgramContext,
    module: &ir::Module,
    statics: &mut HashMap<Id, StaticMember>,
    mode: Mode,
) {
    // Here's where all the definitions follow.
    for def in module.definitions() {
        match &def.item {
            ir::DefinitionItem::Constant(..) => {} // Constants are written directly in the prelude
            ir::DefinitionItem::Module(definition) if definition.module.as_module().is_none() => {} // Imported modules are in prelude too
            ir::DefinitionItem::Module(definition) => {
                // Regular modules are partially in prelude, but also they have a body.
                context.write_module(statics.clone(), definition, mode.child());
            }
            ir::DefinitionItem::Function(function) => {
                context.write_function(statics, function);
            }
            ir::DefinitionItem::Rule(rule) => {
                context.write_rule(statics, rule);
            }
            ir::DefinitionItem::Procedure(procedure) => {
                context.write_procedure(statics, procedure);
            }
            ir::DefinitionItem::Test(test) if mode == Mode::Test(&[], &test.name) => {
                context.write_test(statics, test);
            }
            ir::DefinitionItem::Test(..) => {}
        }
    }
}

/// Writes the prelude of the module. The prelude consists of:
/// 1. Accepting all parameters
/// 2. Binding the parameters to their variables
/// 3. Evaluating constants
/// 4. Initializing submodules
/// 5. Creating the exported member lookup function
/// 6. Defining accessors for context values
///
/// During constant evaluation, variables must be checked for boundness. If they
/// are not yet bound, then execution fizzles.
///
/// This does *not* take into account the order in which declarations should
/// be evaluated. That must be resolved ahead of time... by someone else.
/// Really all that matters is that every value is evaluated before any
/// expression that references it. The better that resolution, the less likely
/// we have false-positive circular dependencies.
///
/// The tricky part is that inline modules may reference a lot of things from
/// their parent module while the parent module may also be referencing things
/// from the child... The suggested ordering then:
/// 1. Constants that don't reference child modules
/// 2. Constants that reference parameterless child modules
/// 3. Constants that reference parameterized child modules
pub(crate) fn write_module_prelude(context: &mut Context, module: &ir::Module, mode: Mode) {
    // Record how many values are in the parent context ahead of time. We'll be modifying
    // the context with new entries shortly.
    let base = context.scope.context_size();

    // 1. Accepting all parameters
    //
    // The module itself is a procedure with 0 arity, which should be called immediately after
    // loading the chunk.
    //
    // Its parameters are passed next like function parameters, one at a time.
    for _ in 0..module.parameters.len() {
        context.close(RETURN).unlock_function();
    }

    // 2. Binding the parameters to their variables
    //
    // Collect the module parameters before declaring them so we can be sure of their
    // ordering later.
    let variables = module
        .parameters
        .iter()
        .flat_map(|param| param.bindings())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    context.declare_variables(variables.iter().cloned());
    for (i, parameter) in module.parameters.iter().enumerate() {
        context
            .instruction(Instruction::LoadLocal(i as u32))
            .pattern_match(parameter, END);
    }

    // Then save the extracted bindings into an array which will be stored in the
    // module object. Further declarations will be able to reference these variables
    // by pulling them from the array.
    if mode == Mode::Document {
        // Documents do not have access to their parent module's parameters, so they
        // get a fresh context array.
        context.constant(Vec::<Value>::new());
    } else {
        // Submodules do, and they are prefixed to the parameters. It so happens
        // that the parent's statics will already cover these, so the only consideration
        // is that the new module's parameter accessors need to know how many parameters
        // are in the parent module such that they know which index to start pulling from.
        context
            .instruction(Instruction::LoadRegister(MODULE))
            .instruction(Instruction::Clone);
    }
    for _ in 0..variables.len() {
        context
            .instruction(Instruction::Swap)
            .instruction(Instruction::Insert);
    }
    context.undeclare_variables(variables.clone(), false);

    // The variables actually just got inserted all in reverse order, so when we go
    // to record them with their indices in the scope array, flip them around first.
    let mut variables = variables.into_iter().rev().enumerate().collect::<Vec<_>>();
    let current_module = context.scope.intermediate();
    // These variables are now in the context, so mark them down as such.
    for (_, var) in &variables {
        let label = context.make_label(&format!("context::{}", var.symbol()));
        context
            .scope
            .declare_static(var.clone(), StaticMember::Context(label));
    }

    // Constants are evaluated ahead of time and stored very much the same
    // as parameter variables. They are just like derived values after all.
    //
    // TODO: this does not account for non-source-order dependencies of
    // constants.
    //
    // Modules are treated like constants too, so that initialization
    // of modules with no parameters is performed up front once.
    //
    // Submodules are evaluated and initialized after constants so that
    // they can reference constants and parameters.
    //
    // TODO: this still does not account for external modules that are
    // referenced from multiple files, they still get initialized once
    // per file.
    //
    // TODO: this also does not account for circular dependencies or
    // even non-source-order dependencies of modules. Circular dependencies
    // can probably be rejected at analysis, but non-source-order eventually
    // needs support. Just a DAG, probably easy to build as part of the
    // same analysis for cycles.
    //
    // We'll just assume dependency detection is eventually going to be
    // implemented by sorting the definitions array correctly.
    //
    // After each thing is evaluated, it's stored into the scope array
    // so that later declarations can access them. Context passing is
    // simulated so that it works like normal.
    context
        .instruction(Instruction::LoadRegister(MODULE))
        .instruction(Instruction::LoadLocal(current_module))
        .instruction(Instruction::SetRegister(MODULE));
    context.scope.intermediate(); // previous module

    // 3. Evaluating constants
    let mut old_statics = HashMap::new();
    for def in module.definitions() {
        let declared_id = match &def.item {
            ir::DefinitionItem::Constant(constant) => {
                // TODO: there's no real reason why constant doesn't allow pattern
                // matching + multiple names except that I am lazy. It will be added
                // eventually... probably.
                context.evaluate(&constant.value.value);
                Some(&constant.name.id)
            }
            ir::DefinitionItem::Module(definition) => {
                let static_member = context
                    .scope
                    .lookup_static(&definition.name.id)
                    .unwrap()
                    .clone();
                match static_member {
                    StaticMember::Chunk(location) => {
                        context
                            .instruction(Instruction::Chunk(location.into()))
                            .instruction(Instruction::Call(0));
                    }
                    StaticMember::Label(..) => {
                        // Local modules get written into the context as `unit` first,
                        // ensuring that they are reserved space in the context without
                        // initializing them. The initialization comes later.
                        context.constant(());
                    }
                    StaticMember::Context(..) => unreachable!(),
                }
                Some(&definition.name.id)
            }
            ir::DefinitionItem::Function(definition) => {
                let label = context
                    .scope
                    .lookup_static(&definition.name.id)
                    .unwrap()
                    .clone()
                    .unwrap_label();
                // Local functions are inserted into the context as closures
                // over the module context.
                //
                // Same for procedures and rules.
                context.func_closure(definition.overloads[0].parameters.len(), label);
                Some(&definition.name.id)
            }
            ir::DefinitionItem::Procedure(definition) => {
                let label = context
                    .scope
                    .lookup_static(&definition.name.id)
                    .unwrap()
                    .clone()
                    .unwrap_label();
                context.proc_closure(definition.overloads[0].parameters.len(), label);
                Some(&definition.name.id)
            }
            ir::DefinitionItem::Rule(definition) => {
                let label = context
                    .scope
                    .lookup_static(&definition.name.id)
                    .unwrap()
                    .clone()
                    .unwrap_label();
                context.rule_closure(definition.overloads[0].parameters.len(), label);
                Some(&definition.name.id)
            }
            _ => None,
        };

        if let Some(declared_id) = declared_id {
            // After each declaration of this sort, we have to do this whole dance
            // to keep the running state up to date. Blegh.
            context
                .instruction(Instruction::LoadRegister(MODULE))
                .instruction(Instruction::Swap)
                .instruction(Instruction::Insert)
                .instruction(Instruction::SetRegister(MODULE));
            let label = context.make_label(&format!("context::{}", declared_id.symbol()));
            let old_static = context
                .scope
                .declare_static(declared_id.clone(), StaticMember::Context(label))
                .unwrap();
            old_statics.insert(declared_id.clone(), old_static);
            variables.push((variables.len(), declared_id.clone()));
        }
    }

    // 4. Initializing submodules
    for def in module
        .definitions()
        .iter()
        .filter(|def| def.item.as_module().is_some())
    {
        let module = def.item.as_module().unwrap();
        let old_static = old_statics.get(&module.name.id).unwrap();
        match old_static {
            // Already initialized. But why?
            StaticMember::Chunk(..) => {}
            StaticMember::Label(label) => {
                let (index, _) = variables
                    .iter()
                    .find(|(_, id)| id == &module.name.id)
                    .unwrap();
                context
                    .reference(label)
                    .instruction(Instruction::Call(0))
                    .instruction(Instruction::LoadRegister(MODULE))
                    .instruction(Instruction::Swap)
                    .constant(base + index)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Assign)
                    .instruction(Instruction::SetRegister(MODULE));
            }
            StaticMember::Context(..) => unreachable!(),
        }
    }

    context.instruction(Instruction::SetRegister(MODULE));
    context.scope.end_intermediate(); // previous module

    // 5. Creating the exported member lookup function
    //
    // After initialization is complete, there is one more closure which is the one
    // that accepts the symbol to be imported.
    //
    // This must be after initialization so that re-uses of the same module don't
    // cause reinitialization. It's up to the caller to ensure they preserve copies
    // of the module though.
    context.close(RETURN).unlock_module();

    // The current module's parameters are stored into register 1 such that when a function
    // is called, its parameters can be located. Through this public interface that
    // invariant is upheld.
    let mut symbol_list = vec![];
    for def in module.definitions() {
        if def.is_exported {
            let next_export = context.make_label("next_export");
            let name = def
                .name()
                .unwrap()
                .name()
                .expect("ids of definitions have names");
            context
                .instruction(Instruction::LoadLocal(current_module + 1))
                .atom(name)
                .instruction(Instruction::ValEq)
                .cond_jump(&next_export)
                .instruction(Instruction::Pop);
            symbol_list.push(Value::from(context.make_atom(name)));

            match &def.item {
                ir::DefinitionItem::Constant(constant) => {
                    let label = context
                        .scope
                        .lookup_static(&constant.name.id)
                        .unwrap()
                        .clone()
                        .unwrap_context();
                    // Constants are stored in the modules context, load them like context parameters.
                    context
                        .instruction(Instruction::LoadRegister(MODULE))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(MODULE))
                        .reference(label)
                        .instruction(Instruction::Call(0))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(MODULE))
                        .instruction(Instruction::Return);
                }
                ir::DefinitionItem::Function(func) => {
                    let label = context
                        .scope
                        .lookup_static(&func.name.id)
                        .unwrap()
                        .clone()
                        .unwrap_context();
                    context
                        .instruction(Instruction::LoadRegister(MODULE))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(MODULE))
                        .reference(label)
                        .instruction(Instruction::Call(0))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(MODULE))
                        .instruction(Instruction::Return);
                }
                ir::DefinitionItem::Procedure(proc) => {
                    let label = context
                        .scope
                        .lookup_static(&proc.name.id)
                        .unwrap()
                        .clone()
                        .unwrap_context();
                    context
                        .instruction(Instruction::LoadRegister(MODULE))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(MODULE))
                        .reference(label)
                        .instruction(Instruction::Call(0))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(MODULE))
                        .instruction(Instruction::Return);
                }
                ir::DefinitionItem::Rule(rule) => {
                    let label = context
                        .scope
                        .lookup_static(&rule.name.id)
                        .unwrap()
                        .clone()
                        .unwrap_context();
                    context
                        .instruction(Instruction::LoadRegister(MODULE))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(MODULE))
                        .reference(label)
                        .instruction(Instruction::Call(0))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(MODULE))
                        .instruction(Instruction::Return);
                }

                ir::DefinitionItem::Module(submodule) => {
                    let label = context
                        .scope
                        .lookup_static(&submodule.name.id)
                        .unwrap()
                        .clone()
                        .unwrap_context();
                    let submodule_arity = submodule.module.as_module().unwrap().parameters.len();
                    // The partially constructed module is already in the current module's context,
                    // so we just do basically the same as accessing a constant or parameter.
                    context
                        .instruction(Instruction::LoadRegister(MODULE))
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(MODULE))
                        .reference(label)
                        .instruction(Instruction::Call(0))
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::SetRegister(MODULE));
                    let mut partial_module = context.scope.intermediate();
                    // Capture all the parameters and apply them one by one, as they arrive.
                    // Each call needs to keep the partially applied module closure around,
                    // as each should be independent.
                    for _ in 0..submodule_arity {
                        context.close(RETURN).unlock_function();
                        let parameter = context.scope.intermediate();
                        context.instruction(Instruction::LoadRegister(MODULE));
                        context.scope.intermediate(); // previous module
                        context
                            .instruction(Instruction::LoadLocal(current_module))
                            .instruction(Instruction::SetRegister(MODULE))
                            .instruction(Instruction::LoadLocal(partial_module))
                            .instruction(Instruction::LoadLocal(parameter))
                            .call_function()
                            .instruction(Instruction::SetLocal(parameter))
                            .instruction(Instruction::SetRegister(MODULE));
                        context.scope.end_intermediate(); // previous module
                        partial_module = parameter;
                    }
                    // Then one more "parameter" is the symbol to import. The call is basically the same.
                    context.close(RETURN).unlock_module();
                    let parameter = context.scope.intermediate();
                    context.instruction(Instruction::LoadRegister(MODULE));
                    context.scope.intermediate(); // previous module
                    context
                        .instruction(Instruction::LoadLocal(current_module))
                        .instruction(Instruction::SetRegister(MODULE))
                        .instruction(Instruction::LoadLocal(partial_module))
                        .instruction(Instruction::LoadLocal(parameter))
                        .call_module()
                        .instruction(Instruction::SetLocal(parameter))
                        .instruction(Instruction::SetRegister(MODULE));
                    context.scope.end_intermediate(); // previous module

                    // After every parameter was passed, the top of stack is the resolved imported symbol,
                    // so it just needs to be returned.
                    context.instruction(Instruction::Return);
                    // Undo all the scope changes finally. It could be done more efficiently sure,
                    // but this way is clear.
                    for _ in 0..submodule_arity + 1 {
                        context.scope.end_intermediate();
                    }
                    context.scope.end_intermediate();
                }
                ir::DefinitionItem::Test(..) => unreachable!("tests cannot be exported"),
            }
            context.label(next_export);
        } else if let Some(test) = def.item.as_test() {
            // As the single special case, if the current definition is the test that is being
            // compiled to a program, force it into the exports.
            if mode == Mode::Test(&[], &test.name) {
                let next_export = context.make_label("next_export");

                context
                    .instruction(Instruction::LoadLocal(current_module + 1))
                    .atom("trilogy:__testentry__")
                    .instruction(Instruction::ValEq)
                    .cond_jump(&next_export)
                    .instruction(Instruction::Pop);
                let proc_label = "trilogy:__testentry__";
                context
                    .close(RETURN)
                    .unlock_procedure(0)
                    .instruction(Instruction::LoadRegister(MODULE))
                    .instruction(Instruction::LoadLocal(current_module))
                    .instruction(Instruction::SetRegister(MODULE))
                    .reference(proc_label)
                    .call_procedure(0);
                context
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::SetRegister(MODULE))
                    .instruction(Instruction::Return);
                context.label(next_export);
            }
        }
    }

    context
        .constant(symbol_list)
        .instruction(Instruction::Cons)
        .atom("UnresolvedImport")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);
    context.scope.end_intermediate();

    // 6. Defining accessors for context values
    //
    // For definitions to actually access the module parameters and constants, they're defined as
    // 0-arity functions that are aware of the module convention.
    for (i, var) in variables {
        // NOTE: a submodule thinks it is in its own context, because by the time the child's
        // initialization code is generated, we've already generated the parent's initialization
        // code, therefore recording the module in the static context. Meanwhile, when we actually
        // go to run that initialized code, the module (and its parents) are not actually in the
        // context because they aren't quite initialized yet.
        //
        // A tricky little situation.
        let label = context
            .scope
            .lookup_static(&var)
            .unwrap()
            .clone()
            .unwrap_context();
        context.label(&label);
        context
            .instruction(Instruction::LoadRegister(MODULE))
            .constant(base + i)
            .instruction(Instruction::Access)
            .instruction(Instruction::Return);
    }
}
