use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ForStatement {
    pub branches: Vec<ForStatementBranch>,
    pub else_block: Option<Block>,
}

impl Spanned for ForStatement {
    fn span(&self) -> Span {
        match &self.else_block {
            None => self.branches.span(),
            Some(block) => self.branches.span().union(block.span()),
        }
    }
}

#[derive(Clone, Debug, Spanned)]
pub struct ForStatementBranch {
    start: Token,
    pub query: Query,
    pub body: Block,
}
