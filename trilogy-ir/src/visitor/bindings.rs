use super::{IrVisitable, IrVisitor};
use crate::ir::*;
use crate::Id;
use std::collections::HashSet;

pub struct Bindings {
    bindings: HashSet<Id>,
}

impl Bindings {
    pub fn of<N: IrVisitable>(node: &N) -> HashSet<Id> {
        let mut bindings = Self {
            bindings: HashSet::default(),
        };
        node.visit(&mut bindings);
        bindings.bindings
    }
}

impl IrVisitor for Bindings {
    fn visit_value(&mut self, node: &Value) {
        use Value::*;

        match node {
            Sequence(seq) => self.visit_sequence(seq),
            Pack(pack) => self.visit_pack(pack),
            Mapping(pair) => self.visit_mapping(pair),
            Conjunction(pair) => self.visit_conjunction(pair),
            Disjunction(pair) => self.visit_disjunction(pair),
            Application(application) => self.visit_application(application),
            Reference(ident) => {
                self.bindings.insert(ident.id.clone());
            }
            _ => {}
        }
    }

    fn visit_application(&mut self, node: &Application) {
        match &node.function.value {
            Value::Builtin(Builtin::Pin) => {}
            _ => node.visit(self),
        }
    }

    fn visit_query_value(&mut self, node: &QueryValue) {
        use QueryValue::*;

        match node {
            Not(..) | Is(..) => {}
            _ => node.visit(self),
        }
    }

    fn visit_direct_unification(&mut self, node: &Unification) {
        node.pattern.visit(self);
    }

    fn visit_element_unification(&mut self, node: &Unification) {
        node.pattern.visit(self);
    }

    fn visit_handler(&mut self, node: &Handler) {
        node.pattern.visit(self);
    }

    fn visit_case(&mut self, node: &Case) {
        node.pattern.visit(self);
    }
}

pub trait HasBindings: IrVisitable + Sized {
    fn bindings(&self) -> HashSet<Id> {
        Bindings::of(self)
    }
}

impl<T: IrVisitable> HasBindings for T {}
