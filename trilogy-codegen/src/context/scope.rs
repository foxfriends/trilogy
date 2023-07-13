use std::collections::HashMap;
use trilogy_ir::Id;
use trilogy_vm::Value;

#[derive(Clone, Debug)]
pub(crate) enum Binding {
    Variable(usize),
    Constant(Value),
    Label(String),
}

impl Binding {
    pub fn is_variable(&self) -> bool {
        matches!(self, Binding::Variable(..))
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, Binding::Constant(..))
    }

    pub fn is_label(&self) -> bool {
        matches!(self, Binding::Label(..))
    }

    pub fn constant_value(&self) -> Option<&Value> {
        match &self {
            Self::Constant(value) => Some(value),
            _ => None,
        }
    }

    pub fn variable_offset(&self) -> Option<&usize> {
        match &self {
            Self::Variable(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_label(&self) -> Option<&str> {
        match &self {
            Self::Label(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Scope {
    identifiers: HashMap<Id, Binding>,
}

impl Scope {
    pub fn declare_variable(&mut self, id: Id, offset: usize) {
        self.identifiers.insert(id, Binding::Variable(offset));
    }

    pub fn declare_constant(&mut self, id: Id, value: Value) {
        self.identifiers.insert(id, Binding::Constant(value));
    }

    pub fn declare_label(&mut self, id: Id, label: String) {
        self.identifiers.insert(id, Binding::Label(label));
    }

    pub fn lookup(&self, id: &Id) -> Option<&Binding> {
        self.identifiers.get(id)
    }
}
