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
        let if_false = loop {
            branches.push(IfBranch::parse(parser)?);
            match parser.expect(TokenType::KwElse) {
                Ok(..) => {
                    if parser.check(TokenType::KwIf).is_ok() {
                        continue;
                    } else {
                        break Some(Block::parse(parser)?);
                    }
                }
                Err(..) => break None,
            }
        };
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
