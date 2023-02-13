use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Statement {
    Let(Box<LetStatement>),
    Assignment(Box<AssignmentStatement>),
    If(Box<IfStatement>),
    Match(Box<MatchStatement>),
    While(Box<WhileStatement>),
    For(Box<ForStatement>),
    Break(Box<BreakStatement>),
    Continue(Box<ContinueStatement>),
    Resume(Box<ResumeStatement>),
    Cancel(Box<CancelStatement>),
    Return(Box<ReturnStatement>),
    End(Box<EndStatement>),
    Exit(Box<ExitStatement>),
    Yield(Box<YieldStatement>),
    Call(Box<CallStatement>),
    Expression(Box<Expression>),
    Assert(Box<AssertStatement>),
    Handled(Box<HandledBlock>),
    Block(Box<Block>),
}

impl Statement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser.peek();
        use TokenType::*;
        match token.token_type {
            KwLet => Ok(Self::Let(Box::new(LetStatement::parse(parser)?))),
            KwIf => Ok(Self::If(Box::new(IfStatement::parse(parser)?))),
            KwMatch => todo!(),
            KwWhile => todo!(),
            KwFor => todo!(),
            KwBreak => todo!(),
            KwContinue => todo!(),
            KwResume => todo!(),
            KwCancel => todo!(),
            KwReturn => todo!(),
            KwEnd => todo!(),
            KwExit => todo!(),
            KwYield => todo!(),
            KwAssert => todo!(),
            OBrace => Ok(Self::Block(Box::new(Block::parse(parser)?))),
            _ => {
                // TODO: this is probably going to be come expression statement/assignment/call,
                // why do I require them parenthesized? Just let any unambiguous expressions be used.
                let error = SyntaxError::new(token.span, "unexpected token in statement");
                parser.error(error.clone());
                Err(error)
            }
        }
    }
}
