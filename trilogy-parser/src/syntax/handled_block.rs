use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct HandledBlock {
    start: Token,
    pub block: Block,
    pub handlers: Vec<Handler>,
}

impl Spanned for HandledBlock {
    fn span(&self) -> Span {
        self.start.span.union(if self.handlers.is_empty() {
            self.block.span()
        } else {
            self.handlers.span()
        })
    }
}

impl HandledBlock {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwWith)
            .map_err(|token| parser.expected(token, "expected `with`"))?;

        let block = Block::parse(parser)?;

        let mut handlers = vec![];
        loop {
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
                return Ok(Self {
                    start,
                    block,
                    handlers,
                });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(handled_block_else_yield: "with {} else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::Else _)])");
    test_parse!(handled_block_else_invert: "with {} else invert {}" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::Else _)])");
    test_parse!(handled_block_else_resume: "with {} else resume {}" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::Else _)])");
    test_parse!(handled_block_else_cancel: "with {} else cancel {}" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::Else _)])");
    test_parse!(handled_block_yield: "with {} when 'x yield else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_invert_block: "with {} when 'x invert {} else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_resume_block: "with {} when 'x resume {} else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_cancel_block: "with {} when 'x cancel {} else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_invert_expr: "with {} when 'x invert 3 else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_resume_expr: "with {} when 'x resume 3 else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_cancel_expr: "with {} when 'x cancel 3 else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_multiple_yield: "with {} when 'x yield when 'y yield else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::When _) (Handler::Else _)])");
    test_parse!(handled_block_multiple_invert: "with {} when 'x invert {} when 'y invert {} else yield" => HandledBlock::parse => "(HandledBlock (Block []) [(Handler::When _) (Handler::When _) (Handler::Else _)])");
    test_parse_error!(handled_block_expr: "with 3 else yield" => HandledBlock::parse);
}
