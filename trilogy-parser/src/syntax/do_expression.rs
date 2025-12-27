use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A procedure closure `do` expression.
///
/// ```trilogy
/// do() {}
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct DoExpression {
    pub head: DoHead,
    pub body: DoBody,
    span: Span,
}

impl Spanned for DoExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl DoExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let head = DoHead::parse(parser)?;
        Self::parse_with_head(parser, head)
    }

    pub(crate) fn parse_with_head(parser: &mut Parser, head: DoHead) -> SyntaxResult<Self> {
        if head.parameter_list.is_none() {
            parser.error(ErrorKind::DoMissingParameterList.at(head.span()));
        }
        let body = DoBody::parse(parser)?;
        Ok(Self {
            span: head.span().union(body.span()),
            head,
            body,
        })
    }

    pub fn r#do(&self) -> &Token {
        &self.head.r#do
    }
}

/// The body of a procedure closure.
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum DoBody {
    /// A block used as the body of a `do` closure.
    ///
    /// Returns unit if no explicit `return` statement is reached.
    Block(Box<Block>),
    /// A single expression used as the body and return value of a `do` closure.
    Expression(Box<Expression>),
}

impl DoBody {
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

    test_parse!(do_block: "do() {}" => DoExpression::parse => "(DoExpression _ _ [] _ (DoBody::Block _))");
    test_parse_error!(do_block_no_params: "do {}" => DoExpression::parse);
    test_parse!(do_block_params: "do(a, b) {}" => DoExpression::parse => "(DoExpression _ _ [_ _] _ (DoBody::Block _))");
    test_parse!(do_block_params_trailing_comma: "do(a, b, ) {}" => DoExpression::parse => "(DoExpression _ _ [_ _] _ (DoBody::Block _))");
    test_parse_error!(do_block_params_leading_comma: "do(, a) {}" => DoExpression::parse);
    test_parse_error!(do_block_params_empty_comma: "do(,) {}" => DoExpression::parse);
    test_parse_error!(do_block_missing_paren: "do(a {}" => DoExpression::parse => "expected `)` to end parameter list");
    test_parse_error!(do_block_invalid: "do() { exit }" => DoExpression::parse);

    test_parse!(do_expr_spaced: "do () 3" => DoExpression::parse => "(DoExpression _ _ [] _ (DoBody::Expression (Expression::Number _)))");
    test_parse_error!(do_expr_bang: "do!() 3" => DoExpression::parse => "expected `(` to begin parameter list");
    test_parse_error!(do_no_parens: "do 3" => DoExpression::parse => "expected `(` to begin parameter list");

    test_parse!(do_expr: "do() 3" => DoExpression::parse => "(DoExpression _ _ [] _ (DoBody::Expression (Expression::Number _)))");
    test_parse!(do_expr_params: "do(a, b) a + b" => DoExpression::parse => "(DoExpression _ _ [_ _] _ (DoBody::Expression (Expression::Binary _)))");
    test_parse!(do_expr_params_trailing_comma: "do(a, b, ) a + b" => DoExpression::parse => "(DoExpression _ _ [_ _] _ (DoBody::Expression (Expression::Binary _)))");
    test_parse_error!(do_expr_params_leading_comma: "do(, a) a" => DoExpression::parse);
    test_parse_error!(do_expr_params_empty_comma: "do(,) 3" => DoExpression::parse);
    test_parse_error!(do_expr_invalid: "do() return" => DoExpression::parse);
}
