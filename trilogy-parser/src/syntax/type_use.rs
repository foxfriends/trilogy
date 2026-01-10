use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// The `use` portion of a module definition.
#[derive(Clone, Debug)]
pub struct TypeUse {
    /// The `use` token
    pub r#use: Token,
    /// The imported names
    pub names: Punctuated<ImportedName>,
    pub span: Span,
}

impl Spanned for TypeUse {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Spanned)]
pub enum ImportedName {
    Named(Identifier),
    Aliased(AliasedName),
}

impl ImportedName {
    pub fn original_name(&self) -> &Identifier {
        match self {
            Self::Named(name) => name,
            Self::Aliased(name) => &name.original,
        }
    }

    pub fn aliased_name(&self) -> &Identifier {
        match self {
            Self::Named(name) => name,
            Self::Aliased(name) => &name.aliased,
        }
    }
}

#[derive(Clone, Debug, Spanned)]
pub struct AliasedName {
    pub original: Identifier,
    pub r#as: Token,
    pub aliased: Identifier,
}

impl TypeUse {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#use = parser
            .expect(TokenType::KwUse)
            .expect("caller should find `use` keyword");

        let mut names = Punctuated::new();
        loop {
            let original = Identifier::parse(parser)?;
            let name = if let Ok(r#as) = parser.expect(TokenType::KwAs) {
                let aliased = Identifier::parse(parser)?;
                ImportedName::Aliased(AliasedName {
                    original,
                    r#as,
                    aliased,
                })
            } else {
                ImportedName::Named(original)
            };
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
