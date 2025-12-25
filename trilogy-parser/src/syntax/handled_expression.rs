use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct HandledExpression {
    pub with: Token,
    pub expression: Expression,
    pub obrace: Token,
    pub handlers: Vec<Handler>,
    pub cbrace: Token,
    span: Span,
}

impl Spanned for HandledExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl HandledExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let with = parser
            .expect(KwWith)
            .map_err(|token| parser.expected(token, "expected `with`"))?;

        let expression = Expression::parse(parser)?;

        let obrace = parser
            .expect(OBrace)
            .map_err(|token| parser.expected(token, "expected { to begin `with` handlers"))?;

        let mut handlers = vec![];
        let cbrace = loop {
            if let Ok(cbrace) = parser.expect(CBrace) {
                break cbrace;
            }
            if let Err(token) = parser.check([KwWhen, KwElse]) {
                let error = SyntaxError::new(
                    token.span,
                    "expected `when`, or `else` to start an effect handler",
                );
                parser.error(error.clone());
                return Err(error);
            }
            let handler = Handler::parse(parser)?;
            let end = matches!(handler, Handler::Else(..));
            handlers.push(handler);
            if end {
                break parser.expect(CBrace).map_err(|token| {
                    parser.expected(token, "expected } to end `with` handlers")
                })?;
            }
        };
        Ok(Self {
            span: with.span.union(handlers.last().unwrap().span()),
            with,
            expression,
            obrace,
            handlers,
            cbrace,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(handled_expr_else_yield: "with 3 { else yield }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::Else _)] _)");
    test_parse!(handled_expr_else_resume: "with 3 { else resume 3 }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::Else _)] _)");
    test_parse!(handled_expr_else_cancel: "with 3 { else cancel 3 }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::Else _)] _)");
    test_parse!(handled_expr_yield: "with 3 { when 'x yield else yield }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::When _) (Handler::Else _)] _)");
    test_parse_error!(handled_expr_resume_block: "with 3 { when 'x resume {} else yield }" => HandledExpression::parse);
    test_parse_error!(handled_expr_cancel_block: "with 3 { when 'x cancel {} else yield }" => HandledExpression::parse);
    test_parse!(handled_expr_resume_expr: "with 3 { when 'x resume 3 else yield }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::When _) (Handler::Else _)] _)");
    test_parse!(handled_expr_cancel_expr: "with 3 { when 'x cancel 3 else yield }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::When _) (Handler::Else _)] _)");
    test_parse!(handled_expr_multiple_yield: "with 3 { when 'x yield when 'y yield else yield }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::When _) (Handler::When _) (Handler::Else _)] _)");
    test_parse_error!(handled_expr_block: "with {} else yield" => HandledExpression::parse);
    test_parse!(handled_expr_no_else: "with 3 { when 'x yield }" => HandledExpression::parse => "(HandledExpression _ _ _ [(Handler::When _)] _)");
}
