use crate::prelude::StackTracker;

use super::StaticMember;
use std::collections::HashMap;
use trilogy_ir::Id;
use trilogy_vm::{Instruction, Offset};

#[derive(Clone, Debug)]
pub(crate) enum Binding<'a> {
    Variable(Offset),
    Context(&'a str),
    Static(&'a str),
}

impl Binding<'_> {
    pub fn unwrap_local(&self) -> Offset {
        match self {
            Self::Variable(index) => *index,
            _ => panic!("attemped to unwrap local, but {self:?} is not a local"),
        }
    }
}

impl<'a> From<&'a StaticMember> for Binding<'a> {
    fn from(value: &'a StaticMember) -> Self {
        match &value {
            StaticMember::Label(label) => Binding::Static(label),
            StaticMember::Context(label) => Binding::Context(label),
            StaticMember::Chunk(..) => {
                panic!("chunks should be evaluated ahead of time and converted to context")
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Scope<'a> {
    statics: &'a mut HashMap<Id, StaticMember>,
    locals: HashMap<Id, Offset>,
    parameters: usize,

    kw_resume: Vec<Offset>,
    kw_cancel: Vec<Offset>,
    kw_break: Vec<Offset>,
    kw_continue: Vec<Offset>,
}

impl<'a> Scope<'a> {
    pub fn new(statics: &'a mut HashMap<Id, StaticMember>, parameters: usize) -> Self {
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

    pub fn remember_variable(&mut self, id: Id, offset: Offset) {
        self.locals.insert(id, offset);
    }

    pub fn declare_variable(&mut self, id: Id) -> bool {
        if self.locals.contains_key(&id) {
            return false;
        }
        self.remember_variable(id, (self.parameters + self.locals.len()) as Offset);
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

    pub fn declare_static(&mut self, id: Id, static_member: StaticMember) -> Option<StaticMember> {
        self.statics.insert(id, static_member)
    }

    pub fn closure(&mut self, parameters: usize) -> Offset {
        let offset = self.parameters + self.locals.len();
        self.parameters += parameters;
        offset as Offset
    }

    pub fn unclosure(&mut self, parameters: usize) {
        self.parameters -= parameters;
    }

    pub fn kw_break(&self) -> Option<Instruction> {
        let offset = self.kw_break.last()?;
        Some(Instruction::LoadLocal(*offset))
    }

    pub fn kw_continue(&self) -> Option<Instruction> {
        let offset = self.kw_continue.last()?;
        Some(Instruction::LoadLocal(*offset))
    }

    pub fn kw_cancel(&self) -> Option<Instruction> {
        let offset = self.kw_cancel.last()?;
        Some(Instruction::LoadLocal(*offset))
    }

    pub fn kw_resume(&self) -> Option<Instruction> {
        let offset = self.kw_resume.last()?;
        Some(Instruction::LoadLocal(*offset))
    }

    pub fn context_size(&self) -> usize {
        self.statics
            .values()
            .filter(|x| matches!(x, StaticMember::Context(..)))
            .count()
    }
}

impl StackTracker for Scope<'_> {
    fn intermediate(&mut self) -> Offset {
        self.closure(1)
    }

    fn end_intermediate(&mut self) -> &mut Self {
        self.unclosure(1);
        self
    }

    fn push_resume(&mut self, offset: Offset) -> &mut Self {
        self.kw_resume.push(offset);
        self
    }

    fn pop_resume(&mut self) -> &mut Self {
        self.kw_resume.pop();
        self
    }

    fn push_cancel(&mut self, offset: Offset) -> &mut Self {
        self.kw_cancel.push(offset);
        self
    }

    fn pop_cancel(&mut self) -> &mut Self {
        self.kw_cancel.pop();
        self
    }

    fn push_break(&mut self, offset: Offset) -> &mut Self {
        self.kw_break.push(offset);
        self
    }

    fn pop_break(&mut self) -> &mut Self {
        self.kw_break.pop();
        self
    }

    fn push_continue(&mut self, offset: Offset) -> &mut Self {
        self.kw_continue.push(offset);
        self
    }

    fn pop_continue(&mut self) -> &mut Self {
        self.kw_continue.pop();
        self
    }
}
