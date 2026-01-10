use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A direct unification query.
///
/// ```trilogy
/// pattern = expression
/// ```
#[derive(Clone, Debug)]
pub struct DirectUnification {
    pub pattern: Pattern,
    pub eq: Token,
    pub expression: Expression,
    pub span: Span,
}

impl DirectUnification {
    pub(crate) fn parse(parser: &mut Parser, pattern: Pattern) -> SyntaxResult<Self> {
        let eq = parser.expect(OpEq).expect("Caller should have found this");
        let expression = Expression::parse_or_pattern(parser)?.map_err(|patt| {
            let error = SyntaxError::new(
                patt.span(),
                "expected an expression on the right side of `=`, but found a pattern",
            );
            parser.error(error.clone());
            error
        })?;
        Ok(Self {
            span: pattern.span().union(expression.span()),
            pattern,
            eq,
            expression,
        })
    }
}

impl Spanned for DirectUnification {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(direct_keyword: "x = 5" => Query::parse => Query::Direct(DirectUnification { .. }));
    test_parse!(direct_pattern: "5 = 5" => Query::parse => Query::Direct(DirectUnification { .. }));
    test_parse!(direct_collection: "[..a] = [1, 2, 3]" => Query::parse => Query::Direct(DirectUnification { .. }));
    test_parse_error!(direct_no_op_eq: "[..a] += [1, 2, 3]" => Query::parse);
    test_parse_error!(direct_no_expr: "a b = 123" => Query::parse);
    test_parse_error!(direct_invalid_expr: "a = let x = 5" => Query::parse);
}
