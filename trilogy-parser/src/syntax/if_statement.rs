use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct IfStatement {
    pub branches: Vec<IfBranch>,
    pub if_false: Option<Block>,
}

impl Spanned for IfStatement {
    fn span(&self) -> Span {
        match &self.if_false {
            None => self.branches.span(),
            Some(block) => self.branches.span().union(block.span()),
        }
    }
}

#[derive(Clone, Debug, Spanned)]
pub struct IfBranch {
    start: Token,
    pub condition: Expression,
    pub body: Block,
}
