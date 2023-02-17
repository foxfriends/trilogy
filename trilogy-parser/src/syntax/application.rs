use super::{expression::Precedence, *};
use crate::Parser;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Application {
    pub function: Expression,
    pub argument: Expression,
}

impl Application {
    pub(crate) fn parse(parser: &mut Parser, function: Expression) -> SyntaxResult<Self> {
        let argument = Expression::parse_precedence(parser, Precedence::Application)?;
        Ok(Self { function, argument })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(application_simple: "hello world" => Expression::parse => "(Expression::Application (Application _ _))");
    test_parse!(application_path: "@a::hello @b::world" => Expression::parse => "(Expression::Application (Application _ _))");
    test_parse!(application_parenthesized: "hello (a + world)" => Expression::parse => "(Expression::Application (Application _ _))");
    test_parse!(application_unary_not: "hello !b" => Expression::parse => "(Expression::Application (Application _ (Expression::Unary _)))");
    test_parse!(not_application_unary_minus: "hello - b" => Expression::parse => "(Expression::Binary _)");
    test_parse!(application_unary_negate: "hello ~b" => Expression::parse => "(Expression::Application (Application _ (Expression::Unary _)))");
    test_parse!(application_keyword: "hello if x then 3 else 4" => Expression::parse => "(Expression::Application (Application _ (Expression::IfElse _)))");
    test_parse!(application_binop: "hello a + world" => Expression::parse => "
        (Expression::Binary
          (BinaryOperation
            (Expression::Application (Application _ _))
            _
            _))");
}
