use crate::context::{Labeler, Scope};
use crate::module::Mode;
use crate::preamble::RETURN;
use crate::prelude::*;
use std::collections::HashMap;
use trilogy_ir::{ir, Id};
use trilogy_vm::{Atom, ChunkBuilder, Instruction};

#[derive(Clone, Debug)]
pub(crate) enum StaticMember {
    Chunk(String),
    Context(String),
    Label(String),
}

impl StaticMember {
    pub fn unwrap_label(self) -> String {
        match self {
            Self::Label(label) => label,
            _ => panic!("expected static member to be a label, but it was {self:?}"),
        }
    }
}

pub(crate) struct ProgramContext<'a> {
    pub labeler: Labeler,
    builder: &'a mut ChunkBuilder,
}

impl<'a> ProgramContext<'a> {
    fn new(builder: &'a mut ChunkBuilder) -> Self {
        Self {
            builder,
            labeler: Labeler::new(),
        }
    }
}

pub fn write_program(builder: &mut ChunkBuilder, module: &ir::Module) {
    let mut context = ProgramContext::new(builder);
    context.write_main();
    write_preamble(&mut context);

    let mut statics = HashMap::default();
    context.collect_static(module, &mut statics);

    // Parameters len will be 0, but let's write it out anyway
    context.label("trilogy:__entrymodule__");
    let mut precontext = context.begin(&statics, module.parameters.len() + 1);
    statics.extend(write_module_prelude(
        &mut precontext,
        module,
        Mode::Document,
    ));
    write_module_definitions(&mut context, module, &statics);
}

pub fn write_module(builder: &mut ChunkBuilder, module: &ir::Module) {
    let mut context = ProgramContext::new(builder);
    let mut statics = HashMap::default();
    context.collect_static(module, &mut statics);
    context.entrypoint();
    // Parameters len will be 0, but let's write it out anyway
    let mut precontext = context.begin(&statics, module.parameters.len() + 1);
    statics.extend(write_module_prelude(
        &mut precontext,
        module,
        Mode::Document,
    ));
    write_module_definitions(&mut context, module, &statics);
}

impl ProgramContext<'_> {
    pub fn shift(&mut self, label: &str) -> &mut Self {
        self.builder.shift(label);
        self
    }

    pub fn close(&mut self, label: &str) -> &mut Self {
        self.builder.close(label);
        self
    }

    pub fn cond_jump(&mut self, label: &str) -> &mut Self {
        self.builder.cond_jump(label);
        self
    }

    pub fn instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.builder.instruction(instruction);
        self
    }

    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.builder.label(label);
        self
    }

    pub fn atom(&self, value: &str) -> Atom {
        self.builder.atom(value)
    }

    pub fn entrypoint(&mut self) -> &mut Self {
        self.builder.entrypoint();
        self
    }

    /// Writes the entrypoint of the program.
    pub fn write_main(&mut self) {
        let main = self.builder.atom("main");
        self.builder
            .entrypoint()
            .label("trilogy:__entrypoint__")
            .reference("trilogy:__entrymodule__")
            .instruction(Instruction::Const(main.into()))
            .instruction(Instruction::Call(1))
            .instruction(Instruction::Call(0))
            .instruction(Instruction::Copy)
            .instruction(Instruction::Const(().into()))
            .instruction(Instruction::ValEq)
            .cond_jump("trilogy:__exit_runoff__")
            .instruction(Instruction::Const(0.into()))
            .label("trilogy:__exit_runoff__")
            .instruction(Instruction::Exit);
    }

    /// Writes a procedure.
    ///
    /// The calling convention of procedures is to call with all arguments on the stack in order.
    pub fn write_procedure(
        &mut self,
        statics: &HashMap<Id, StaticMember>,
        procedure: &ir::ProcedureDefinition,
    ) {
        let for_id = self.labeler.for_id(&procedure.name.id);
        self.label(for_id);
        assert!(procedure.overloads.len() == 1);
        let overload = &procedure.overloads[0];
        let context = self.begin(statics, overload.parameters.len());
        write_procedure(context, overload);
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
    pub fn write_rule(&mut self, statics: &HashMap<Id, StaticMember>, rule: &ir::RuleDefinition) {
        let for_id = self.labeler.for_id(&rule.name.id);
        self.label(for_id);
        let arity = rule.overloads[0].parameters.len();
        let mut context = self.begin(statics, 0);
        context.instruction(Instruction::Const(((), 0).into()));
        context.scope.intermediate(); // TODO: do we need to know the index of this (it's 0)?
        context.close(RETURN);
        context.scope.closure(arity); // TODO: do we need to know the index of these (1...n)?
        context.close(RETURN);

        context
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Uncons);
        for (i, overload) in rule.overloads.iter().enumerate() {
            let skip = context.labeler.unique_hint("next_overload");
            let fail = context.labeler.unique_hint("fail");
            context
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(i.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&skip)
                .instruction(Instruction::Pop);
            write_rule(&mut context, overload, &fail);
            context
                .instruction(Instruction::Swap)
                .instruction(Instruction::Const(i.into()))
                .instruction(Instruction::Cons)
                .instruction(Instruction::SetLocal(0))
                .instruction(Instruction::Return);
            context
                .label(fail)
                // The 'done and the state are discarded by write_rule, so here we just
                // have to create the next state
                .instruction(Instruction::Const(().into()))
                .instruction(Instruction::Const((i + 1).into()))
                .label(skip);
        }
        let done = context.atom("done");
        context
            .instruction(Instruction::Cons)
            .instruction(Instruction::SetLocal(0))
            .instruction(Instruction::Const(done.into()))
            .instruction(Instruction::Return);
    }

    /// Writes a function. Calling convention of functions is to pass one arguments at a time
    /// via repeated `CALL 1`. Eventually the returned value will be the final value of the
    /// function.
    pub fn write_function(
        &mut self,
        statics: &HashMap<Id, StaticMember>,
        function: &ir::FunctionDefinition,
    ) {
        let for_id = self.labeler.for_id(&function.name.id);
        self.label(for_id);
        let arity = function.overloads[0].parameters.len();
        let mut context = self.begin(statics, arity);
        for _ in 1..arity {
            context.close(RETURN);
        }
        for overload in &function.overloads {
            write_function(&mut context, overload);
        }
        context.instruction(Instruction::Fizzle);
    }

    /// Writes a module. Modules are prefixed with a single prelude function, which takes
    /// the module's parameters and returns a module object that can be used to access the
    /// public members. If there are no parameters, the prelude function is the module object
    /// already.
    ///
    /// The module object is a callable that takes one argument, an atom that is the identifier
    /// of the member to access, and returns that member bound to the module's context arguments.
    pub fn write_module(
        &mut self,
        mut statics: HashMap<Id, StaticMember>,
        def: &ir::ModuleDefinition,
    ) {
        let for_id = self.labeler.for_id(&def.name.id);
        self.label(for_id);
        let module = def.module.as_module().unwrap();
        self.collect_static(module, &mut statics);
        let mut context = self.begin(&statics, 1 + module.parameters.len());
        statics.extend(write_module_prelude(&mut context, module, Mode::Module));
        write_module_definitions(self, module, &statics);
    }

    fn collect_static(&self, module: &ir::Module, statics: &mut HashMap<Id, StaticMember>) {
        statics.extend(module.definitions().iter().filter_map(|def| {
            let id = match &def.item {
                ir::DefinitionItem::Constant(..) => None,
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

    fn begin<'a>(
        &'a mut self,
        statics: &'a HashMap<Id, StaticMember>,
        parameters: usize,
    ) -> Context<'a> {
        let scope = Scope::new(statics, parameters);
        Context::new(self.builder, &mut self.labeler, scope)
    }
}
