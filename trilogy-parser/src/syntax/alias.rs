use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, PrettyPrintSExpr, Spanned)]
pub enum Alias {
    Same(Identifier),
    Rename(Identifier, Identifier),
}

impl Alias {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let identifier = Identifier::parse(parser)?;
        if parser.expect(TokenType::KwAs).is_ok() {
            Ok(Self::Rename(identifier, Identifier::parse(parser)?))
        } else {
            Ok(Self::Same(identifier))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(alias_same: "hello" => Alias::parse => "(Alias::Same (Identifier))");
    test_parse!(alias_rename: "hello as goodbye" => Alias::parse => "(Alias::Rename (Identifier) (Identifier))");
    test_parse_error!(alias_first_not_ident: "3 as goodbye" => Alias::parse);
    test_parse_error!(alias_second_not_ident: "hello as 3" => Alias::parse);
}
