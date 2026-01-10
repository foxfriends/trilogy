use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// An array comprehension expression.
///
/// ```trilogy
/// [x for query(x)]
/// ```
#[derive(Clone, Debug)]
pub struct ArrayComprehension {
    pub open_bracket: Token,
    pub expression: Expression,
    pub r#for: Token,
    pub query: Query,
    pub close_bracket: Token,
    pub span: Span,
}

impl Spanned for ArrayComprehension {
    fn span(&self) -> Span {
        self.span
    }
}

impl ArrayComprehension {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_bracket: Token,
        expression: Expression,
    ) -> SyntaxResult<Self> {
        let r#for = parser
            .expect(KwFor)
            .map_err(|token| parser.expected(token, "expected `for` in array comprehension"))?;
        let query = Query::parse(parser)?;
        let close_bracket = parser
            .expect(CBrack)
            .map_err(|token| parser.expected(token, "expected `]` to end array comprehension"))?;
        let span = open_bracket.span.union(close_bracket.span);
        Ok(Self {
            open_bracket,
            expression,
            r#for,
            query,
            close_bracket,
            span,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(arraycomp_simple: "[x for x in array]" => Expression::parse => Expression::ArrayComprehension(ArrayComprehension { .. }));
    test_parse!(arraycomp_complex: "[x:y for lookup(x) and another(y)]" => Expression::parse => Expression::ArrayComprehension(ArrayComprehension { .. }));
    test_parse!(array_comp_seq: "[{x; y} for lookup(x, y)]" => Expression::parse => Expression::ArrayComprehension(ArrayComprehension { .. }));

    test_parse_error!(arraycomp_no_commas: "[x, y for lookup(x, y)]" => Expression::parse => "only one element may precede the `for` keyword in a comprehension");
    test_parse_error!(arraycomp_no_end: "[x for x in array" => Expression::parse => "expected `]` to end array comprehension");
    test_parse_error!(arraycomp_no_expression: "[for y]" => Expression::parse);

    test_parse_error!(arraycomp_invalid_query: "[x for y]" => Expression::parse);
    test_parse_error!(arraycomp_invalid_expr: "[() for x in y]" => Expression::parse);
}
