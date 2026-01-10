use super::*;
use crate::{Parser, Spanned, token_pattern::TokenPattern};
use become_expression::BecomeExpression;
use trilogy_scanner::{Token, TokenType};

/// The many kinds of expressions in a Trilogy program.
#[derive(Clone, Debug, Spanned)]
pub enum Expression {
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
    Reference(Box<super::Identifier>),
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
    Exit(Box<ExitExpression>),
    Resume(Box<ResumeExpression>),
    Become(Box<BecomeExpression>),
    Cancel(Box<CancelExpression>),
    Return(Box<ReturnExpression>),
    Break(Box<BreakExpression>),
    Continue(Box<ContinueExpression>),
    Fn(Box<FnExpression>),
    Do(Box<DoExpression>),
    Qy(Box<QyExpression>),
    Template(Box<Template>),
    Handled(Box<HandledExpression>),
    Parenthesized(Box<ParenthesizedExpression>),
    ModuleAccess(Box<ModuleAccess>),
    Block(Box<Block>),
}

#[derive(Clone, Debug, Spanned)]
pub enum FollowingExpression {
    Then(Token, Expression),
    Block(Block),
}

impl FollowingExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if let Ok(then) = parser.expect(TokenType::KwThen) {
            let body = Expression::parse(parser)?;
            Ok(Self::Then(then, body))
        } else {
            Ok(Self::Block(Block::parse(parser)?))
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum Precedence {
    None,
    Continuation,
    Or,
    And,
    Equality,
    Comparison,
    Pipe,
    RPipe,
    Cons,
    Glue,
    BitwiseOr,
    BitwiseXor,
    BitwiseShift,
    BitwiseAnd,
    Term,
    Factor,
    Exponent,
    RCompose,
    Compose,
    Application,
    Unary,
    Call,
    Access,
    Path,
}

enum ExpressionResult {
    Continue(Expression),
    Done(Expression),
    Pattern(Pattern),
}

impl Expression {
    pub(crate) const PREFIX: [TokenType; 33] = [
        TokenType::Numeric,
        TokenType::String,
        TokenType::Bits,
        TokenType::KwTrue,
        TokenType::KwFalse,
        TokenType::Atom,
        TokenType::Character,
        TokenType::KwUnit,
        TokenType::OBrack,
        TokenType::OBrackPipe,
        TokenType::OBracePipe,
        TokenType::OpBang,
        TokenType::OpTilde,
        TokenType::KwYield,
        TokenType::KwIf,
        TokenType::KwIs,
        TokenType::KwMatch,
        TokenType::KwEnd,
        TokenType::KwExit,
        TokenType::KwReturn,
        TokenType::KwResume,
        TokenType::KwBreak,
        TokenType::KwContinue,
        TokenType::KwCancel,
        TokenType::KwLet,
        TokenType::Identifier,
        TokenType::KwWith,
        TokenType::KwFn,
        TokenType::KwDo,
        TokenType::KwQy,
        TokenType::OpDollar,
        TokenType::TemplateStart,
        TokenType::OParen,
    ];

    fn binary(parser: &mut Parser, lhs: Expression) -> SyntaxResult<ExpressionResult> {
        let binary = BinaryOperation::parse(parser, lhs)?;
        match binary {
            Ok(binary) => Ok(ExpressionResult::Continue(Self::Binary(Box::new(binary)))),
            Err(pattern) => Ok(ExpressionResult::Pattern(pattern)),
        }
    }

    fn parse_follow(
        parser: &mut Parser,
        precedence: Precedence,
        lhs: Expression,
    ) -> SyntaxResult<ExpressionResult> {
        // A bit of strangeness here because `peek()` takes a mutable reference,
        // so we can't use the fields afterwards... but we need their value after
        // and I'd really rather not clone every token of every expression, so
        // instead we peek first, then do a force peek after (to skip an extra chomp).
        parser.peek();
        let is_spaced = parser.is_spaced;
        let is_line_start = parser.is_line_start;
        let token = parser.force_peek();

        use ExpressionResult::{Continue, Done};
        use TokenType::*;

        match token.token_type {
            OpDot if precedence < Precedence::Access => Self::binary(parser, lhs),
            OpAmpAmp if precedence < Precedence::And => Self::binary(parser, lhs),
            OpPipePipe if precedence < Precedence::Or => Self::binary(parser, lhs),
            OpPlus | OpMinus if precedence < Precedence::Term => Self::binary(parser, lhs),
            OpStar | OpSlash | OpPercent | OpSlashSlash if precedence < Precedence::Factor => {
                Self::binary(parser, lhs)
            }
            OpStarStar if precedence <= Precedence::Exponent => Self::binary(parser, lhs),
            OpLt | OpGt | OpLtEq | OpGtEq if precedence == Precedence::Comparison => {
                let expr = Self::binary(parser, lhs);
                if let Ok(Continue(expr)) = &expr {
                    parser.error(SyntaxError::new(
                        expr.span(),
                        "comparison operators cannot be chained, use parentheses to disambiguate",
                    ));
                }
                expr
            }
            OpLt | OpGt | OpLtEq | OpGtEq if precedence < Precedence::Comparison => {
                Self::binary(parser, lhs)
            }
            OpEqEq | OpEqEqEq if precedence == Precedence::Equality => {
                let expr = Self::binary(parser, lhs);
                if let Ok(Continue(expr)) = &expr {
                    parser.error(SyntaxError::new(
                        expr.span(),
                        "equality operators cannot be chained, use parentheses to disambiguate",
                    ));
                }
                expr
            }
            OpBangEq | OpBangEqEq | OpEqEq | OpEqEqEq if precedence < Precedence::Equality => {
                Self::binary(parser, lhs)
            }
            OpAmp if precedence < Precedence::BitwiseAnd => Self::binary(parser, lhs),
            OpPipe if precedence < Precedence::BitwiseOr => Self::binary(parser, lhs),
            OpCaret if precedence < Precedence::BitwiseXor => Self::binary(parser, lhs),
            OpShr | OpShl | OpShrEx | OpShlEx | OpShrCon | OpShlCon
                if precedence < Precedence::BitwiseShift =>
            {
                Self::binary(parser, lhs)
            }
            OpColonColon if precedence < Precedence::Path => Ok(Continue(Self::ModuleAccess(
                Box::new(ModuleAccess::parse(parser, lhs)?),
            ))),
            OpColon if precedence <= Precedence::Cons => Self::binary(parser, lhs),
            OpLtLt if precedence < Precedence::Compose => Self::binary(parser, lhs),
            OpGtGt if precedence < Precedence::RCompose => Self::binary(parser, lhs),
            OpPipeGt if precedence < Precedence::Pipe => Self::binary(parser, lhs),
            OpLtPipe if precedence <= Precedence::RPipe => Self::binary(parser, lhs),
            OpGlue if precedence < Precedence::Glue => Self::binary(parser, lhs),
            OpBang if precedence < Precedence::Call && !is_spaced => Ok(Continue(Self::Call(
                Box::new(CallExpression::parse(parser, lhs)?),
            ))),

            // NOTE: despite and/or being allowed in patterns, we can't accept them here because
            // they are also allowed in queries, in which case an expression was never an option,
            // unless that expression was the start of a lookup, in which case the and/or is not
            // permitted, so... if we're parsing as maybe expression or pattern, we aren't
            // expecting and/or...
            KwAnd | KwOr => Ok(Done(lhs)),
            // Unary operators and keywords are not permitted here. They must be parenthesized
            // to be considered an argument to an application. It messes with handler parsing
            // otherwise, while also being confusing in terms of precedence rules.
            KwYield | KwResume | KwCancel | KwReturn | KwContinue | KwBreak | KwBecome => {
                Ok(Done(lhs))
            }

            // A function application never spans across two lines. Furthermore,
            // the application requires a space, even when the parse would be
            // otherwise unambiguous.
            //
            // Sadly, the list of things that can follow, for an application, is
            // anything prefix (except unary operators or blocks) so this becomes
            // a very long list.
            _ if Expression::PREFIX.matches(token)
                && precedence < Precedence::Application
                && !is_line_start
                && is_spaced =>
            {
                Ok(Continue(Self::Application(Box::new(Application::parse(
                    parser, lhs,
                )?))))
            }
            // If nothing matched, it must be the end of the expression
            _ => Ok(Done(lhs)),
        }
    }

    fn parse_prefix(parser: &mut Parser) -> SyntaxResult<Result<Self, Pattern>> {
        let token = parser.peek();
        use TokenType::*;
        match token.token_type {
            Numeric => Ok(Ok(Self::Number(Box::new(NumberLiteral::parse(parser)?)))),
            String => Ok(Ok(Self::String(Box::new(StringLiteral::parse(parser)?)))),
            Bits => Ok(Ok(Self::Bits(Box::new(BitsLiteral::parse(parser)?)))),
            KwTrue | KwFalse => Ok(Ok(Self::Boolean(Box::new(BooleanLiteral::parse(parser)?)))),
            Atom => {
                let atom = AtomLiteral::parse(parser)?;
                if parser.check(OParen).is_ok() {
                    let result = StructLiteral::parse(parser, atom)?;
                    match result {
                        Ok(expr) => Ok(Ok(Self::Struct(Box::new(expr)))),
                        Err(patt) => Ok(Err(Pattern::Struct(Box::new(patt)))),
                    }
                } else {
                    Ok(Ok(Self::Atom(Box::new(atom))))
                }
            }
            Character => Ok(Ok(Self::Character(Box::new(CharacterLiteral::parse(
                parser,
            )?)))),
            KwUnit => Ok(Ok(Self::Unit(Box::new(UnitLiteral::parse(parser)?)))),
            OBrack => {
                let start = parser.expect(OBrack).unwrap();
                if let Ok(end) = parser.expect(CBrack) {
                    return Ok(Ok(Self::Array(Box::new(ArrayLiteral::new_empty(
                        start, end,
                    )))));
                }
                match ArrayElement::parse(parser)? {
                    Ok(ArrayElement::Element(expression)) if parser.check(KwFor).is_ok() => {
                        Ok(Ok(Self::ArrayComprehension(Box::new(
                            ArrayComprehension::parse_rest(parser, start, expression)?,
                        ))))
                    }
                    Ok(element) => match ArrayLiteral::parse_rest(parser, start, element)? {
                        Ok(expr) => Ok(Ok(Self::Array(Box::new(expr)))),
                        Err(patt) => Ok(Err(Pattern::Array(Box::new(patt)))),
                    },
                    Err(next) => Ok(Err(Pattern::Array(Box::new(
                        ArrayPattern::parse_from_expression(
                            parser,
                            start,
                            Punctuated::new(),
                            next,
                        )?,
                    )))),
                }
            }
            OBrackPipe => {
                let start = parser.expect(OBrackPipe).unwrap();
                if let Ok(end) = parser.expect(CBrackPipe) {
                    return Ok(Ok(Self::Set(Box::new(SetLiteral::new_empty(start, end)))));
                }
                match SetElement::parse(parser)? {
                    Ok(SetElement::Element(expression)) if parser.expect(KwFor).is_ok() => {
                        Ok(Ok(Self::SetComprehension(Box::new(
                            SetComprehension::parse_rest(parser, start, expression)?,
                        ))))
                    }
                    Ok(element) => {
                        let result = SetLiteral::parse_rest(parser, start, element)?;
                        match result {
                            Ok(expr) => Ok(Ok(Self::Set(Box::new(expr)))),
                            Err(patt) => Ok(Err(Pattern::Set(Box::new(patt)))),
                        }
                    }
                    Err(next) => Ok(Err(Pattern::Set(Box::new(
                        SetPattern::parse_from_expression(parser, start, vec![], next)?,
                    )))),
                }
            }
            OBracePipe => {
                let start = parser.expect(OBracePipe).unwrap();
                if let Ok(end) = parser.expect(CBracePipe) {
                    return Ok(Ok(Self::Record(Box::new(RecordLiteral::new_empty(
                        start, end,
                    )))));
                }
                match RecordElement::parse(parser)? {
                    Ok(RecordElement::Element { key, value, .. })
                        if parser.expect(KwFor).is_ok() =>
                    {
                        Ok(Ok(Self::RecordComprehension(Box::new(
                            RecordComprehension::parse_rest(parser, start, key, value)?,
                        ))))
                    }
                    Ok(element) => {
                        let result = RecordLiteral::parse_rest(parser, start, element)?;
                        match result {
                            Ok(literal) => Ok(Ok(Self::Record(Box::new(literal)))),
                            Err(pattern) => Ok(Err(Pattern::Record(Box::new(pattern)))),
                        }
                    }
                    Err(next) => Ok(Err(Pattern::Record(Box::new(
                        RecordPattern::parse_from_expression(parser, start, vec![], next)?,
                    )))),
                }
            }
            OpBang | OpMinus | OpTilde | KwYield | KwTypeof => match UnaryOperation::parse(parser)?
            {
                Ok(expr) => Ok(Ok(Self::Unary(Box::new(expr)))),
                Err(patt) => Ok(Err(patt)),
            },
            KwIf => {
                let expr = IfElseExpression::parse(parser)?;
                if !expr.is_strict_expression() {
                    parser.error(ErrorKind::IfExpressionRestriction.at(expr.span()));
                }
                Ok(Ok(Self::IfElse(Box::new(expr))))
            }
            KwMatch => Ok(Ok(Self::Match(Box::new(MatchExpression::parse(parser)?)))),
            KwEnd => Ok(Ok(Self::End(Box::new(EndExpression::parse(parser)?)))),
            KwExit => Ok(Ok(Self::Exit(Box::new(ExitExpression::parse(parser)?)))),
            KwReturn => Ok(Ok(Self::Return(Box::new(ReturnExpression::parse(parser)?)))),
            KwResume => Ok(Ok(Self::Resume(Box::new(ResumeExpression::parse(parser)?)))),
            KwBecome => Ok(Ok(Self::Become(Box::new(BecomeExpression::parse(parser)?)))),
            KwBreak => Ok(Ok(Self::Break(Box::new(BreakExpression::parse(parser)?)))),
            KwContinue => Ok(Ok(Self::Continue(Box::new(ContinueExpression::parse(
                parser,
            )?)))),
            KwCancel => Ok(Ok(Self::Cancel(Box::new(CancelExpression::parse(parser)?)))),
            KwLet => Ok(Ok(Self::Let(Box::new(LetExpression::parse(parser)?)))),
            Identifier => Ok(Ok(Self::Reference(Box::new(super::Identifier::parse(
                parser,
            )?)))),
            KwWith => Ok(Ok(Self::Handled(Box::new(HandledExpression::parse(
                parser,
            )?)))),
            KwFn => Ok(Ok(Self::Fn(Box::new(FnExpression::parse(parser)?)))),
            KwDo => Ok(Ok(Self::Do(Box::new(DoExpression::parse(parser)?)))),
            KwQy => Ok(Ok(Self::Qy(Box::new(QyExpression::parse(parser)?)))),
            OpDollar => Ok(Ok(Self::Template(Box::new(Template::parse_tagged(
                parser,
            )?)))),
            TemplateStart => Ok(Ok(Self::Template(Box::new(Template::parse_bare(parser)?)))),
            OParen => match KeywordReference::try_parse(parser) {
                Some(keyword) => Ok(Ok(Self::Keyword(Box::new(keyword)))),
                None => match ParenthesizedExpression::parse(parser)? {
                    Ok(expression) => Ok(Ok(Self::Parenthesized(Box::new(expression)))),
                    Err(pattern) => Ok(Err(Pattern::Parenthesized(Box::new(pattern)))),
                },
            },
            KwIs => Ok(Ok(Self::Is(Box::new(IsExpression::parse(parser)?)))),
            KwMut | Discard | OpCaret => Ok(Err(Pattern::parse(parser)?)),

            // Invalid stuff, but we can do better error messages by handling some specifically
            KwNot => {
                let error = ErrorKind::KwNotInExpression.at(token.span);
                parser.error(error.clone());
                Err(error)
            }
            OBrace => Ok(Ok(Expression::Block(Box::new(Block::parse(parser)?)))),
            _ => {
                let error = SyntaxError::new(
                    token.span,
                    format!("unexpected token {:?} in expression", token.token_type),
                );
                parser.error(error.clone());
                Err(error)
            }
        }
    }

    fn parse_suffix(
        parser: &mut Parser,
        precedence: Precedence,
        mut expr: Expression,
    ) -> SyntaxResult<Result<Self, Pattern>> {
        loop {
            match Self::parse_follow(parser, precedence, expr)? {
                ExpressionResult::Continue(updated) => expr = updated,
                ExpressionResult::Done(expr) => return Ok(Ok(expr)),
                ExpressionResult::Pattern(patt) => return Ok(Err(patt)),
            }
        }
    }

    fn parse_precedence_inner(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Result<Self, Pattern>> {
        match Self::parse_prefix(parser)? {
            Ok(expr) => Self::parse_suffix(parser, precedence, expr),
            Err(patt) => Ok(Err(Pattern::parse_suffix(
                parser,
                pattern::Precedence::None,
                patt,
            )?)),
        }
    }

    pub(crate) fn parse_or_pattern_precedence(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Result<Self, Pattern>> {
        Self::parse_precedence_inner(parser, precedence)
    }

    pub(crate) fn parse_precedence(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Self> {
        Self::parse_or_pattern_precedence(parser, precedence)?.map_err(|patt| {
            SyntaxError::new(
                patt.span(),
                "expected an expression in parameter list, but found a pattern",
            )
        })
    }

    pub(crate) fn parse_or_pattern(parser: &mut Parser) -> SyntaxResult<Result<Self, Pattern>> {
        Self::parse_or_pattern_precedence(parser, Precedence::None)
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        Self::parse_precedence(parser, Precedence::None)
    }

    pub(crate) fn is_lvalue(&self) -> bool {
        match self {
            Self::Binary(op) if matches!(op.operator, BinaryOperator::Access(..)) => true,
            _ => self.is_pattern(),
        }
    }

    pub(crate) fn is_pattern(&self) -> bool {
        match self {
            Self::Reference(..) => true,
            Self::Atom(..) => true,
            Self::Number(..) => true,
            Self::Boolean(..) => true,
            Self::Unit(..) => true,
            Self::Bits(..) => true,
            Self::String(..) => true,
            Self::Character(..) => true,
            Self::Unary(op) if matches!(op.operator, UnaryOperator::Negate(..)) => {
                op.operand.is_pattern()
            }
            Self::Binary(op)
                if matches!(
                    op.operator,
                    BinaryOperator::Glue(..) | BinaryOperator::Cons(..)
                ) =>
            {
                op.lhs.is_pattern() && op.rhs.is_pattern()
            }
            Self::Struct(inner) => inner.value.is_pattern(),
            Self::Array(array) => {
                array.elements.iter().all(|element| match element {
                    ArrayElement::Element(element) => element.is_pattern(),
                    ArrayElement::Spread(_, element) => element.is_pattern(),
                }) && array
                    .elements
                    .iter()
                    .filter(|element| matches!(element, ArrayElement::Spread(..)))
                    .count()
                    <= 1
            }
            Self::Set(set) => set.elements.iter().all(|element| match element {
                SetElement::Element(element) => element.is_pattern(),
                SetElement::Spread(_, spread)
                    if std::ptr::eq(element, set.elements.last().unwrap()) =>
                {
                    spread.is_pattern()
                }
                _ => false,
            }),
            Self::Record(record) => record.elements.iter().all(|element| match element {
                RecordElement::Element { key, value, .. } => key.is_pattern() && value.is_pattern(),
                RecordElement::Spread { value, .. }
                    if std::ptr::eq(element, record.elements.last().unwrap()) =>
                {
                    value.is_pattern()
                }
                _ => false,
            }),
            Self::Parenthesized(paren) => paren.expression.is_pattern(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(expr_prec_boolean: "!true && false || true && !false" => Expression::parse =>
        Expression::Binary(
            BinaryOperation {
                lhs: Expression::Binary(BinaryOperation { lhs: Expression::Unary(..), operator: BinaryOperator::And(..), .. }),
                operator: BinaryOperator::Or(..),
                rhs: Expression::Binary(BinaryOperation { operator: BinaryOperator::And(..), rhs: Expression::Unary(..), .. }),
                ..
            }
        )
    );
    test_parse!(expr_prec_arithmetic: "- 1 / 2 + 3 ** e - 4 * 5" => Expression::parse =>
        Expression::Binary(BinaryOperation {
            lhs:
                Expression::Binary(BinaryOperation {
                    lhs:
                        Expression::Binary(BinaryOperation {
                            lhs: Expression::Unary(_),
                            operator: BinaryOperator::Divide(..),
                            ..
                        }),
                    operator: BinaryOperator::Add(..),
                    rhs:
                        Expression::Binary(BinaryOperation {
                            operator: BinaryOperator::Power(..),
                            ..
                        }),
                    ..
                }),
            operator: BinaryOperator::Subtract(_),
            rhs:
                Expression::Binary(BinaryOperation {
                    operator: BinaryOperator::Multiply(..),
                    ..
                }),
            ..
        })
    );
    test_parse!(expr_prec_factor: "1 / 2 % 3 // 4 * 5" => Expression::parse =>
        Expression::Binary(
            BinaryOperation {
                lhs: Expression::Binary(
                    BinaryOperation {
                        lhs: Expression::Binary(BinaryOperation {
                            lhs: Expression::Binary(BinaryOperation { operator: BinaryOperator::Divide(_), .. }),
                            operator: BinaryOperator::Remainder(_),
                            ..
                        }),
                        operator: BinaryOperator::IntDivide(_),
                        ..
                    }
                ),
                operator: BinaryOperator::Multiply(_),
                ..
            }
        )
    );
    test_parse!(expr_prec_bitwise: "~x & y <~ 4 | x ^ y ~> 3" => Expression::parse =>
        Expression::Binary(
            BinaryOperation {
                lhs: Expression::Binary(
                    BinaryOperation {
                        lhs: Expression::Binary(BinaryOperation { lhs: Expression::Unary(..), operator:BinaryOperator::BitwiseAnd(_), .. }),
                        operator: BinaryOperator::LeftShift(..),
                        ..
                    }
                ),
                operator: BinaryOperator::BitwiseOr(..),
                rhs: Expression::Binary(
                    BinaryOperation {
                        operator: BinaryOperator::BitwiseXor(_),
                        rhs: Expression::Binary(
                            BinaryOperation { operator: BinaryOperator::RightShift(_), .. },
                        ),
                        ..
                    }
                ),
                ..
            }
        )
    );
    test_parse!(expr_prec_app_pipe: "f x |> map (fn y. 2 * y) |> print" => Expression::parse =>
        Expression::Binary(
          BinaryOperation {
                lhs: Expression::Binary(BinaryOperation { lhs: Expression::Application(_), operator: BinaryOperator::Pipe(_), rhs: Expression::Application(_), .. }),
                operator: BinaryOperator::Pipe(..),
                rhs: Expression::Reference(..),
                ..
            }
        )
    );
    test_parse!(expr_prec_pipe_rpipe: "x |> f <| g <| y |> h" => Expression::parse =>
        Expression::Binary(
            BinaryOperation {
          lhs: Expression::Binary(
              BinaryOperation {
              lhs: Expression::Reference(_),
              operator: BinaryOperator::Pipe(_),
              rhs: Expression::Binary(
                  BinaryOperation {
                  lhs: Expression::Reference(_),
                  operator: BinaryOperator::RPipe(_),
                  rhs: Expression::Binary
                    (BinaryOperation {
                      lhs: Expression::Reference(_),
                      operator: BinaryOperator::RPipe(_),
                      rhs: Expression::Reference(_),..
                    }),
                      ..
                }),
                  ..
              }),
              operator: BinaryOperator::Pipe(_),
              rhs: Expression::Reference(_),
              ..
            }
        )
    );
    test_parse!(expr_prec_compose_rcompose: "x >> f << g << y >> h" => Expression::parse =>
    Expression::Binary
      (BinaryOperation {
        lhs: Expression::Binary
          (BinaryOperation {
            lhs: Expression::Reference(_),
            operator: BinaryOperator::RCompose(_),
            rhs: Expression::Binary
              (BinaryOperation {
                lhs: Expression::Binary(
                  BinaryOperation {
                    lhs: Expression::Reference(_),
                    operator: BinaryOperator::Compose(_),
                    rhs: Expression::Reference(_),
                    ..
                  }
                ),
                operator: BinaryOperator::Compose(_),
                rhs: Expression::Reference(_),
                ..
                }),
                ..
            }),
        operator: BinaryOperator::RCompose(_),
        rhs: Expression::Reference(_),
        ..
        })
    );
    test_parse!(expr_prec_application: "x - f y + f y <| -z" => Expression::parse =>
      Expression::Binary
        (BinaryOperation {
          lhs: Expression::Binary
            (BinaryOperation {
              lhs: Expression::Binary
                (BinaryOperation {
                  lhs: Expression::Reference(_),
                  operator: BinaryOperator::Subtract(_),
                  rhs: Expression::Application(_),
                  ..}),
              operator: BinaryOperator::Add(_),
              rhs: Expression::Application(_),
              ..
          }),
          operator: BinaryOperator::RPipe(_),
          rhs: Expression::Unary(_),
          ..
      })
    );
    test_parse!(expr_prec_if_else: "if true then 5 + 6 else 7 + 8" => Expression::parse =>
      Expression::IfElse
        (IfElseExpression {
          condition: Expression::Boolean(..),
          when_true: FollowingExpression::Then(..),
          when_false: Some(ElseClause{..}),
          ..
          })
    );

    test_parse_error!(expr_eq_without_parens: "x == y == z" => Expression::parse => "equality operators cannot be chained, use parentheses to disambiguate");
    test_parse_error!(expr_cmp_without_parens: "x <= y <= z" => Expression::parse => "comparison operators cannot be chained, use parentheses to disambiguate");
    test_parse!(expr_prec_cmp_eq: "x < y == y > z" => Expression::parse =>
    Expression::Binary
      (BinaryOperation {
        lhs: Expression::Binary(..),
        operator: BinaryOperator::StructuralEquality(_),
        rhs: Expression::Binary(..),
        ..
        })
    );

    test_parse_error!(expr_multiline_application: "f\nx" => Expression::parse);
    test_parse_error!(expr_application_no_space: "f(x)" => Expression::parse);
    test_parse!(expr_multiline_operators: "f\n<| x" => Expression::parse =>
      Expression::Binary(BinaryOperation {
          lhs: Expression::Reference(_),
          operator: BinaryOperator::RPipe(_),
          rhs: Expression::Reference(_),
          ..
      })
    );

    test_parse!(expr_is: "true && is check(y) and also(z) && false" => Expression::parse =>
      Expression::Binary
        (BinaryOperation {
          lhs: Expression::Binary
            (BinaryOperation {
              lhs: Expression::Boolean(_),
              operator: BinaryOperator::And(_),
              rhs: Expression::Is(_), ..
              }),
          operator: BinaryOperator::And(_),
          rhs: Expression::Boolean(_),
          ..
      })
    );
    test_parse!(expr_is_prec: "false || is x = y && z" => Expression::parse => 
      Expression::Binary
        (BinaryOperation {
          lhs: Expression::Boolean(_),
          operator: BinaryOperator::Or(_),
          rhs: Expression::Is(_),
          ..}));

    test_parse!(expr_prec_glue_cons: "\"hello\" : \"hello\" <> \"world\" : 3 + 3 : \"world\"" => Expression::parse => 
      Expression::Binary
        (BinaryOperation {
          lhs: Expression::String(_),
          operator: BinaryOperator::Cons(_),
          rhs: Expression::Binary
            (BinaryOperation {
              lhs: Expression::Binary
                (BinaryOperation {
                  lhs: Expression::String(_),
                  operator: BinaryOperator::Glue(_),
                  rhs: Expression::String(_),
                  ..
                  }),
              operator: BinaryOperator::Cons(_),
              rhs: Expression::Binary
                (BinaryOperation {
                  lhs: Expression::Binary
                    (BinaryOperation {
                      lhs: Expression::Number(_),
                      operator: BinaryOperator::Add(_),
                      rhs: Expression::Number(_),
                      ..
                      }),
                  operator: BinaryOperator::Cons(_),
                  rhs: Expression::String(_),
                  ..
                  })
              , ..}),
                  ..}));

    test_parse!(expr_prec_call: "a + b!() + c!()" => Expression::parse =>
    Expression::Binary
      (BinaryOperation {
        lhs: Expression::Binary(
          BinaryOperation {
            lhs: Expression::Reference(_),
            operator: BinaryOperator::Add(_),
            rhs: Expression::Call(_),
            ..}),
        operator: BinaryOperator::Add(_),
        rhs: Expression::Call(_),
        ..
        }));

    test_parse!(expr_prec_access: "a.1!() + b.'hello 3" => Expression::parse =>
    Expression::Binary
      (BinaryOperation {
        lhs: Expression::Call(
          CallExpression {
            procedure: Expression::Binary
              (BinaryOperation {
                lhs: Expression::Reference(_),
                operator: BinaryOperator::Access(_),
                rhs: Expression::Number(_),
                ..
                }),
            ..
          }
        ),
        operator: BinaryOperator::Add(_),
        rhs: Expression::Application
          (Application {
            function: Expression::Binary
              (BinaryOperation {
                lhs: Expression::Reference(_),
                operator: BinaryOperator::Access(_),
                rhs: Expression::Atom(_),
                    ..
            }),
            argument: Expression::Number(_)
            , ..
        }), ..})
        );
    test_parse!(expr_mod_access: "mod1::mod2::member" => Expression::parse => 
      Expression::ModuleAccess
        (ModuleAccess {
          lhs: Expression::ModuleAccess(ModuleAccess {
              lhs: Expression::Reference(_),
              rhs: Identifier{..}, .. }),
          rhs: Identifier{..}, .. }));
    test_parse!(expr_prec_paths: "a b::c d::e f" => Expression::parse =>
      Expression::Application
        (Application {
          function: Expression::Application
            (Application {
              function: Expression::Application
                (Application {
                  function: Expression::Reference(_),
                  argument: Expression::ModuleAccess(_) , ..}),
              argument: Expression::ModuleAccess(_),
              ..}),
          argument: Expression::Reference(_), .. })
    );
}
