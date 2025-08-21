use super::{IrVisitable, IrVisitor};
use crate::Converter;
use crate::ir::*;

pub struct ValidForRule<'a, 'c> {
    converter: &'a mut Converter<'c>,
}

impl<'a, 'c> ValidForRule<'a, 'c> {
    fn validate<N: IrVisitable>(converter: &'a mut Converter<'c>, node: &N) {
        let mut validator = Self { converter };
        node.visit(&mut validator);
    }
}

impl IrVisitor for ValidForRule<'_, '_> {
    fn visit_expression(&mut self, node: &Expression) {
        match &node.value {
            Value::Builtin(Builtin::Return) => {
                self.converter.error(crate::Error::NoReturnFromRule {
                    expression: node.span,
                })
            }
            _ => node.visit(self),
        }
    }

    fn visit_procedure(&mut self, _proc: &Procedure) {}
    fn visit_function(&mut self, _func: &Function) {}
    fn visit_rule(&mut self, _rule: &Rule) {}
}

pub trait ValidateForRule: IrVisitable + Sized {
    fn validate_for_rule(&self, converter: &mut Converter) {
        ValidForRule::validate(converter, self)
    }
}

impl<T: IrVisitable> ValidateForRule for T {}
