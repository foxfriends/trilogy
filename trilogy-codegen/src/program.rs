use crate::context::{Labeler, Scope};
use crate::preamble::RETURN;
use crate::prelude::*;
use std::collections::HashMap;
use trilogy_ir::{ir, Id};
use trilogy_vm::{Atom, Instruction, OpCode, ProgramBuilder};

pub(crate) struct ProgramContext<'a> {
    pub labeler: Labeler,
    builder: &'a mut ProgramBuilder,
}

impl<'a> ProgramContext<'a> {
    fn new(builder: &'a mut ProgramBuilder, location: String) -> Self {
        Self {
            builder,
            labeler: Labeler::new(location),
        }
    }
}

pub fn write_program(builder: &mut ProgramBuilder, module: &ir::Module) {
    let mut context = ProgramContext::new(builder, module.location().to_owned());
    write_preamble(&mut context);
    write_module(&mut context, module, true);
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

    pub fn write_procedure(
        &mut self,
        statics: &HashMap<Id, String>,
        procedure: &ir::ProcedureDefinition,
    ) {
        let beginning = self.labeler.begin(&procedure.name);
        self.write_label(beginning);
        let for_id = self.labeler.for_id(&procedure.name.id);
        self.write_label(for_id);
        assert!(procedure.overloads.len() == 1);
        let overload = &procedure.overloads[0];
        let context = self.begin(statics, overload.parameters.len());
        write_procedure(context, overload);
    }

    pub fn write_rule(&mut self, statics: &HashMap<Id, String>, rule: &ir::RuleDefinition) {
        let beginning = self.labeler.begin(&rule.name);
        self.write_label(beginning);
        let for_id = self.labeler.for_id(&rule.name.id);
        self.write_label(for_id);
        let arity = rule.overloads[0].parameters.len();
        let mut context = self.begin(statics, arity);
        for _ in 1..arity {
            context.close(RETURN);
        }
        for overload in &rule.overloads {
            write_rule(&mut context, overload);
        }
        context
            .write_instruction(Instruction::Const(().into()))
            .write_instruction(Instruction::Return);
    }

    pub fn write_function(
        &mut self,
        statics: &HashMap<Id, String>,
        function: &ir::FunctionDefinition,
    ) {
        let beginning = self.labeler.begin(&function.name);
        self.write_label(beginning);
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

    fn begin<'a>(&'a mut self, statics: &'a HashMap<Id, String>, parameters: usize) -> Context<'a> {
        let scope = Scope::new(statics, parameters);
        Context::new(self.builder, &mut self.labeler, scope)
    }
}
