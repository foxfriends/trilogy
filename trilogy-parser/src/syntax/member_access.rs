use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct MemberAccess {
    pub container: Expression,
    pub member: Member,
}

impl MemberAccess {
    pub(crate) fn parse(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Self> {
        parser
            .expect(TokenType::OpDot)
            .expect("Caller should have found this");
        let member = if parser.expect(TokenType::OBrack).is_ok() {
            let expression = Expression::parse(parser)?;
            parser
                .expect(TokenType::CBrack)
                .map_err(|token| parser.expected(token, "expected `]`"))?;
            Member::Dynamic(Box::new(expression))
        } else {
            Member::Static(Box::new(Identifier::parse(parser)?))
        };
        Ok(Self {
            container: lhs,
            member,
        })
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Member {
    Static(Box<Identifier>),
    Dynamic(Box<Expression>),
}
