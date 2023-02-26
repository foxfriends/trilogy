use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct FunctionDefinition {
    pub head: FunctionHead,
    pub body: Expression,
}

impl FunctionDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let head = FunctionHead::parse(parser)?;
        parser
            .expect(TokenType::OpEq)
            .map_err(|token| parser.expected(token, "expected `=` in function definition"))?;
        let body = Expression::parse(parser)?;
        Ok(FunctionDefinition { head, body })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(func_one_param: "func hello x = x" => FunctionDefinition::parse => "(FunctionDefinition (FunctionHead _ [_]) _)");
    test_parse!(func_multi_param: "func hello x y z = x + y + z" => FunctionDefinition::parse => "(FunctionDefinition (FunctionHead _ [_ _ _]) _)");
    test_parse!(func_pattern_param: "func find f x:xs = if f x then x else find f xs" => FunctionDefinition::parse => "(FunctionDefinition (FunctionHead _ [_ _]) _)");
    test_parse_error!(func_no_params: "func three = 3" => FunctionDefinition::parse);
    test_parse_error!(func_invalid_body: "func hello x = {}" => FunctionDefinition::parse);
    test_parse_error!(func_missing_body: "func hello x" => FunctionDefinition::parse => "expected `=` in function definition");
}
