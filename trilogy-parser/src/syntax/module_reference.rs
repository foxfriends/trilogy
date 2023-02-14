use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ModuleReference {
    start: Token,
    pub name: Identifier,
    pub arguments: Vec<Expression>,
}

impl ModuleReference {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::OpAt)
            .map_err(|token| parser.expected(token, "expected `@` in module reference"))?;
        let name = Identifier::parse(parser)?;

        // Same logic as with a regular application, module references are like
        // a weird hard-coded application situation, in path precedence.
        let mut arguments = vec![];
        while parser.check(Expression::PREFIX).is_ok() && parser.is_spaced {
            arguments.push(Expression::parse_precedence(parser, Precedence::Path)?);
        }

        Ok(Self {
            start,
            name,
            arguments,
        })
    }
}
