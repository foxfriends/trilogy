use crate::Parser;
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
