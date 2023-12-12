use trilogy_ir::ir;
use trilogy_ir::visitor::{IrVisitable, IrVisitor};

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub(crate) struct BreakContinue {
    pub st_break: bool,
    pub fc_break: bool,
    pub st_continue: bool,
    pub fc_continue: bool,
    is_application: bool,
}

impl BreakContinue {
    pub fn check(val: &impl IrVisitable) -> Self {
        let mut needs = Self {
            st_break: false,
            fc_break: false,
            st_continue: false,
            fc_continue: false,
            is_application: false,
        };
        val.visit(&mut needs);
        needs
    }
}

impl IrVisitor for BreakContinue {
    fn visit_builtin(&mut self, value: &ir::Builtin) {
        use ir::Builtin::*;

        match value {
            Break if self.is_application => self.st_break = true,
            Break => self.fc_break = true,
            Continue if self.is_application => self.st_continue = true,
            Continue => self.fc_continue = true,
            _ => {}
        }
    }

    fn visit_while(&mut self, node: &ir::While) {
        node.condition.visit(self);
    }

    fn visit_for(&mut self, node: &ir::Iterator) {
        node.query.visit(self);
    }

    fn visit_value(&mut self, node: &ir::Value) {
        match node {
            ir::Value::Builtin(..) => node.visit(self),
            _ => {
                self.is_application = false;
                node.visit(self);
            }
        }
    }

    fn visit_application(&mut self, node: &ir::Application) {
        self.is_application = true;
        node.function.visit(self);
        node.argument.visit(self);
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub(crate) struct ResumeCancel {
    pub st_cancel: bool,
    pub fc_cancel: bool,
    pub st_resume: bool,
    pub fc_resume: bool,
    is_application: bool,
}

impl ResumeCancel {
    pub fn check(val: impl IrVisitable) -> Self {
        let mut needs = Self {
            st_cancel: false,
            fc_cancel: false,
            st_resume: false,
            fc_resume: false,
            is_application: false,
        };
        val.visit(&mut needs);
        needs
    }
}

impl IrVisitor for ResumeCancel {
    fn visit_builtin(&mut self, value: &ir::Builtin) {
        use ir::Builtin::*;

        match value {
            Cancel if self.is_application => self.st_cancel = true,
            Cancel => self.fc_cancel = true,
            Resume if self.is_application => self.st_resume = true,
            Resume => self.fc_resume = true,
            _ => {}
        }
    }

    fn visit_handler(&mut self, node: &ir::Handler) {
        node.pattern.visit(self);
        node.guard.visit(self);
    }

    fn visit_value(&mut self, node: &ir::Value) {
        match node {
            ir::Value::Builtin(..) => node.visit(self),
            _ => {
                self.is_application = false;
                node.visit(self);
            }
        }
    }

    fn visit_application(&mut self, node: &ir::Application) {
        self.is_application = true;
        node.function.visit(self);
        node.argument.visit(self);
    }
}
