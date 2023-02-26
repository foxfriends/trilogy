use super::{expression::Precedence, *};
use crate::{Parser, Spanned, TokenPattern};
use source_span::Span;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct FunctionAssignment {
    pub lhs: Expression,
    pub function: Identifier,
    pub arguments: Vec<Expression>,
}

impl Spanned for FunctionAssignment {
    fn span(&self) -> Span {
        self.lhs.span().union(if self.arguments.is_empty() {
            self.function.span()
        } else {
            self.arguments.span()
        })
    }
}

impl FunctionAssignment {
    pub(crate) fn parse(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Self> {
        let function = Identifier::parse_eq(parser)?;
        let mut arguments = vec![];
        loop {
            // NOTE: This has potential to be a pretty unintuitive parse. Sugar
            // was never that healthy I suppose.
            arguments.push(Expression::parse_precedence(
                parser,
                Precedence::Application,
            )?);
            parser.chomp(); // to ensure any line ending is detected
            if parser.is_line_start || !Expression::PREFIX.matches(parser.peek()) {
                break;
            }
        }
        Ok(Self {
            lhs,
            function,
            arguments,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(fn_assignment_single_arg: "xs push= x" => Statement::parse => "(Statement::FunctionAssignment (FunctionAssignment _ _ [_]))");
    test_parse!(fn_assignment_multi_arg: "xs fold= x f" => Statement::parse => "(Statement::FunctionAssignment (FunctionAssignment _ _ [_ _]))");
    test_parse!(fn_assignment_operators_paren: "xs push= (x + y)" => Statement::parse => "(Statement::FunctionAssignment (FunctionAssignment _ _ [_]))");
    test_parse_error!(fn_assignment_no_arg: "xs reverse=" => Statement::parse);
    test_parse_error!(fn_assignment_lhs_not_lvalue: "xs ys push= x y" => Statement::parse);
    test_parse_error!(fn_assignment_spaced: "xs push = x" => Statement::parse);
    test_parse_error!(fn_assignment_operators: "xs push= x + y" => Statement::parse);
}
