use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// A constant definition item.
///
/// ```trilogy
/// const name = value
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ConstantDefinition {
    pub r#const: Token,
    pub name: Identifier,
    pub eq: Token,
    pub body: Expression,
    span: Span,
}

impl Spanned for ConstantDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl ConstantDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#const = parser.expect(TokenType::KwConst).map_err(|token| {
            parser.expected(token, "expected `const` to begin constant definition")
        })?;
        let name = Identifier::parse(parser)?;
        let eq = parser
            .expect(TokenType::OpEq)
            .map_err(|token| parser.expected(token, "expected `=` in constant definition"))?;
        let body = Expression::parse(parser)?;
        Ok(ConstantDefinition {
            span: r#const.span.union(body.span()),
            r#const,
            name,
            eq,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(const_valid: "const x = 123" => ConstantDefinition::parse => "(ConstantDefinition _ _ _ _)");
    test_parse_error!(const_no_name: "const = 5" => ConstantDefinition::parse);
    test_parse_error!(const_no_value: "const hello" => ConstantDefinition::parse => "expected `=` in constant definition");
}
