use crate::format::{PrettyPrint, PrettyPrinted, PrettyPrinter};
use crate::Parser;
use pretty::DocAllocator;
use trilogy_scanner::Token;
use trilogy_scanner::TokenType::{self, DocInner, DocOuter};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
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

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(documentation_inner: "#! hello\n" => Documentation::parse_inner => "(Documentation)");
    test_parse!(documentation_inner_multiline: "#! hello\n#! world\n" => Documentation::parse_inner => "(Documentation)");
    test_parse!(documentation_inner_gaps: "#! hello\n\n#! world\n" => Documentation::parse_inner => "(Documentation)");

    test_parse!(documentation_outer: "## hello\n" => Documentation::parse_outer => "(Documentation)");
    test_parse!(documentation_outer_multiline: "## hello\n## world\n" => Documentation::parse_outer => "(Documentation)");
    test_parse!(documentation_outer_gaps: "## hello\n\n## world\n" => Documentation::parse_outer => "(Documentation)");
}
