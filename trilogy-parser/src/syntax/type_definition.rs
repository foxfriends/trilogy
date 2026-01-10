use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct TypeDefinition {
    pub head: TypeHead,
    pub open_brace: Token,
    pub definitions: Vec<Definition>,
    pub close_brace: Token,
    pub span: Span,
}

impl Spanned for TypeDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl TypeDefinition {
    fn synchronize(parser: &mut Parser) {
        parser.synchronize([
            DocOuter, KwType, KwImport, KwFunc, KwProc, KwRule, KwSlot, KwExport, CBrace, EndOfFile,
        ]);
    }

    pub(crate) fn parse(parser: &mut Parser, head: TypeHead) -> SyntaxResult<Self> {
        let open_brace = parser.expect(OBrace).unwrap();

        let mut span = head.span();

        if let Ok(close_brace) = parser.expect(CBrace) {
            // empty type may be single line
            return Ok(Self {
                span: span.union(close_brace.span),
                head,
                open_brace,
                definitions: vec![],
                close_brace,
            });
        }

        let mut definitions = vec![];
        loop {
            match Definition::parse_in_module(parser) {
                Ok(Some(definition)) => definitions.push(definition),
                Ok(None) => break,
                Err(..) => TypeDefinition::synchronize(parser),
            }
        }

        if parser.check(CBrace).is_ok() && !parser.is_line_start {
            let error = SyntaxError::new(
                parser.peek().span,
                "definition in type must end with a line break",
            );
            parser.error(error);
        }

        let close_brace = parser.expect(CBrace).map_err(|token| {
            parser.expected(token, "expected } to close_brace a local type definition")
        })?;
        span = span.union(close_brace.span);

        Ok(Self {
            span,
            head,
            open_brace,
            definitions,
            close_brace,
        })
    }
}
