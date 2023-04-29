use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Block {
    start: Token,
    pub statements: Vec<Statement>,
    end: Token,
}

impl Spanned for Block {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}

impl Block {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OBrace)
            .map_err(|token| parser.expected(token, "expected `{`"))?;

        let mut statements = vec![];
        let end = loop {
            if let Ok(end) = parser.expect(CBrace) {
                break end;
            }
            statements.push(Statement::parse(parser)?);
            if let Ok(end) = parser.expect(CBrace) {
                break end;
            }
            if parser.expect(OpSemi).is_err() && !parser.is_line_start {
                let token = parser.peek();
                let error = SyntaxError::new(
                    token.span,
                    "expected end of block, or `;` or line break to separate statements",
                );
                parser.error(error);
            }
        };

        Ok(Self {
            start,
            statements,
            end,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(block_empty: "{}" => Block::parse => "(Block ())");
    test_parse!(block_single: "{ let x = 5 }" => Block::parse => "(Block [(Statement::Let _)])");
    test_parse!(block_single_semi: "{ let x = 5; }" => Block::parse => "(Block [(Statement::Let _)])");
    test_parse!(block_single_lines: "{
        let x = 5;
    }" => Block::parse => "(Block [(Statement::Let _)])");
    test_parse!(block_semis: "{ let x = 5; return x * 2; }" => Block::parse => "(Block [(Statement::Let _) (Statement::Return _)])");
    test_parse!(block_end_no_semi: "{ let x = 5; return x * 2 }" => Block::parse => "(Block [(Statement::Let _) (Statement::Return _)])");
    test_parse!(block_lines: "{
        let x = 5
        return x * 2
    }" => Block::parse => "(Block [(Statement::Let _) (Statement::Return _)])");
    test_parse!(block_lines_and_semis: "{
        let x = 5;
        return x * 2;
    }" => Block::parse => "(Block [(Statement::Let _) (Statement::Return _)])");
    test_parse_error!(block_no_breaks: "{ end end }" => Block::parse => "expected end of block, or `;` or line break to separate statements");
    test_parse_error!(block_no_close: "{ end " => Block::parse => "expected end of block, or `;` or line break to separate statements");
    test_parse_error!(block_no_braces: "end; end" => Block::parse => "expected `{`");
}
