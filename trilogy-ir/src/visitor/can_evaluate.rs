use super::{IrVisitable, IrVisitor};
use crate::ir;

pub struct CanEvaluate(bool);

impl CanEvaluate {}

impl IrVisitor for CanEvaluate {
    fn visit_wildcard(&mut self) {
        self.0 = false;
    }

    fn visit_conjunction(&mut self, _: &(ir::Expression, ir::Expression)) {
        self.0 = false;
    }

    fn visit_disjunction(&mut self, _: &(ir::Expression, ir::Expression)) {
        self.0 = false;
    }

    fn visit_query(&mut self, _: &ir::Query) {}
    fn visit_pattern(&mut self, _: &ir::Expression) {}
}

pub trait HasCanEvaluate: IrVisitable + Sized {
    fn can_evaluate(&self) -> bool {
        let mut can_evaluate = CanEvaluate(true);
        self.visit(&mut can_evaluate);
        can_evaluate.0
    }
}

impl<T: IrVisitable> HasCanEvaluate for T {}
