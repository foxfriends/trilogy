use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// A slot definition item.
///
/// ```trilogy
/// slot name = value
/// ```
#[derive(Clone, Debug)]
pub struct SlotDefinition {
    pub slot: Token,
    pub r#mut: Option<Token>,
    pub name: Identifier,
    pub eq: Token,
    pub body: Expression,
    pub span: Span,
}

impl Spanned for SlotDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl SlotDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let slot = parser
            .expect(TokenType::KwSlot)
            .map_err(|token| parser.expected(token, "expected `slot` to begin slot definition"))?;
        let r#mut = parser.expect(TokenType::KwMut).ok();
        let name = Identifier::parse(parser)?;
        let eq = parser
            .expect(TokenType::OpEq)
            .map_err(|token| parser.expected(token, "expected `=` in slot definition"))?;
        let body = Expression::parse(parser)?;
        Ok(SlotDefinition {
            span: slot.span.union(body.span()),
            r#mut,
            slot,
            name,
            eq,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(const_valid: "slot x = 123" => SlotDefinition::parse => SlotDefinition{..});
    test_parse!(const_mutable_valid: "slot mut x = 123" => SlotDefinition::parse => SlotDefinition{..});
    test_parse_error!(const_no_name: "slot = 5" => SlotDefinition::parse);
    test_parse_error!(const_no_value: "slot hello" => SlotDefinition::parse => "expected `=` in slot definition");
}
