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

    kw_return: usize,
    #[allow(dead_code)]
    kw_resume: Option<usize>,
    #[allow(dead_code)]
    kw_cancel: Option<usize>,
    #[allow(dead_code)]
    kw_break: Option<usize>,
    #[allow(dead_code)]
    kw_continue: Option<usize>,
}

impl<'a> Scope<'a> {
    pub fn new(statics: &'a HashMap<Id, String>, parameters: usize) -> Self {
        Self {
            parameters,
            statics,
            locals: HashMap::default(),
            kw_return: 0,
            kw_resume: None,
            kw_cancel: None,
            kw_break: None,
            kw_continue: None,
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
        let offset = self.parameters + self.locals.len() + 1;
        self.kw_return += 1;
        self.parameters += parameters + 1;
        offset
    }

    pub fn unclosure(&mut self, parameters: usize) {
        self.kw_return -= 1;
        self.parameters -= parameters + 1;
    }

    pub fn kw_return(&self) -> Instruction {
        if self.kw_return == 0 {
            Instruction::Return
        } else {
            Instruction::Reset
        }
    }
}
