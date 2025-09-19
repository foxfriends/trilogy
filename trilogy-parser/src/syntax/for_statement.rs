use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ForStatement {
    pub r#for: Token,
    pub query: Query,
    pub body: Block,
    span: Span,
}

impl Spanned for ForStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl ForStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#for = parser.expect(TokenType::KwFor).unwrap();
        let query = Query::parse(parser)?;
        let body = Block::parse(parser)?;
        Ok(Self {
            span: r#for.span.union(body.span()),
            r#for,
            query,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(for_in: "for x in xs {}" => ForStatement::parse => "(ForStatement _ _ _)");
    test_parse!(for_lookup: "for check(a, b, 3) {}" => ForStatement::parse => "(ForStatement _ _ _)");
    test_parse!(for_body: "for check(a, b, 3) { break unit }" => ForStatement::parse => "(ForStatement _ _ _)");
    test_parse_error!(for_query_expr: "for a + b { break }" => ForStatement::parse);
    test_parse_error!(for_body_expr: "for check(a, b) (a + b)" => ForStatement::parse);
}
