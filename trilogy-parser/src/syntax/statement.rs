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
            KwMatch => Ok(Self::Match(Box::new(MatchStatement::parse(parser)?))),
            KwWhile => Ok(Self::While(Box::new(WhileStatement::parse(parser)?))),
            KwFor => Ok(Self::For(Box::new(ForStatement::parse(parser)?))),
            KwBreak => Ok(Self::Break(Box::new(BreakStatement::parse(parser)?))),
            KwContinue => Ok(Self::Continue(Box::new(ContinueStatement::parse(parser)?))),
            KwResume => Ok(Self::Resume(Box::new(ResumeStatement::parse(parser)?))),
            KwCancel => Ok(Self::Cancel(Box::new(CancelStatement::parse(parser)?))),
            KwReturn => Ok(Self::Return(Box::new(ReturnStatement::parse(parser)?))),
            KwEnd => Ok(Self::End(Box::new(EndStatement::parse(parser)?))),
            KwExit => Ok(Self::Exit(Box::new(ExitStatement::parse(parser)?))),
            KwYield => Ok(Self::Yield(Box::new(YieldStatement::parse(parser)?))),
            KwAssert => Ok(Self::Assert(Box::new(AssertStatement::parse(parser)?))),
            KwWhen => Ok(Self::Handled(Box::new(HandledBlock::parse(parser)?))),
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
