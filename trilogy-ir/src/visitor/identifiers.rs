use super::{IrVisitable, IrVisitor};
use crate::ir::*;

pub struct Identifiers {
    identifiers: Vec<Identifier>,
}

impl Identifiers {
    pub fn of<N: IrVisitable>(node: &N) -> Vec<Identifier> {
        let mut identifiers = Self {
            identifiers: Vec::default(),
        };
        node.visit(&mut identifiers);
        identifiers.identifiers
    }
}

impl IrVisitor for Identifiers {
    fn visit_value(&mut self, node: &Value) {
        match node {
            Value::Reference(ident) => {
                self.identifiers.push((**ident).clone());
            }
            _ => node.visit(self),
        }
    }

    fn visit_fn(&mut self, _node: &Function) {}
    fn visit_do(&mut self, _node: &Procedure) {}
    fn visit_qy(&mut self, _node: &Rule) {}
}
