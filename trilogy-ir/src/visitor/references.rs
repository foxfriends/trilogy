use super::{IrVisitable, IrVisitor};
use crate::ir::*;
use crate::Id;
use std::collections::HashSet;

pub struct References {
    is_expression: bool,
    references: HashSet<Id>,
}

impl References {
    pub fn of<N: IrVisitable>(node: &N) -> HashSet<Id> {
        let mut references = Self {
            is_expression: true,
            references: HashSet::default(),
        };
        node.visit(&mut references);
        references.references
    }
}

impl IrVisitor for References {
    fn visit_reference(&mut self, node: &Identifier) {
        if self.is_expression {
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
