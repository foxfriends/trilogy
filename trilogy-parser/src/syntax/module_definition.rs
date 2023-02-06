use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct ModuleDefinition {
    pub head: ModuleHead,
    pub definitions: Vec<Definition>,
    end: Token,
}

impl ModuleDefinition {
    fn synchronize(parser: &mut Parser) {
        parser.synchronize([
            DocOuter, KwModule, KwFunc, KwProc, KwRule, KwImport, KwExport, CBrace, EndOfFile,
        ]);
    }

    pub(crate) fn parse(parser: &mut Parser, head: ModuleHead) -> SyntaxResult<Self> {
        parser.expect(OBrace).expect("Caller should find `{`.");

        let mut definitions = vec![];
        loop {
            match Definition::parse_in_module(parser) {
                Ok(Some(definition)) => definitions.push(definition),
                Ok(None) => break,
                Err(..) => ModuleDefinition::synchronize(parser),
            }
        }

        let end = parser.expect(CBrace).map_err(|token| {
            parser.expected(token, "expected } to end a local module definition")
        })?;

        Ok(Self {
            head,
            definitions,
            end,
        })
    }
}
