use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct QyExpression {
    pub qy_token: Token,
    pub oparen: Token,
    pub parameters: Vec<Pattern>,
    pub cparen: Token,
    pub arrow: Token,
    pub body: Query,
}

impl Spanned for QyExpression {
    fn span(&self) -> Span {
        self.qy_token.span.union(self.body.span())
    }
}

impl QyExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let qy_token = parser.expect(KwQy).expect("Caller should have found this");
        let mut parameters = vec![];
        let oparen = parser.expect(OParen).map_err(|token| {
            parser.expected(
                token,
                "expected `(` to qy_token parameter list following `qy`",
            )
        })?;
        let cparen = loop {
            if let Ok(paren) = parser.expect(CParen) {
                break paren;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(OpComma).is_ok() {
                continue;
            }
            let cparen = parser
                .expect(CParen)
                .map_err(|token| parser.expected(token, "expected `)` to end parameter list"))?;
            break cparen;
        };
        let arrow = parser
            .expect(OpLeftArrow)
            .map_err(|token| parser.expected(token, "expected `<-` to begin `qy` body"))?;
        let body = Query::parse(parser)?;
        Ok(Self {
            qy_token,
            oparen,
            parameters,
            cparen,
            arrow,
            body,
        })
    }

    pub fn qy_token(&self) -> &Token {
        &self.qy_token
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
