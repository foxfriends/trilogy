use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A boolean query.
///
/// ```trilogy
/// is expression
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct BooleanQuery {
    pub is: Token,
    pub expression: Expression,
    span: Span,
}

impl BooleanQuery {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let is = parser.expect(KwIs).unwrap();
        let expression = Expression::parse_parameter_list(parser)?.map_err(|patt| {
            let error = SyntaxError::new(
                patt.span(),
                "expected an expression after `is`, but found a pattern",
            );
            parser.error(error.clone());
            error
        })?; // this isn't a parameter list, but we don't allow commas
        Ok(Self {
            span: is.span.union(expression.span()),
            is,
            expression,
        })
    }
}

impl Spanned for BooleanQuery {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(bool_query_simple: "is true" => BooleanQuery::parse => "(BooleanQuery _ _)");
    test_parse!(bool_query_expression: "is x < 5" => BooleanQuery::parse => "(BooleanQuery _ _)");
    test_parse!(bool_query_application: "is f x y" => BooleanQuery::parse => "(BooleanQuery _ _)");
    test_parse_error!(bool_query_commas: "is x, x" => BooleanQuery::parse);
    test_parse!(bool_query_commas_parens: "is (x, x)" => BooleanQuery::parse => "(BooleanQuery _ _)");
    test_parse_error!(bool_query_invalid_expr: "is { let x = 5 }" => BooleanQuery::parse);
}
