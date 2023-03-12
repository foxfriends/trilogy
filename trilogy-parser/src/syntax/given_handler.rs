use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct GivenHandler {
    start: Token,
    pub head: RuleHead,
    pub body: Option<Query>,
}

impl GivenHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwGiven)
            .expect("Caller should have found this");
        let head = RuleHead::parse(parser)?;
        if parser.expect(OpLeftArrow).is_ok() {
            let body = Query::parse(parser)?;
            Ok(Self {
                start,
                head,
                body: Some(body),
            })
        } else {
            Ok(Self {
                start,
                head,
                body: None,
            })
        }
    }
}

impl Spanned for GivenHandler {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.head.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(given_fact: "given hello(1)" => GivenHandler::parse => "(GivenHandler (RuleHead _ [_]) ())");
    test_parse!(given_fact_multiparam: "given hello(1, 2, 3)" => GivenHandler::parse => "(GivenHandler (RuleHead _ [_ _ _]) ())");
    test_parse!(given_fact_noparam: "given hello()" => GivenHandler::parse => "(GivenHandler (RuleHead _ []) ())");
    test_parse!(given_rule: "given hello(a) <- is a > 0" => GivenHandler::parse => "(GivenHandler (RuleHead _ [_]) (Query::Is _))");
    test_parse!(given_rule_multiparam: "given hello(a, b) <- lhs(a) and rhs(b)" => GivenHandler::parse => "(GivenHandler (RuleHead _ [_ _]) (Query::Conjunction _))");
    test_parse!(given_rule_noparam: "given hello() <- what()" => GivenHandler::parse => "(GivenHandler (RuleHead _ []) (Query::Lookup _))");
    test_parse_error!(given_fact_invalid_pattern: "given hello(2 * n, n)" => GivenHandler::parse);
    test_parse_error!(given_rule_invalid_pattern: "given hello(2 * n, x) <- x = n / 2" => GivenHandler::parse);
    test_parse_error!(given_rule_invalid_body: "given hello(n, x) <- {}" => GivenHandler::parse);
    test_parse_error!(given_rule_invalid_separator: "given hello(n, x) => is x < n" => GivenHandler::parse);
}
