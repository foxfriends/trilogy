use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct DirectUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}

impl DirectUnification {
    pub(crate) fn parse(parser: &mut Parser, pattern: Pattern) -> SyntaxResult<Self> {
        parser.expect(OpEq).expect("Caller should have found this");
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

    test_parse!(direct_keyword: "x = 5" => Query::parse => "(Query::Direct (DirectUnification _ _))");
    test_parse!(direct_pattern: "5 = 5" => Query::parse => "(Query::Direct (DirectUnification _ _))");
    test_parse!(direct_collection: "[..a] = [1, 2, 3]" => Query::parse => "(Query::Direct (DirectUnification _ _))");
    test_parse_error!(direct_no_op_eq: "[..a] += [1, 2, 3]" => Query::parse);
    test_parse_error!(direct_no_expr: "a b = 123" => Query::parse);
    test_parse_error!(direct_invalid_expr: "a = {}" => Query::parse);
}
