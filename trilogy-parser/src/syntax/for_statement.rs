use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ForStatement {
    pub branches: Vec<ForStatementBranch>,
    pub else_block: Option<Block>,
}

impl ForStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mut branches = vec![];
        while parser.check(TokenType::KwFor).is_ok() {
            branches.push(ForStatementBranch::parse(parser)?);
        }
        let else_block = parser
            .expect(TokenType::KwElse)
            .ok()
            .map(|_| Block::parse(parser))
            .transpose()?;
        Ok(Self {
            branches,
            else_block,
        })
    }
}

impl Spanned for ForStatement {
    fn span(&self) -> Span {
        match &self.else_block {
            None => self.branches.span(),
            Some(block) => self.branches.span().union(block.span()),
        }
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ForStatementBranch {
    start: Token,
    pub query: Query,
    pub body: Block,
}

impl ForStatementBranch {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwFor)
            .expect("Caller should have found this");
        let query = Query::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self { start, query, body })
    }
}
