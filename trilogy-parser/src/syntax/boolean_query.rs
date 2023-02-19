use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BooleanQuery {
    start: Token,
    pub expression: Expression,
}

impl BooleanQuery {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwIs)
            .map_err(|token| parser.expected(token, "expected `is`"))?;
        let expression = Expression::parse_parameter_list(parser)?; // this isn't a parameter list, but we don't allow commas
        Ok(Self { start, expression })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(bool_query_simple: "is true" => BooleanQuery::parse => "(BooleanQuery _)");
    test_parse!(bool_query_expression: "is x < 5" => BooleanQuery::parse => "(BooleanQuery _)");
    test_parse!(bool_query_application: "is f x y" => BooleanQuery::parse => "(BooleanQuery _)");
    test_parse_error!(bool_query_commas: "is x, x" => BooleanQuery::parse);
    test_parse!(bool_query_commas_parens: "is (x, x)" => BooleanQuery::parse => "(BooleanQuery _)");
    test_parse_error!(bool_query_no_is: "(x, x)" => BooleanQuery::parse => "expected `is`");
    test_parse_error!(bool_query_invalid_expr: "is { let x = 5 }" => BooleanQuery::parse);
}
