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
