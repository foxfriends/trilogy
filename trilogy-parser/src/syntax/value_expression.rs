use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
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
pub(crate) enum Precedence {
    Primary,
    Path,
    Access,
    Call,
    Unary,
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
    fn binary(parser: &mut Parser, lhs: ValueExpression) -> SyntaxResult<Result<Self, Self>> {
        BinaryOperation::parse(parser, lhs)
            .map(Box::new)
            .map(Self::Binary)
            .map(Ok)
    }

    fn parse_follow(
        parser: &mut Parser,
        precedence: Precedence,
        lhs: ValueExpression,
    ) -> SyntaxResult<Result<Self, Self>> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            OpColonColon if precedence < Precedence::Path => todo!("Module path"),
            OParen if precedence < Precedence::Path => todo!("Module params"),
            OpDot if precedence < Precedence::Access => todo!(),
            KwAnd if precedence < Precedence::And => Self::binary(parser, lhs),
            KwOr if precedence < Precedence::Or => {
                let op = parser.expect(KwOr).unwrap();
                let rhs = Self::parse_precedence(parser, Precedence::Or)?;
                Ok(Ok(Self::Binary(Box::new(BinaryOperation {
                    lhs: lhs.into(),
                    operator: BinaryOperator::Or(op),
                    rhs: rhs.into(),
                }))))
            }
            OpPlus | OpMinus if precedence < Precedence::Term => Self::binary(parser, lhs),
            OpStar | OpSlash | OpPercent | OpSlashSlash if precedence < Precedence::Factor => {
                Self::binary(parser, lhs)
            }
            OpStarStar if precedence <= Precedence::Exponent => Self::binary(parser, lhs),
            OpLt | OpGt | OpGtEq | OpGtEq if precedence == Precedence::Comparison => {
                let expr = Self::binary(parser, lhs);
                if let Ok(Ok(expr)) = &expr {
                    parser.error(SyntaxError::new(
                        expr.span(),
                        "comparison operators cannot be chained, use parentheses to disambiguate",
                    ));
                }
                expr
            }
            OpLt | OpGt | OpGtEq | OpGtEq if precedence < Precedence::Comparison => {
                Self::binary(parser, lhs)
            }
            OpEqEq | OpEqEqEq if precedence == Precedence::Equality => {
                let expr = Self::binary(parser, lhs);
                if let Ok(Ok(expr)) = &expr {
                    parser.error(SyntaxError::new(
                        expr.span(),
                        "comparison operators cannot be chained, use parentheses to disambiguate",
                    ));
                }
                expr
            }
            OpEqEq | OpEqEqEq if precedence < Precedence::Equality => Self::binary(parser, lhs),
            OpAmp if precedence < Precedence::BitwiseAnd => Self::binary(parser, lhs),
            OpPipe if precedence < Precedence::BitwiseOr => Self::binary(parser, lhs),
            OpCaret if precedence < Precedence::BitwiseXor => Self::binary(parser, lhs),
            OpShr | OpShl if precedence < Precedence::BitwiseShift => Self::binary(parser, lhs),
            OpColon if precedence <= Precedence::Cons => Self::binary(parser, lhs),
            OpSemi if precedence < Precedence::Sequence => Self::binary(parser, lhs),
            OpLtLt if precedence < Precedence::Compose => Self::binary(parser, lhs),
            OpGtGt if precedence < Precedence::RCompose => Self::binary(parser, lhs),
            OpPipeGt if precedence < Precedence::Pipe => Self::binary(parser, lhs),
            OpLtPipe if precedence <= Precedence::RPipe => Self::binary(parser, lhs),
            OpGlue if precedence <= Precedence::Glue => Self::binary(parser, lhs),
            KwWhen => todo!(),
            KwGiven => todo!(),
            // If nothing matched, it must be the end of the expression
            _ => Ok(Err(lhs)),
        }
    }

    fn parse_prefix(parser: &mut Parser) -> SyntaxResult<Self> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
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
            KwUnit => Ok(Self::Unit(Box::new(UnitLiteral::parse(parser)?))),
            OBrack => todo!("Array + Comp"),
            OBracePipe => todo!("Set + Comp"),
            OBrace => todo!("Record + Comp"),
            DollarOParen => todo!("Iter Comp"),
            KwNot | OpMinus | OpTilde | KwYield => {
                Ok(Self::Unary(Box::new(UnaryOperation::parse(parser)?)))
            }
            KwIf => todo!("Conditional"),
            KwMatch => todo!("Match"),
            KwEnd => todo!("End"),
            KwReturn => todo!("Return"),
            KwResume => todo!("Resume"),
            KwBreak => todo!("Break"),
            KwContinue => todo!("Continue"),
            KwCancel => todo!("Cancel"),
            Identifier => todo!("Variable"),
            IdentifierBang => todo!("Procedure call"),
            KwFn => todo!("Fn expression"),
            KwDo => todo!("Do expression"),
            TemplateStart => todo!("Template"),
            OParen => Ok(Self::Parenthesized(Box::new(
                ParenthesizedExpression::parse(parser)?,
            ))),
            _ => Err(SyntaxError::new(
                token.span,
                "Unexpected token in expression",
            )),
        }
    }

    pub(crate) fn parse_precedence(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Self> {
        let mut expr = Self::parse_prefix(parser)?;
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
