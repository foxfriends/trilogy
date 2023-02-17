use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
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
