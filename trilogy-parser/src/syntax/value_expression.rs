use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned)]
pub enum ValueExpression {
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
    Bits(Box<BitsLiteral>),
    Boolean(Box<BooleanLiteral>),
    Unit(Box<UnitLiteral>),
    Atom(Box<AtomLiteral>),
    Struct(Box<StructLiteral>),
    Array(Box<ArrayLiteral>),
    Set(Box<SetLiteral>),
    Record(Box<RecordLiteral>),
    ArrayComprehension(Box<ArrayComprehension>),
    SetComprehension(Box<SetComprehension>),
    RecordComprehension(Box<RecordComprehension>),
    IteratorComprehension(Box<IteratorComprehension>),
    MemberAccess(Box<MemberAccess>),
    Keyword(Box<KeywordReference>),
    Application(Box<Application>),
    Call(Box<CallExpression>),
    Binary(Box<BinaryOperation>),
    Unary(Box<UnaryOperation>),
    Let(Box<LetExpression>),
    IfElse(Box<IfElseExpression>),
    Match(Box<MatchExpression>),
    Is(Box<IsExpression>),
    End(Box<EndExpression>),
    Resume(Box<ResumeExpression>),
    Cancel(Box<CancelExpression>),
    Return(Box<ReturnExpression>),
    Break(Box<BreakExpression>),
    Continue(Box<ContinueExpression>),
    Fn(Box<FnExpression>),
    Do(Box<DoExpression>),
    Template(Box<Template>),
    Parenthesized(Box<ParenthesizedExpression>),
    SyntaxError(Box<SyntaxError>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum Precedence {
    Primary,
    Access,
    Unary,
    Call,
    Application,
    Compose,
    RCompose,
    Exponent,
    Factor,
    Term,
    BitwiseAnd,
    BitwiseShift,
    BitwiseXor,
    BitwiseOr,
    Glue,
    Cons,
    Comparison,
    Equality,
    And,
    Or,
    Pipe,
    RPipe,
    Handler,
    Conditional,
    Match,
    Yield,
    Sequence,
    Continuation,
    None,
}

impl ValueExpression {
    fn parse_follow(
        parser: &mut Parser,
        _precedence: Precedence,
        _lhs: ValueExpression,
    ) -> SyntaxResult<Result<Self, Self>> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            KwAnd => todo!(),
            KwOr => todo!(),
            OpPlus => todo!(),
            OpMinus => todo!(),
            OpStar => todo!(),
            OpSlash => todo!(),
            OpPercent => todo!(),
            OpSlashSlash => todo!(),
            OpStarStar => todo!(),
            OpLt => todo!(),
            OpGt => todo!(),
            OpAmp => todo!(),
            OpPipe => todo!(),
            OpCaret => todo!(),
            OpTilde => todo!(),
            OpShr => todo!(),
            OpShl => todo!(),
            OpAt => todo!(),
            OpDot => todo!(),
            OpComma => todo!(),
            OpColon => todo!(),
            OpLeftArrow => todo!(),
            OpSemi => todo!(),
            OpLtLt => todo!(),
            OpGtGt => todo!(),
            OpPipeGt => todo!(),
            OpLtPipe => todo!(),
            OpGlue => todo!(),
            KwWhen => todo!(),
            KwGiven => todo!(),
            OpColonColon => todo!(),
            _ => {
                todo!("Most expressions are not supported yet")
            }
        }
    }

    fn parse_prefix(parser: &mut Parser, _precedence: Precedence) -> SyntaxResult<Self> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            // prefix
            Numeric => Ok(Self::Number(Box::new(NumberLiteral::parse(parser)?))),
            String => Ok(Self::String(Box::new(StringLiteral::parse(parser)?))),
            Bits => Ok(Self::Bits(Box::new(BitsLiteral::parse(parser)?))),
            KwTrue | KwFalse => Ok(Self::Boolean(Box::new(BooleanLiteral::parse(parser)?))),
            Atom => {
                let atom = AtomLiteral::parse(parser)?;
                if parser.check(OParen).is_some() {
                    Ok(Self::Struct(Box::new(StructLiteral::parse(parser, atom)?)))
                } else {
                    Ok(Self::Atom(Box::new(atom)))
                }
            }
            Character => Ok(Self::Character(Box::new(CharacterLiteral::parse(parser)?))),
            OBrack => todo!("Array + Comp"),
            OBracePipe => todo!("Set + Comp"),
            OBrace => todo!("Record + Comp"),
            DollarOParen => todo!("Iter Comp"),
            KwUnit => Ok(Self::Unit(Box::new(UnitLiteral::parse(parser)?))),
            KwNot => todo!("Unary Not"),
            OpMinus => todo!("Unary Minus"),
            OpTilde => todo!("Unary Invert"),
            KwIf => todo!("Conditional"),
            KwMatch => todo!("Match"),
            KwEnd => todo!("End"),
            KwReturn => todo!("Return"),
            KwResume => todo!("Resume"),
            KwBreak => todo!("Break"),
            KwContinue => todo!("Continue"),
            KwCancel => todo!("Cancel"),
            KwYield => todo!("Yield"),
            Identifier => todo!("Variable"),
            IdentifierBang => todo!("Procedure call"),
            KwFn => todo!("Fn expression"),
            KwDo => todo!("Do expression"),
            TemplateStart => todo!("Template"),
            OParen => Ok(Self::Parenthesized(Box::new(
                ParenthesizedExpression::parse(parser)?,
            ))),
            _ => {
                todo!("Most expressions are not supported yet")
            }
        }
    }

    fn parse_infix(_parser: &mut Parser, _precedence: Precedence) -> SyntaxResult<Self> {
        todo!()
    }

    fn parse_precedence(parser: &mut Parser, precedence: Precedence) -> SyntaxResult<Self> {
        let mut expr = Self::parse_prefix(parser, precedence)?;
        loop {
            match Self::parse_follow(parser, precedence, expr)? {
                Ok(updated) => expr = updated,
                Err(expr) => return Ok(expr),
            }
        }
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        Self::parse_precedence(parser, Precedence::Primary)
    }
}
