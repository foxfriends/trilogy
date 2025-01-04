use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ForStatement {
    pub branches: Vec<ForStatementBranch>,
    pub else_block: Option<Block>,
    span: Span,
}

impl ForStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mut branches = vec![];
        loop {
            branches.push(ForStatementBranch::parse(parser)?);
            if parser.check(TokenType::KwElse).is_ok() && parser.predict(TokenType::KwFor) {
                parser.consume();
                continue;
            }
            break;
        }
        let else_block = parser
            .expect(TokenType::KwElse)
            .ok()
            .map(|_| Block::parse(parser))
            .transpose()?;
        let span = match &else_block {
            None => branches
                .first()
                .unwrap()
                .span()
                .union(branches.last().unwrap().span()),
            Some(block) => branches.span().union(block.span()),
        };

        Ok(Self {
            span,
            branches,
            else_block,
        })
    }
}

impl Spanned for ForStatement {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ForStatementBranch {
    pub r#for: Token,
    pub query: Query,
    pub body: Block,
    span: Span,
}

impl Spanned for ForStatementBranch {
    fn span(&self) -> Span {
        self.span
    }
}

impl ForStatementBranch {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
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

    test_parse!(for_in: "for x in xs {}" => ForStatement::parse => "(ForStatement [(ForStatementBranch _ _ _)] ())");
    test_parse!(for_chained: "for x in xs {} else for y in ys {} else for z in zs {}" => ForStatement::parse => "(ForStatement [(ForStatementBranch _ _ _) (ForStatementBranch _ _ _) (ForStatementBranch _ _ _)] ())");
    test_parse!(for_else: "for x in xs {} else for y in ys {} else {}" => ForStatement::parse => "(ForStatement [(ForStatementBranch _ _ _) (ForStatementBranch _ _ _)] (Block _ _ _))");
    test_parse!(for_lookup: "for check(a, b, 3) {}" => ForStatement::parse => "(ForStatement [(ForStatementBranch _ _ _)] ())");
    test_parse!(for_body: "for check(a, b, 3) { break unit }" => ForStatement::parse => "(ForStatement [(ForStatementBranch _ _ (Block _ [_] _))] ())");
    test_parse_error!(for_query_expr: "for a + b { break }" => ForStatement::parse);
    test_parse_error!(for_body_expr: "for check(a, b) (a + b)" => ForStatement::parse);
}
