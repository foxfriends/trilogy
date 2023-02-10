use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleReference {
    pub name: Identifier,
    pub arguments: Option<ModuleArguments>,
}

impl ModuleReference {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        let arguments = ModuleArguments::parse(parser)?;
        Ok(Self { name, arguments })
    }

    pub(crate) fn parse_arguments(mut self, parser: &mut Parser) -> SyntaxResult<Self> {
        parser
            .check(TokenType::OParen)
            .expect("caller should have found this");
        let valid = !parser.is_spaced();
        let arguments = ModuleArguments::force_parse(parser)?;
        if !valid {
            parser.error(SyntaxError::new(
                self.span().union(arguments.span()),
                "module arguments may not be spaced",
            ));
        }
        self.arguments = Some(arguments);
        Ok(self)
    }
}

impl Spanned for ModuleReference {
    fn span(&self) -> Span {
        match &self.arguments {
            Some(arguments) => self.name.span().union(arguments.span()),
            None => self.name.span(),
        }
    }
}

impl From<Identifier> for ModuleReference {
    fn from(name: Identifier) -> Self {
        Self {
            name,
            arguments: None,
        }
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleArguments {
    start: Token,
    pub arguments: Vec<ModuleReference>,
    end: Token,
}

impl ModuleArguments {
    fn parse(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        // There may be no space between a module and its arguments, as a space
        // is used in function application; a parenthesized parameter to a function
        // may otherwise be ambiguous. A bit of a hack, but I think we'll survive.
        if parser.check(TokenType::OParen).is_none() || parser.is_spaced() {
            return Ok(None);
        }
        ModuleArguments::force_parse(parser).map(Some)
    }

    fn force_parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(TokenType::OParen).unwrap();

        let mut arguments = vec![];
        loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Self {
                    start,
                    arguments,
                    end,
                });
            }
            arguments.push(ModuleReference::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            if let Ok(end) = parser.expect(TokenType::CParen) {
                return Ok(Self {
                    start,
                    arguments,
                    end,
                });
            }
        }
    }
}

impl Spanned for ModuleArguments {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}
