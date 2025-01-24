use inkwell::{
    basic_block::BasicBlock,
    values::{FunctionValue, PointerValue},
};
use std::collections::{HashMap, HashSet};
use trilogy_ir::Id;

#[derive(Clone, Copy)]
pub(crate) enum Variable<'ctx> {
    Closed(PointerValue<'ctx>),
    Owned(PointerValue<'ctx>),
}

impl<'ctx> Variable<'ctx> {
    pub(crate) fn ptr(&self) -> PointerValue<'ctx> {
        match self {
            Self::Closed(ptr) => *ptr,
            Self::Owned(ptr) => *ptr,
        }
    }
}

pub(crate) struct Scope<'ctx> {
    pub(crate) function: FunctionValue<'ctx>,
    pub(crate) variables: HashMap<Id, Variable<'ctx>>,
    pub(crate) parent_variables: HashSet<Id>,
    pub(crate) closure: Vec<Id>,
    pub(crate) upvalues: HashMap<Id, PointerValue<'ctx>>,
    pub(crate) cleanup: Option<BasicBlock<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub(crate) fn begin(function: FunctionValue<'ctx>) -> Scope<'ctx> {
        Scope {
            function,
            parent_variables: HashSet::default(),
            variables: HashMap::default(),
            closure: vec![],
            upvalues: HashMap::default(),
            cleanup: None,
        }
    }

    pub(crate) fn child(&self, function: FunctionValue<'ctx>) -> Scope<'ctx> {
        Scope {
            function,
            parent_variables: self
                .variables
                .keys()
                .chain(self.parent_variables.iter())
                .cloned()
                .collect(),
            variables: HashMap::default(),
            closure: vec![],
            upvalues: HashMap::default(),
            cleanup: None,
        }
    }

    pub(crate) fn set_cleanup(&mut self, cleanup: BasicBlock<'ctx>) {
        self.cleanup = Some(cleanup);
    }

    pub(crate) fn sret(&self) -> PointerValue<'ctx> {
        self.function
            .get_first_param()
            .unwrap()
            .into_pointer_value()
    }

    pub(crate) fn get_closure_ptr(&self) -> PointerValue<'ctx> {
        self.function.get_last_param().unwrap().into_pointer_value()
    }
}
