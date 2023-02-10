use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct MemberAccess {
    pub container: Expression,
    pub member: Member,
}

impl MemberAccess {
    pub(crate) fn parse(parser: &mut Parser, lhs: impl Into<Expression>) -> SyntaxResult<Self> {
        parser
            .expect(TokenType::OpDot)
            .expect("Caller should have found this");
        let member = if parser.expect(TokenType::OBrack).is_ok() {
            let expression = Expression::parse(parser)?;
            parser.expect(TokenType::CBrack).map_err(|token| {
                let error = SyntaxError::new(token.span, "expected `]`");
                parser.error(error.clone());
                error
            })?;
            Member::Dynamic(Box::new(expression))
        } else {
            Member::Static(Box::new(Identifier::parse(parser)?))
        };
        Ok(Self {
            container: lhs.into(),
            member,
        })
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Member {
    Static(Box<Identifier>),
    Dynamic(Box<Expression>),
}
