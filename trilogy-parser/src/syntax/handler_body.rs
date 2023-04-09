use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum HandlerBody {
    Block(Box<Block>),
    Expression(Box<Expression>),
}

impl HandlerBody {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if parser.check(OBrace).is_ok() {
            Ok(Self::Block(Box::new(Block::parse(parser)?)))
        } else {
            Ok(Self::Expression(Box::new(Expression::parse_precedence(
                parser,
                Precedence::Continuation,
            )?)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(handlerbody_block: "{ let x = 5; resume x }" => HandlerBody::parse => "(HandlerBody::Block (Block _))");
    test_parse!(handlerbody_expr: "let y = 5, resume y" => HandlerBody::parse => "(HandlerBody::Expression _)");
}
