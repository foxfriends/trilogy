use crate::context::{Labeler, Scope};
use crate::preamble::RETURN;
use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use trilogy_ir::{ir, Id};
use trilogy_vm::{Atom, Instruction, OpCode, ProgramBuilder};

pub(crate) struct ProgramContext<'a> {
    pub labeler: Labeler,
    modules_written: HashSet<*const ir::ModuleCell>,
    builder: &'a mut ProgramBuilder,
}

impl<'a> ProgramContext<'a> {
    fn new(builder: &'a mut ProgramBuilder) -> Self {
        Self {
            builder,
            modules_written: HashSet::new(),
            labeler: Labeler::new(),
        }
    }
}

pub fn write_program(builder: &mut ProgramBuilder, module: &ir::Module) {
    let mut context = ProgramContext::new(builder);
    write_preamble(&mut context);
    write_module(&mut context, module, None, true);
}

impl ProgramContext<'_> {
    pub fn shift(&mut self, label: &str) -> &mut Self {
        self.builder
            .write_opcode(OpCode::Shift)
            .write_offset_label(label.to_owned());
        self
    }

    pub fn close(&mut self, label: &str) -> &mut Self {
        self.builder
            .write_opcode(OpCode::Close)
            .write_offset_label(label.to_owned());
        self
    }

    pub fn cond_jump(&mut self, label: &str) -> &mut Self {
        self.builder
            .write_opcode(OpCode::CondJump)
            .write_offset_label(label.to_owned());
        self
    }

    pub fn write_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.builder.write_instruction(instruction);
        self
    }

    pub fn write_label(&mut self, label: String) -> &mut Self {
        self.builder
            .write_label(label)
            .expect("should not insert same label twice");
        self
    }

    pub fn atom(&mut self, value: &str) -> Atom {
        self.builder.atom(value)
    }

    /// Writes a procedure. Calling convention of procedures is to simply call with all arguments
    /// on the stack in order.
    pub fn write_procedure(
        &mut self,
        statics: &HashMap<Id, String>,
        procedure: &ir::ProcedureDefinition,
    ) {
        let for_id = self.labeler.for_id(&procedure.name.id);
        self.write_label(for_id);
        assert!(procedure.overloads.len() == 1);
        let overload = &procedure.overloads[0];
        let context = self.begin(statics, overload.parameters.len());
        write_procedure(context, overload);
    }

    /// Writes a rule. Calling convention of rules is to call once with no arguments to
    /// perform setup, then call the returned closure with the input arguments prefixed with one
    /// extra argument which is a Set listing the indexes of the parameters which are to
    /// be treated as "fixed" for the purpose of backtracking. "Incomplete" parameters (entirely
    /// or partially unbound) may be passed with any value as they will be treated as unset.
    ///
    /// The return value is much like an iterator, either 'next(V) or 'done, where V will be a
    /// tuple of resulting bindings in argument order which are to be pattern matched against
    /// the input patterns.
    pub fn write_rule(&mut self, statics: &HashMap<Id, String>, rule: &ir::RuleDefinition) {
        let for_id = self.labeler.for_id(&rule.name.id);
        self.write_label(for_id);
        let arity = rule.overloads[0].parameters.len();
        let mut context = self.begin(statics, 0);
        context.write_instruction(Instruction::Const(((), 0).into()));
        context.scope.intermediate(); // TODO: do we need to know the index of this (it's 0)?
        context.close(RETURN);
        context.scope.closure(arity + 1); // TODO: do we need to know the index of these (1 + n)?
        context.close(RETURN);

        context
            .write_instruction(Instruction::LoadLocal(0))
            .write_instruction(Instruction::Uncons);
        for (i, overload) in rule.overloads.iter().enumerate() {
            let skip = context.labeler.unique_hint("next_overload");
            let fail = context.labeler.unique_hint("fail");
            context
                .write_instruction(Instruction::Copy)
                .write_instruction(Instruction::Const(i.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(&skip)
                .write_instruction(Instruction::Pop);
            write_rule(&mut context, overload, &fail);
            context
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Const(i.into()))
                .write_instruction(Instruction::Cons)
                .write_instruction(Instruction::SetLocal(0))
                .write_instruction(Instruction::Return);
            context
                .write_label(fail)
                .write_instruction(Instruction::Const(((), i + 1).into()))
                .write_instruction(Instruction::SetLocal(0))
                .write_instruction(Instruction::LoadLocal(0))
                .write_instruction(Instruction::Uncons)
                .write_label(skip);
        }
        let done = context.atom("done");
        context
            .write_instruction(Instruction::Const(done.into()))
            .write_instruction(Instruction::Return);
    }

    /// Writes a function. Calling convention of functions is to pass one arguments at a time
    /// via repeated `CALL 1`. Eventually the returned value will be the final value of the
    /// function.
    pub fn write_function(
        &mut self,
        statics: &HashMap<Id, String>,
        function: &ir::FunctionDefinition,
    ) {
        let for_id = self.labeler.for_id(&function.name.id);
        self.write_label(for_id);
        let arity = function.overloads[0].parameters.len();
        let mut context = self.begin(statics, arity);
        for _ in 1..arity {
            context.close(RETURN);
        }
        for overload in &function.overloads {
            write_function(&mut context, overload);
        }
        context.write_instruction(Instruction::Fizzle);
    }

    pub fn write_module(
        &mut self,
        statics: Option<&HashMap<Id, String>>,
        def: &ir::ModuleDefinition,
    ) {
        let ptr = Arc::as_ptr(&def.module);
        if self.modules_written.contains(&ptr) {
            return;
        }
        self.modules_written.insert(ptr);
        let module = def.module.as_module().unwrap();
        write_module(self, module, statics, false);
    }

    fn begin<'a>(&'a mut self, statics: &'a HashMap<Id, String>, parameters: usize) -> Context<'a> {
        let scope = Scope::new(statics, parameters);
        Context::new(self.builder, &mut self.labeler, scope)
    }
}
