use crate::entrypoint::StaticMember;
use std::collections::HashMap;
use trilogy_ir::Id;
use trilogy_vm::Instruction;

#[derive(Clone, Debug)]
pub(crate) enum Binding<'a> {
    Variable(u32),
    Static(&'a str),
    Chunk(&'a str),
}

impl<'a> From<&'a StaticMember> for Binding<'a> {
    fn from(value: &'a StaticMember) -> Self {
        match &value {
            StaticMember::Label(label) => Binding::Static(label),
            StaticMember::Chunk(chunk) => Binding::Chunk(chunk),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Scope<'a> {
    statics: &'a HashMap<Id, StaticMember>,
    locals: HashMap<Id, u32>,
    parameters: usize,

    kw_resume: Vec<u32>,
    kw_cancel: Vec<u32>,
    kw_break: Vec<u32>,
    kw_continue: Vec<u32>,
}

impl<'a> Scope<'a> {
    pub fn new(statics: &'a HashMap<Id, StaticMember>, parameters: usize) -> Self {
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
        self.locals
            .insert(id, (self.parameters + self.locals.len()) as u32);
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
            .or_else(|| self.lookup_static(id).map(Into::into))
    }

    pub fn lookup_static(&self, id: &Id) -> Option<&'_ StaticMember> {
        self.statics.get(id)
    }

    pub fn closure(&mut self, parameters: usize) -> u32 {
        let offset = self.parameters + self.locals.len();
        self.parameters += parameters;
        offset as u32
    }

    pub fn unclosure(&mut self, parameters: usize) {
        self.parameters -= parameters;
    }

    pub fn intermediate(&mut self) -> u32 {
        self.parameters += 1;
        (self.parameters + self.locals.len() - 1) as u32
    }

    pub fn end_intermediate(&mut self) {
        self.parameters -= 1;
    }

    pub fn push_break(&mut self) -> u32 {
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

    pub fn push_continue(&mut self) -> u32 {
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

    pub fn push_cancel(&mut self) -> u32 {
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

    pub fn push_resume(&mut self) -> u32 {
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
