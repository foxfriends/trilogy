use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned)]
pub enum Statement {
    Let(Box<LetStatement>),
    Assignment(Box<AssignmentStatement>),
    FunctionAssignment(Box<FunctionAssignment>),
    If(Box<IfElseExpression>),
    Match(Box<MatchExpression>),
    While(Box<WhileStatement>),
    For(Box<ForStatement>),
    Defer(Box<DeferStatement>),
    Using(Box<UsingStatement>),
    Assert(Box<AssertStatement>),
    Expression(Box<Expression>),
    Block(Box<Block>),
}

impl Statement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser.peek();
        use TokenType::*;
        match token.token_type {
            KwLet => Ok(Self::Let(Box::new(LetStatement::parse(parser)?))),
            KwIf => {
                let expr = IfElseExpression::parse(parser)?;
                Ok(Self::If(Box::new(expr)))
            }
            KwMatch => Ok(Self::Match(Box::new(MatchExpression::parse(parser)?))),
            KwWhile => Ok(Self::While(Box::new(WhileStatement::parse(parser)?))),
            KwFor => Ok(Self::For(Box::new(ForStatement::parse(parser)?))),
            KwDefer => Ok(Self::Defer(Box::new(DeferStatement::parse(parser)?))),
            KwAssert => Ok(Self::Assert(Box::new(AssertStatement::parse(parser)?))),
            OBrace => Ok(Self::Block(Box::new(Block::parse(parser)?))),
            KwUsing => Ok(Self::Using(Box::new(UsingStatement::parse(parser, None)?))),
            KwDo => {
                let head = DoHead::parse(parser)?;
                if parser.check(KwUsing).is_ok() {
                    Ok(Self::Using(Box::new(UsingStatement::parse(
                        parser,
                        Some(head),
                    )?)))
                } else {
                    let procedure = DoExpression::parse_with_head(parser, head)?;
                    let expression = Expression::Do(Box::new(procedure));
                    Ok(Self::Expression(Box::new(expression)))
                }
            }
            _ => {
                let expression = Expression::parse(parser)?;
                if parser.check(IdentifierEq).is_ok() {
                    if !expression.is_lvalue() {
                        parser.error(SyntaxError::new(
                            expression.span(),
                            "cannot assign to an expression that is not a valid assignment target",
                        ));
                    }
                    Ok(Self::FunctionAssignment(Box::new(
                        FunctionAssignment::parse(parser, expression)?,
                    )))
                } else if parser
                    .check(AssignmentStatement::ASSIGNMENT_OPERATOR)
                    .is_ok()
                {
                    if !expression.is_lvalue() {
                        parser.error(SyntaxError::new(
                            expression.span(),
                            "cannot assign to an expression that is not a valid assignment target",
                        ));
                    }
                    Ok(Self::Assignment(Box::new(AssignmentStatement::parse(
                        parser, expression,
                    )?)))
                } else {
                    Ok(Self::Expression(Box::new(expression)))
                }
            }
        }
    }
}
