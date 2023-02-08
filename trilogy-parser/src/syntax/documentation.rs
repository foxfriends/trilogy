use crate::{
    format::{PrettyPrint, PrettyPrinted, PrettyPrinter},
    Parser,
};
use pretty::DocAllocator;
use trilogy_scanner::{
    Token,
    TokenType::{self, DocInner, DocOuter},
};

#[derive(Clone, Debug, Spanned)]
pub struct Documentation {
    tokens: Vec<Token>,
}

impl Documentation {
    fn parse(parser: &mut Parser, token_type: TokenType) -> Option<Self> {
        let mut tokens = vec![];

        while let Ok(token) = parser.expect(token_type) {
            tokens.push(token);
        }
        if tokens.is_empty() {
            return None;
        }

        Some(Self { tokens })
    }

    pub(crate) fn parse_inner(parser: &mut Parser) -> Option<Self> {
        Self::parse(parser, DocInner)
    }

    pub(crate) fn parse_outer(parser: &mut Parser) -> Option<Self> {
        Self::parse(parser, DocOuter)
    }
}

impl<'a> PrettyPrint<'a> for Documentation {
    fn pretty_print(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a> {
        let lines = self.tokens.iter().map(|token| {
            let prefix = match token.token_type {
                DocInner => "#! ",
                DocOuter => "## ",
                _ => unreachable!("Documentation has wrong token type"),
            };
            printer.text(prefix).append(
                printer.text(
                    token
                        .value
                        .as_ref()
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .trim()
                        .to_owned(),
                ),
            )
        });
        printer.intersperse(lines, printer.hardline())
    }
}
