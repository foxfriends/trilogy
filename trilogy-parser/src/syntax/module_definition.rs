use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleDefinition {
    pub head: ModuleHead,
    pub open_brace: Token,
    pub definitions: Vec<Definition>,
    pub close_brace: Token,
    pub module_use: Option<ModuleUse>,
    span: Span,
}

impl Spanned for ModuleDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl ModuleDefinition {
    fn synchronize(parser: &mut Parser) {
        parser.synchronize([
            DocOuter, KwModule, KwFunc, KwProc, KwRule, KwConst, KwExport, CBrace, EndOfFile,
        ]);
    }

    pub(crate) fn parse(parser: &mut Parser, head: ModuleHead) -> SyntaxResult<Self> {
        let open_brace = parser.expect(OBrace).unwrap();

        let mut span = head.span();

        if let Ok(close_brace) = parser.expect(CBrace) {
            // empty module may be single line
            return Ok(Self {
                span: span.union(close_brace.span),
                head,
                open_brace,
                definitions: vec![],
                close_brace,
                // Empty module does not export anything, so cannot have anything used
                module_use: None,
            });
        }

        let mut definitions = vec![];
        loop {
            match Definition::parse_in_module(parser) {
                Ok(Some(definition)) => definitions.push(definition),
                Ok(None) => break,
                Err(..) => ModuleDefinition::synchronize(parser),
            }
        }

        if parser.check(CBrace).is_ok() && !parser.is_line_start {
            let error = SyntaxError::new(
                parser.peek().span,
                "definition in module must end with a line break",
            );
            parser.error(error);
        }

        let close_brace = parser.expect(CBrace).map_err(|token| {
            parser.expected(token, "expected } to close_brace a local module definition")
        })?;

        let module_use = if parser.check(KwUse).is_ok() {
            let usage = ModuleUse::parse(parser)?;
            span = span.union(usage.span());
            Some(usage)
        } else {
            span = span.union(close_brace.span);
            None
        };

        Ok(Self {
            span,
            head,
            open_brace,
            definitions,
            close_brace,
            module_use,
        })
    }
}
