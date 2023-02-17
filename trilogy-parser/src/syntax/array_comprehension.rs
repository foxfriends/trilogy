use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ArrayComprehension {
    start: Token,
    pub expression: Expression,
    pub query: Query,
    end: Token,
}

impl ArrayComprehension {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        expression: Expression,
    ) -> SyntaxResult<Self> {
        let query = Query::parse(parser)?;
        let end = parser
            .expect(CBrack)
            .map_err(|token| parser.expected(token, "expected `]` to end array comprehension"))?;
        Ok(Self {
            start,
            expression,
            query,
            end,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(arraycomp_simple: "[x for x in array]" => Expression::parse => "(Expression::ArrayComprehension (ArrayComprehension _ _))");
    test_parse!(arraycomp_complex: "[x:y for lookup(x) and another(y)]" => Expression::parse => "(Expression::ArrayComprehension (ArrayComprehension _ _))");
    test_parse!(arraycomp_parenthesized_commas: "[(x, y) for lookup(x, y)]" => Expression::parse => "(Expression::ArrayComprehension (ArrayComprehension _ _))");

    test_parse_error!(no_arraycomp_commas: "[x, y for lookup(x, y)]" => Expression::parse => "only one element may precede the `for` keyword in a comprehension");
    test_parse_error!(arraycomp_end: "[x for x in array" => Expression::parse => "expected `]` to end array comprehension");

    test_parse_error!(arraycomp_invalid_query: "[x for y]" => Expression::parse);
    test_parse_error!(arraycomp_invalid_expr: "[() for x in y]" => Expression::parse);
}
