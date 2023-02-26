use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ElementUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}

impl ElementUnification {
    pub(crate) fn parse(parser: &mut Parser, pattern: Pattern) -> SyntaxResult<Self> {
        parser.expect(KwIn).expect("Caller should have found this");
        let expression = Expression::parse_parameter_list(parser)?;
        Ok(Self {
            pattern,
            expression,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(element_keyword: "x in []" => Query::parse => "(Query::Element (ElementUnification _ _))");
    test_parse!(element_pattern: "5 in [5]" => Query::parse => "(Query::Element (ElementUnification _ _))");
    test_parse!(element_identifier: "x in xs" => Query::parse => "(Query::Element (ElementUnification _ _))");
    test_parse!(element_collection: "[..a] in [[], [1]]" => Query::parse => "(Query::Element (ElementUnification _ _))");
    test_parse_error!(element_no_expr: "a b in 123" => Query::parse);
    test_parse_error!(element_invalid_expr: "a in {}" => Query::parse);
}
