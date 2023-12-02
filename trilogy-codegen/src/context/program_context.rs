use super::{Labeler, Scope, StaticMember};
use crate::prelude::*;
use crate::{delegate_label_maker, module::Mode};
use std::collections::HashMap;
use trilogy_ir::{ir, Id};
use trilogy_vm::{delegate_chunk_writer, ChunkBuilder, ChunkWriter, Instruction, Value};

pub(crate) struct ProgramContext<'a> {
    labeler: Labeler,
    builder: &'a mut ChunkBuilder,
}

impl<'a> ProgramContext<'a> {
    pub fn new(builder: &'a mut ChunkBuilder) -> Self {
        Self {
            builder,
            labeler: Labeler::new(),
        }
    }
}

delegate_chunk_writer!(ProgramContext<'_>, builder);
delegate_label_maker!(ProgramContext<'_>, labeler);

impl ProgramContext<'_> {
    pub fn entrypoint(&mut self) -> &mut Self {
        self.builder.entrypoint();
        self
    }

    /// Writes the entrypoint of the program.
    pub fn write_main(&mut self, path: &[&str], default_exit: Value) {
        self.entrypoint()
            .label("trilogy:__entrypoint__")
            .reference("trilogy:__entrymodule__")
            .instruction(Instruction::Call(0));

        for seg in path {
            self.atom(seg).call_module();
        }
        self.call_procedure(0)
            .instruction(Instruction::Copy)
            .constant(())
            .instruction(Instruction::ValEq)
            .cond_jump("trilogy:__exit_runoff__")
            .constant(default_exit)
            .label("trilogy:__exit_runoff__")
            .instruction(Instruction::Exit);
    }

    /// Writes a procedure.
    ///
    /// The calling convention of procedures is to call with all arguments on the stack
    /// in order, followed by the struct `'procedure(arity)` where the arity is the number
    /// of arguments that was just passed, so as to prevent the invalid calling of procedures.
    pub fn write_procedure(
        &mut self,
        statics: &mut HashMap<Id, StaticMember>,
        procedure: &ir::ProcedureDefinition,
    ) {
        let for_id = self.labeler.for_id(&procedure.name.id);
        self.label(for_id);
        assert!(procedure.overloads.len() == 1);
        let overload = &procedure.overloads[0];
        let mut context = self.begin(statics, overload.parameters.len());
        context.unlock_procedure(overload.parameters.len());
        write_procedure(context, overload);
    }

    /// Writes a test.
    ///
    /// A test is written basically the same as a procedure with 0 arguments. It will be
    /// called as if it was `proc main!()`.
    pub fn write_test(
        &mut self,
        statics: &mut HashMap<Id, StaticMember>,
        test: &ir::TestDefinition,
    ) {
        self.label("trilogy:__testentry__");
        let mut context = self.begin(statics, 0);
        context.unlock_procedure(0).evaluate(&test.body);
        context
            .instruction(Instruction::Const(Value::Unit))
            .instruction(Instruction::Return);
    }

    /// Writes a rule.
    ///
    /// The calling convention of rules is to call once with no arguments to perform setup,
    /// then call the returned closure with the input arguments. "Incomplete" parameters
    /// (entirely or partially unbound) should be passed as empty cells (created via the
    /// `VAR` instruction) so that they appear unbound within the rule body.
    ///
    /// The return value is much like an iterator, a 0 arity callable that returns either
    /// `'next(V)` or `'done`, where `V` will be a list of resulting bindings in argument
    /// order which are to be pattern matched against the input patterns.
    pub fn write_rule(
        &mut self,
        statics: &mut HashMap<Id, StaticMember>,
        rule: &ir::RuleDefinition,
    ) {
        let for_id = self.labeler.for_id(&rule.name.id);
        self.label(for_id);
        let arity = rule.overloads[0].parameters.len();
        let mut context = self.begin(statics, 0);
        context.constant(((), 0));
        context.scope.intermediate(); // TODO: do we need to know the index of this (it's 0)?
        context.close(RETURN).unlock_rule(arity);
        context.scope.closure(arity); // TODO: do we need to know the index of these (1...n)?
        context.close(RETURN);

        context
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Uncons);
        for (i, overload) in rule.overloads.iter().enumerate() {
            let skip = context.make_label("next_overload");
            let fail = context.make_label("fail");
            context
                .instruction(Instruction::Copy)
                .constant(i)
                .instruction(Instruction::ValEq)
                .cond_jump(&skip)
                .instruction(Instruction::Pop);
            write_rule(&mut context, overload, &fail);
            context
                .instruction(Instruction::Swap)
                .constant(i)
                .instruction(Instruction::Cons)
                .instruction(Instruction::SetLocal(0))
                .instruction(Instruction::Return);
            context
                .label(fail)
                // The 'done and the state are discarded by write_rule, so here we just
                // have to create the next state
                .constant(())
                .constant(i + 1)
                .label(skip);
        }
        context
            .instruction(Instruction::Cons)
            .instruction(Instruction::SetLocal(0))
            .atom("done")
            .instruction(Instruction::Return);
    }

    /// Writes a function. Calling convention of functions is to pass one arguments at a time
    /// via repeated `CALL 1`. Each application should be two values, the first being the actual
    /// argument, and the second the struct `'function(1)` to ensure functions are not called
    /// improperly.
    ///
    /// Eventually the returned value will be the final value of the function.
    pub fn write_function(
        &mut self,
        statics: &mut HashMap<Id, StaticMember>,
        function: &ir::FunctionDefinition,
    ) {
        let for_id = self.labeler.for_id(&function.name.id);
        self.label(for_id);
        let arity = function.overloads[0].parameters.len();
        let mut context = self.begin(statics, arity);
        context.unlock_function();
        for _ in 1..arity {
            context.close(RETURN).unlock_function();
        }
        for overload in &function.overloads {
            write_function(&mut context, overload);
        }
        context
            .atom("NoMatchingFunctionOverload")
            .instruction(Instruction::Panic);
    }

    /// Writes a module. Modules are prefixed with a single prelude function, which takes
    /// the module's parameters and returns a module object that can be used to access the
    /// public members. If there are no parameters, the prelude function is the module object
    /// already.
    ///
    /// The module object is a callable that takes one argument, an atom that is the identifier
    /// of the member to access, and returns that member bound to the module's context arguments.
    ///
    /// Modules have a three-phase calling convention, depending on what stage the module is in.
    /// 1. Initially, a module must be called with no arguments to trigger "initialization"
    /// 2. If there are parameters, each parameter is passed to the module as if it was a function
    /// 3. To finally import a member from a module, the atom representing the name of that member is passed, along with the struct `'module(1)`
    pub fn write_module(
        &mut self,
        mut statics: HashMap<Id, StaticMember>,
        def: &ir::ModuleDefinition,
        mode: Mode,
    ) {
        let for_id = self.labeler.for_id(&def.name.id);
        self.label(for_id);
        let module = def.module.as_module().unwrap();
        self.collect_static(module, &mut statics);
        let mut context = self.begin(&mut statics, module.parameters.len());
        write_module_prelude(&mut context, module, mode);
        write_module_definitions(self, module, &mut statics, mode);
    }

    pub fn collect_static(&self, module: &ir::Module, statics: &mut HashMap<Id, StaticMember>) {
        statics.extend(module.definitions().iter().filter_map(|def| {
            let id = match &def.item {
                ir::DefinitionItem::Constant(def) => Some(def.name.id.clone()),
                ir::DefinitionItem::Function(func) => Some(func.name.id.clone()),
                ir::DefinitionItem::Rule(rule) => Some(rule.name.id.clone()),
                ir::DefinitionItem::Procedure(proc) => Some(proc.name.id.clone()),
                ir::DefinitionItem::Module(module) if module.module.as_module().is_some() => {
                    Some(module.name.id.clone())
                }
                ir::DefinitionItem::Module(module) => match &*module.module {
                    ir::ModuleCell::External(path) => {
                        return Some((module.name.id.clone(), StaticMember::Chunk(path.to_owned())))
                    }
                    ir::ModuleCell::Module(..) => Some(module.name.id.clone()),
                },
                ir::DefinitionItem::Test(..) => None,
            }?;
            let label = self.labeler.for_id(&id);
            Some((id, StaticMember::Label(label)))
        }));
    }

    pub fn begin<'a>(
        &'a mut self,
        statics: &'a mut HashMap<Id, StaticMember>,
        parameters: usize,
    ) -> Context<'a> {
        let scope = Scope::new(statics, parameters);
        Context::new(self.builder, &mut self.labeler, scope)
    }
}
