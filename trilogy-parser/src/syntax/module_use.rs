use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// The `use` portion of a module definition.
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ModuleUse {
    /// The `use` token
    pub r#use: Token,
    /// The imported names
    pub names: Punctuated<Identifier>,
    span: Span,
}

impl Spanned for ModuleUse {
    fn span(&self) -> Span {
        self.span
    }
}

impl ModuleUse {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#use = parser
            .expect(TokenType::KwUse)
            .expect("caller should find `use` keyword");

        let mut names = Punctuated::new();
        loop {
            let name = Identifier::parse(parser)?;
            if let Ok(comma) = parser.expect(TokenType::OpComma) {
                names.push(name, comma);
                continue;
            }
            names.push_last(name);
            break;
        }
        Ok(Self {
            span: r#use.span.union(names.span()),
            r#use,
            names,
        })
    }
}
