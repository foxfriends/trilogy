use super::HasBindings;
use super::{IrVisitable, IrVisitor};
use crate::Id;
use crate::ir::*;
use std::collections::HashSet;

pub struct References {
    is_expression: bool,
    ignore: HashSet<Id>,
    references: HashSet<Id>,
}

impl References {
    pub fn of<N: IrVisitable>(node: &N) -> HashSet<Id> {
        let mut references = Self {
            is_expression: true,
            ignore: HashSet::default(),
            references: HashSet::default(),
        };
        node.visit(&mut references);
        references.references
    }
}

impl IrVisitor for References {
    fn visit_reference(&mut self, node: &Identifier) {
        if self.is_expression && !self.ignore.contains(&node.id) {
            self.references.insert(node.id.clone());
        }
    }

    fn visit_application(&mut self, node: &Application) {
        match &node.function.value {
            Value::Builtin(Builtin::Pin) => {
                let was_expression = std::mem::replace(&mut self.is_expression, true);
                node.visit(self);
                self.is_expression = was_expression;
            }
            _ if self.is_expression => node.visit(self),
            _ => {}
        }
    }

    fn visit_pattern(&mut self, node: &Expression) {
        let was_expression = std::mem::replace(&mut self.is_expression, false);
        node.visit(self);
        self.is_expression = was_expression;
    }

    fn visit_fn(&mut self, node: &Function) {
        for param in &node.parameters {
            for binding in param.bindings() {
                self.ignore.insert(binding);
            }
        }
        node.body.visit(self)
    }

    fn visit_case(&mut self, node: &Case) {
        self.ignore.extend(node.bindings());
        node.guard.visit(self);
        node.body.visit(self);
    }

    fn visit_iterator(&mut self, node: &Iterator) {
        self.ignore.extend(node.query.bindings());
        node.value.visit(self);
    }

    fn visit_direct_unification(&mut self, node: &Unification) {
        self.ignore.extend(node.bindings());
        node.expression.visit(self)
    }

    fn visit_element_unification(&mut self, node: &Unification) {
        self.ignore.extend(node.bindings());
        node.expression.visit(self)
    }

    fn visit_handler(&mut self, node: &Handler) {
        self.ignore.extend(node.bindings());
        node.guard.visit(self);
        node.body.visit(self);
    }

    fn visit_do(&mut self, node: &Procedure) {
        for param in &node.parameters {
            self.ignore.extend(param.bindings());
        }
        node.body.visit(self)
    }

    fn visit_qy(&mut self, node: &Rule) {
        for param in &node.parameters {
            self.ignore.extend(param.bindings());
        }
        node.body.visit(self)
    }

    fn visit_query_value(&mut self, node: &QueryValue) {
        use QueryValue::*;

        match node {
            Is(..) => {
                let was_expression = std::mem::replace(&mut self.is_expression, true);
                node.visit(self);
                self.is_expression = was_expression;
            }
            _ => node.visit(self),
        }
    }
}

pub trait HasReferences: IrVisitable + Sized {
    fn references(&self) -> HashSet<Id> {
        References::of(self)
    }
}

impl<T: IrVisitable> HasReferences for T {}
