use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleDefinition {
    pub head: ModuleHead,
    pub obrace: Token,
    pub definitions: Vec<Definition>,
    pub cbrace: Token,
    pub module_use: Option<ModuleUse>,
}

impl Spanned for ModuleDefinition {
    fn span(&self) -> source_span::Span {
        match &self.module_use {
            Some(uses) => self.head.span().union(uses.span()),
            None => self.head.span().union(self.cbrace.span),
        }
    }
}

impl ModuleDefinition {
    fn synchronize(parser: &mut Parser) {
        parser.synchronize([
            DocOuter, KwModule, KwFunc, KwProc, KwRule, KwConst, KwExport, CBrace, EndOfFile,
        ]);
    }

    pub(crate) fn parse(parser: &mut Parser, head: ModuleHead) -> SyntaxResult<Self> {
        let obrace = parser.expect(OBrace).expect("Caller should find `{`.");

        if let Ok(cbrace) = parser.expect(CBrace) {
            // empty module may be single line
            return Ok(Self {
                head,
                obrace,
                definitions: vec![],
                cbrace,
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

        let cbrace = parser.expect(CBrace).map_err(|token| {
            parser.expected(token, "expected } to cbrace a local module definition")
        })?;

        let module_use = if parser.check(KwUse).is_ok() {
            Some(ModuleUse::parse(parser)?)
        } else {
            None
        };

        Ok(Self {
            head,
            obrace,
            definitions,
            cbrace,
            module_use,
        })
    }
}
