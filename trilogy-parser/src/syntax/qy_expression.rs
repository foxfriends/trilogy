use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct QyExpression {
    pub qy: Token,
    pub open_paren: Token,
    pub parameters: Vec<Pattern>,
    pub close_paren: Token,
    pub arrow: Token,
    pub body: Query,
    span: Span,
}

impl Spanned for QyExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl QyExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let qy = parser.expect(KwQy).unwrap();
        let mut parameters = vec![];
        let open_paren = parser.expect(OParen).map_err(|token| {
            parser.expected(token, "expected `(` to qy parameter list following `qy`")
        })?;
        let close_paren = loop {
            if let Ok(paren) = parser.expect(CParen) {
                break paren;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(OpComma).is_ok() {
                continue;
            }
            let close_paren = parser
                .expect(CParen)
                .map_err(|token| parser.expected(token, "expected `)` to end parameter list"))?;
            break close_paren;
        };
        let arrow = parser
            .expect(OpLeftArrow)
            .map_err(|token| parser.expected(token, "expected `<-` to begin `qy` body"))?;
        let body = Query::parse(parser)?;
        Ok(Self {
            span: qy.span.union(body.span()),
            qy,
            open_paren,
            parameters,
            close_paren,
            arrow,
            body,
        })
    }

    pub fn qy(&self) -> &Token {
        &self.qy
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(qy_pass: "qy() <- pass" => QyExpression::parse => "(QyExpression _ _ [] _ _ _)");
    test_parse!(qy_pass_params: "qy(a, b) <- pass" => QyExpression::parse => "(QyExpression _ _ [_ _] _ _ _)");
    test_parse!(qy_pass_params_trailing_comma: "qy(a, b, ) <- pass" => QyExpression::parse => "(QyExpression _ _ [_ _] _ _ _)");
    test_parse_error!(qy_pass_params_leading_comma: "qy(, a) <- pass" => QyExpression::parse);
    test_parse_error!(qy_pass_params_empty_comma: "qy(,) <- pass" => QyExpression::parse);
    test_parse_error!(qy_pass_missing_paren: "qy(a <-" => QyExpression::parse => "expected `)` to end parameter list");
    test_parse_error!(qy_pass_invalid: "qy() <- {}" => QyExpression::parse);
}
