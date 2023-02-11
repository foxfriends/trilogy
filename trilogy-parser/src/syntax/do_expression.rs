use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct DoExpression {
    start: Token,
    pub parameters: Vec<Pattern>,
    pub body: DoBody,
}

impl DoExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwDo).expect("Caller should have found this");
        let mut parameters = vec![];
        loop {
            if parser.check(CParen).is_ok() {
                break;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(OpComma).is_ok() {
                continue;
            }
        }
        let body = DoBody::parse(parser)?;
        Ok(Self {
            start,
            parameters,
            body,
        })
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum DoBody {
    Block(Box<Block>),
    Expression(Box<Expression>),
}

impl DoBody {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if parser.check(OBrace).is_ok() {
            Ok(Self::Block(Box::new(Block::parse(parser)?)))
        } else {
            Ok(Self::Expression(Box::new(Expression::parse_precedence(
                parser,
                Precedence::Continuation,
            )?)))
        }
    }
}
