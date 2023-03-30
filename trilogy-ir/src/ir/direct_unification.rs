use super::*;

#[derive(Clone, Debug)]
pub struct DirectUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}

impl DirectUnification {
    pub(super) fn new(pattern: Pattern, expression: Expression) -> Self {
        Self {
            pattern,
            expression,
        }
    }
}
