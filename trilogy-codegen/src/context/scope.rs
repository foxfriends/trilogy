use std::collections::HashMap;
use trilogy_ir::Id;
use trilogy_vm::Instruction;

#[derive(Clone, Debug)]
pub(crate) enum Binding<'a> {
    Variable(usize),
    Static(&'a str),
}

#[derive(Clone, Debug)]
pub(crate) struct Scope<'a> {
    statics: &'a HashMap<Id, String>,
    locals: HashMap<Id, usize>,
    parameters: usize,

    kw_resume: Vec<usize>,
    kw_cancel: Vec<usize>,
    kw_break: Vec<usize>,
    kw_continue: Vec<usize>,
}

impl<'a> Scope<'a> {
    pub fn new(statics: &'a HashMap<Id, String>, parameters: usize) -> Self {
        Self {
            parameters,
            statics,
            locals: HashMap::default(),
            kw_resume: vec![],
            kw_cancel: vec![],
            kw_break: vec![],
            kw_continue: vec![],
        }
    }

    pub fn declare_variable(&mut self, id: Id) -> bool {
        if self.locals.contains_key(&id) {
            return false;
        }
        self.locals.insert(id, self.parameters + self.locals.len());
        true
    }

    pub fn undeclare_variable(&mut self, id: &Id) -> bool {
        self.locals.remove(id).is_some()
    }

    pub fn lookup(&self, id: &Id) -> Option<Binding<'_>> {
        self.locals
            .get(id)
            .copied()
            .map(Binding::Variable)
            .or_else(|| self.statics.get(id).map(|s| Binding::Static(s)))
    }

    pub fn closure(&mut self, parameters: usize) -> usize {
        let offset = self.parameters + self.locals.len();
        self.parameters += parameters;
        offset
    }

    pub fn unclosure(&mut self, parameters: usize) {
        self.parameters -= parameters;
    }

    pub fn intermediate(&mut self) -> usize {
        self.parameters += 1;
        self.parameters + self.locals.len() - 1
    }

    pub fn end_intermediate(&mut self) {
        self.parameters -= 1;
    }

    pub fn push_break(&mut self) -> usize {
        let offset = self.intermediate();
        self.kw_break.push(offset);
        offset
    }

    pub fn pop_break(&mut self) {
        self.end_intermediate();
        self.kw_break.pop();
    }

    pub fn kw_break(&self) -> Option<Instruction> {
        let offset = self.kw_break.last()?;
        Some(Instruction::LoadLocal(*offset))
    }

    pub fn push_continue(&mut self) -> usize {
        let offset = self.intermediate();
        self.kw_continue.push(offset);
        offset
    }

    pub fn pop_continue(&mut self) {
        self.end_intermediate();
        self.kw_continue.pop();
    }

    pub fn kw_continue(&self) -> Option<Instruction> {
        let offset = self.kw_continue.last()?;
        Some(Instruction::LoadLocal(*offset))
    }

    pub fn push_cancel(&mut self) -> usize {
        let offset = self.intermediate();
        self.kw_cancel.push(offset);
        offset
    }

    pub fn pop_cancel(&mut self) {
        self.end_intermediate();
        self.kw_cancel.pop();
    }

    pub fn kw_cancel(&self) -> Option<Instruction> {
        let offset = self.kw_cancel.last()?;
        Some(Instruction::LoadLocal(*offset))
    }

    pub fn push_resume(&mut self) -> usize {
        let offset = self.intermediate();
        self.kw_resume.push(offset);
        offset
    }

    pub fn pop_resume(&mut self) {
        self.end_intermediate();
        self.kw_resume.pop();
    }

    pub fn kw_resume(&self) -> Option<Instruction> {
        let offset = self.kw_resume.last()?;
        Some(Instruction::LoadLocal(*offset))
    }
}
