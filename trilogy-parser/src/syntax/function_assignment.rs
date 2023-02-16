use super::{expression::Precedence, *};
use crate::Parser;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct FunctionAssignment {
    pub lhs: Expression,
    pub function: Identifier,
    pub arguments: Vec<Expression>,
}

impl FunctionAssignment {
    pub(crate) fn parse(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Self> {
        let function = Identifier::parse_eq(parser)?;
        let mut arguments = vec![];
        while {
            parser.chomp(); // to ensure any line ending is detected
            !parser.is_line_start
        } {
            // NOTE: This has potential to be a pretty unintuitive parse. Sugar
            // was never that healthy I suppose.
            arguments.push(Expression::parse_precedence(
                parser,
                Precedence::Application,
            )?);
        }
        Ok(Self {
            lhs,
            function,
            arguments,
        })
    }
}
