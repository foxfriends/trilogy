use trilogy_parser::syntax::Identifier;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ItemClass {
    Proc,
    Func,
    Rule,
    Module,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemKey {
    pub class: ItemClass,
    pub name: String,
    pub arity: usize,
}

impl ItemKey {
    pub(crate) fn new_module(identifier: &Identifier, arity: usize) -> Self {
        Self {
            class: ItemClass::Module,
            name: identifier.as_ref().to_owned(),
            arity,
        }
    }

    pub(crate) fn new_func(identifier: &Identifier, arity: usize) -> Self {
        Self {
            class: ItemClass::Func,
            name: identifier.as_ref().to_owned(),
            arity,
        }
    }

    pub(crate) fn new_rule(identifier: &Identifier, arity: usize) -> Self {
        Self {
            class: ItemClass::Rule,
            name: identifier.as_ref().to_owned(),
            arity,
        }
    }

    pub(crate) fn new_proc(identifier: &Identifier, arity: usize) -> Self {
        Self {
            class: ItemClass::Proc,
            name: identifier.as_ref().to_owned(),
            arity,
        }
    }
}
