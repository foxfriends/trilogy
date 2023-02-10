use super::*;

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
    Expression(Box<ParenthesizedExpression>),
    Assert(Box<AssertStatement>),
    Block(Box<Block>),
    SyntaxError(Box<SyntaxError>),
}
