use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// An element unification (`in`) query.
///
/// ```trilogy
/// pattern in expression
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ElementUnification {
    pub pattern: Pattern,
    pub r#in: Token,
    pub expression: Expression,
    span: Span,
}

impl Spanned for ElementUnification {
    fn span(&self) -> Span {
        self.span
    }
}

impl ElementUnification {
    pub(crate) fn parse(parser: &mut Parser, pattern: Pattern) -> SyntaxResult<Self> {
        let r#in = parser.expect(KwIn).unwrap();
        let expression = Expression::parse_or_pattern(parser)?.map_err(|patt| {
            let error = SyntaxError::new(
                patt.span(),
                "expected an expression after `in`, but found a pattern",
            );
            parser.error(error.clone());
            error
        })?;
        Ok(Self {
            span: pattern.span().union(expression.span()),
            pattern,
            r#in,
            expression,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(element_keyword: "x in []" => Query::parse => "(Query::Element (ElementUnification _ _ _))");
    test_parse!(element_pattern: "5 in [5]" => Query::parse => "(Query::Element (ElementUnification _ _ _))");
    test_parse!(element_identifier: "x in xs" => Query::parse => "(Query::Element (ElementUnification _ _ _))");
    test_parse!(element_collection: "[..a] in [[], [1]]" => Query::parse => "(Query::Element (ElementUnification _ _ _))");
    test_parse_error!(element_no_expr: "a b in 123" => Query::parse);
    test_parse_error!(element_invalid_expr: "a in let x = 5" => Query::parse);
}
