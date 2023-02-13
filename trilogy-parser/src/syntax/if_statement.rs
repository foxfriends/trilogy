use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct IfStatement {
    pub branches: Vec<IfBranch>,
    pub if_false: Option<Block>,
}

impl IfStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mut branches = vec![];
        while parser.check(TokenType::KwIf).is_ok() {
            branches.push(IfBranch::parse(parser)?);
        }
        let if_false = parser
            .expect(TokenType::KwElse)
            .ok()
            .map(|_| Block::parse(parser))
            .transpose()?;
        Ok(Self { branches, if_false })
    }
}

impl Spanned for IfStatement {
    fn span(&self) -> Span {
        match &self.if_false {
            None => self.branches.span(),
            Some(block) => self.branches.span().union(block.span()),
        }
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct IfBranch {
    start: Token,
    pub condition: Expression,
    pub body: Block,
}

impl IfBranch {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwIf)
            .expect("Caller should have found this");
        let condition = Expression::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self {
            start,
            condition,
            body,
        })
    }
}
