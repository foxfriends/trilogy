use super::*;
use crate::{token_pattern::TokenPattern, Parser, Spanned};
use trilogy_scanner::TokenType::{self, *};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
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
    IteratorComprehension(Box<IteratorComprehension>),
    Reference(Box<Path>),
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
    Cancel(Box<CancelExpression>),
    Return(Box<ReturnExpression>),
    Break(Box<BreakExpression>),
    Continue(Box<ContinueExpression>),
    Fn(Box<FnExpression>),
    Do(Box<DoExpression>),
    Template(Box<Template>),
    Handled(Box<HandledExpression>),
    Parenthesized(Box<ParenthesizedExpression>),
    Module(Box<ModulePath>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum Precedence {
    None,
    Sequence,
    Continuation,
    Pipe,
    RPipe,
    Or,
    And,
    Equality,
    Comparison,
    Cons,
    Glue,
    BitwiseOr,
    BitwiseXor,
    BitwiseShift,
    BitwiseAnd,
    Term,
    Factor,
    Exponent,
    Compose,
    RCompose,
    Path,
    Application,
    Unary,
    Call,
    Access,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum Context {
    ParameterList,
    Default,
}

impl Expression {
    pub(crate) const PREFIX: [TokenType; 34] = [
        Numeric,
        String,
        Bits,
        KwTrue,
        KwFalse,
        Atom,
        Character,
        KwUnit,
        OBrack,
        OBrackPipe,
        OBracePipe,
        DollarOParen,
        OpBang,
        OpTilde,
        KwYield,
        KwIf,
        KwIs,
        KwMatch,
        KwEnd,
        KwExit,
        KwReturn,
        KwResume,
        KwBreak,
        KwContinue,
        KwCancel,
        KwLet,
        Identifier,
        KwWith,
        KwFn,
        KwDo,
        DollarString,
        TemplateStart,
        OParen,
        OpAt,
    ];

    fn binary(parser: &mut Parser, lhs: Expression) -> SyntaxResult<Result<Self, Self>> {
        BinaryOperation::parse(parser, lhs)
            .map(Box::new)
            .map(Self::Binary)
            .map(Ok)
    }

    fn parse_follow(
        parser: &mut Parser,
        precedence: Precedence,
        lhs: Expression,
        context: Context,
    ) -> SyntaxResult<Result<Self, Self>> {
        // A bit of strangeness here because `peek()` takes a mutable reference,
        // so we can't use the fields afterwards... but we need their value after
        // and I'd really rather not clone every token of every expression, so
        // instead we peek first, then do a force peek after (to skip an extra chomp).
        parser.peek();
        let is_spaced = parser.is_spaced;
        let is_line_start = parser.is_line_start;
        let token = parser.force_peek();
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
                if let Ok(Ok(expr)) = &expr {
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
                if let Ok(Ok(expr)) = &expr {
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
            OpShr | OpShl if precedence < Precedence::BitwiseShift => Self::binary(parser, lhs),
            OpColon if precedence <= Precedence::Cons => Self::binary(parser, lhs),
            OpGtGt if precedence < Precedence::Compose => Self::binary(parser, lhs),
            OpLtLt if precedence < Precedence::RCompose => Self::binary(parser, lhs),
            OpPipeGt if precedence < Precedence::Pipe => Self::binary(parser, lhs),
            OpLtPipe if precedence < Precedence::RPipe => Self::binary(parser, lhs),
            OpGlue if precedence < Precedence::Glue => Self::binary(parser, lhs),
            OpBang if precedence < Precedence::Call && !is_spaced => Ok(Ok(Self::Call(Box::new(
                CallExpression::parse(parser, lhs)?,
            )))),
            OpComma if precedence <= Precedence::Sequence && context == Context::Default => {
                Self::binary(parser, lhs)
            }
            // A function application never spans across two lines. Furthermore,
            // the application requires a space, even when the parse would be
            // otherwise unambiguous.
            //
            // Sadly, the list of things that can follow, for an application, is
            // anything prefix (except `-`) so this becomes a very long list.
            _ if Expression::PREFIX.matches(token)
                && precedence < Precedence::Application
                && !is_line_start
                && is_spaced =>
            {
                Ok(Ok(Self::Application(Box::new(Application::parse(
                    parser, lhs,
                )?))))
            }
            // If nothing matched, it must be the end of the expression
            _ => Ok(Err(lhs)),
        }
    }

    fn parse_prefix(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser.peek();
        match token.token_type {
            Numeric => Ok(Self::Number(Box::new(NumberLiteral::parse(parser)?))),
            String => Ok(Self::String(Box::new(StringLiteral::parse(parser)?))),
            Bits => Ok(Self::Bits(Box::new(BitsLiteral::parse(parser)?))),
            KwTrue | KwFalse => Ok(Self::Boolean(Box::new(BooleanLiteral::parse(parser)?))),
            Atom => {
                let atom = AtomLiteral::parse(parser)?;
                if parser.check(OParen).is_ok() {
                    Ok(Self::Struct(Box::new(StructLiteral::parse(parser, atom)?)))
                } else {
                    Ok(Self::Atom(Box::new(atom)))
                }
            }
            Character => Ok(Self::Character(Box::new(CharacterLiteral::parse(parser)?))),
            KwUnit => Ok(Self::Unit(Box::new(UnitLiteral::parse(parser)?))),
            OBrack => {
                let start = parser.expect(OBrack).unwrap();
                if let Ok(end) = parser.expect(CBrack) {
                    return Ok(Self::Array(Box::new(ArrayLiteral::new_empty(start, end))));
                }
                match ArrayElement::parse(parser)? {
                    ArrayElement::Element(expression) if parser.expect(KwFor).is_ok() => {
                        Ok(Self::ArrayComprehension(Box::new(
                            ArrayComprehension::parse_rest(parser, start, expression)?,
                        )))
                    }
                    element => Ok(Self::Array(Box::new(ArrayLiteral::parse_rest(
                        parser, start, element,
                    )?))),
                }
            }
            OBrackPipe => {
                let start = parser.expect(OBrackPipe).unwrap();
                if let Ok(end) = parser.expect(CBrackPipe) {
                    return Ok(Self::Set(Box::new(SetLiteral::new_empty(start, end))));
                }
                match SetElement::parse(parser)? {
                    SetElement::Element(expression) if parser.expect(KwFor).is_ok() => {
                        Ok(Self::SetComprehension(Box::new(
                            SetComprehension::parse_rest(parser, start, expression)?,
                        )))
                    }
                    element => Ok(Self::Set(Box::new(SetLiteral::parse_rest(
                        parser, start, element,
                    )?))),
                }
            }
            OBracePipe => {
                let start = parser.expect(OBracePipe).unwrap();
                if let Ok(end) = parser.expect(CBracePipe) {
                    return Ok(Self::Record(Box::new(RecordLiteral::new_empty(start, end))));
                }
                match RecordElement::parse(parser)? {
                    RecordElement::Element(key, value) if parser.expect(KwFor).is_ok() => {
                        Ok(Self::RecordComprehension(Box::new(
                            RecordComprehension::parse_rest(parser, start, key, value)?,
                        )))
                    }
                    element => Ok(Self::Record(Box::new(RecordLiteral::parse_rest(
                        parser, start, element,
                    )?))),
                }
            }
            DollarOParen => Ok(Self::IteratorComprehension(Box::new(
                IteratorComprehension::parse(parser)?,
            ))),
            OpBang | OpMinus | OpTilde | KwYield => {
                Ok(Self::Unary(Box::new(UnaryOperation::parse(parser)?)))
            }
            KwIf => Ok(Self::IfElse(Box::new(IfElseExpression::parse(parser)?))),
            KwMatch => Ok(Self::Match(Box::new(MatchExpression::parse(parser)?))),
            KwEnd => Ok(Self::End(Box::new(EndExpression::parse(parser)?))),
            KwExit => Ok(Self::Exit(Box::new(ExitExpression::parse(parser)?))),
            KwReturn => Ok(Self::Return(Box::new(ReturnExpression::parse(parser)?))),
            KwResume => Ok(Self::Resume(Box::new(ResumeExpression::parse(parser)?))),
            KwBreak => Ok(Self::Break(Box::new(BreakExpression::parse(parser)?))),
            KwContinue => Ok(Self::Continue(Box::new(ContinueExpression::parse(parser)?))),
            KwCancel => Ok(Self::Cancel(Box::new(CancelExpression::parse(parser)?))),
            KwLet => Ok(Self::Let(Box::new(LetExpression::parse(parser)?))),
            Identifier => Ok(Self::Reference(Box::new(
                super::Identifier::parse(parser)?.into(),
            ))),
            KwWith => Ok(Self::Handled(Box::new(HandledExpression::parse(parser)?))),
            KwFn => Ok(Self::Fn(Box::new(FnExpression::parse(parser)?))),
            KwDo => Ok(Self::Do(Box::new(DoExpression::parse(parser)?))),
            DollarString | TemplateStart => Ok(Self::Template(Box::new(Template::parse(parser)?))),
            OParen => match KeywordReference::try_parse(parser) {
                Some(keyword) => Ok(Self::Keyword(Box::new(keyword))),
                None => Ok(Self::Parenthesized(Box::new(
                    ParenthesizedExpression::parse(parser)?,
                ))),
            },
            OpAt => match ModulePath::parse_or_path(parser)? {
                Ok(module) => Ok(Self::Module(Box::new(module))),
                Err(path) => Ok(Self::Reference(Box::new(path))),
            },
            KwIs => Ok(Self::Is(Box::new(IsExpression::parse(parser)?))),
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
        context: Context,
        mut expr: Expression,
    ) -> SyntaxResult<Self> {
        loop {
            match Self::parse_follow(parser, precedence, expr, context)? {
                Ok(updated) => expr = updated,
                Err(expr) => return Ok(expr),
            }
        }
    }

    fn parse_precedence_inner(
        parser: &mut Parser,
        precedence: Precedence,
        context: Context,
    ) -> SyntaxResult<Self> {
        let expr = Self::parse_prefix(parser)?;
        Self::parse_suffix(parser, precedence, context, expr)
    }

    pub(crate) fn parse_precedence(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Self> {
        Self::parse_precedence_inner(parser, precedence, Context::Default)
    }

    pub(crate) fn parse_parameter_list(parser: &mut Parser) -> SyntaxResult<Self> {
        Self::parse_precedence_inner(parser, Precedence::None, Context::ParameterList)
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
            Self::Reference(path) if path.module.is_none() => true,
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
                RecordElement::Element(key, value) => key.is_pattern() && value.is_pattern(),
                RecordElement::Spread(_, spread)
                    if std::ptr::eq(element, record.elements.last().unwrap()) =>
                {
                    spread.is_pattern()
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

    test_parse!(expr_prec_boolean: "!true && false || true && !false" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary (BinaryOperation (Expression::Unary _) (BinaryOperator::And _) _))
          (BinaryOperator::Or _)
          (Expression::Binary (BinaryOperation _ (BinaryOperator::And _) (Expression::Unary _)))))");
    test_parse!(expr_prec_arithmetic: "- 1 / 2 + 3 ** e - 4 * 5" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Binary (BinaryOperation (Expression::Unary _) (BinaryOperator::Divide _) _))
              (BinaryOperator::Add _)
              (Expression::Binary (BinaryOperation _ (BinaryOperator::Power _) _))))
          (BinaryOperator::Subtract _)
          (Expression::Binary
            (BinaryOperation
              _
              (BinaryOperator::Multiply _)
              _))))");
    test_parse!(expr_prec_factor: "1 / 2 % 3 // 4 * 5" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Binary
                (BinaryOperation
                  (Expression::Binary
                    (BinaryOperation
                      _
                      (BinaryOperator::Divide _)
                      _))
                  (BinaryOperator::Remainder _)
                  _))
              (BinaryOperator::IntDivide _)
              _))
          (BinaryOperator::Multiply _)
          _))");
    test_parse!(expr_prec_bitwise: "~x & y <~ 4 | x ^ y ~> 3" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Binary (BinaryOperation (Expression::Unary _) (BinaryOperator::BitwiseAnd _) _))
              (BinaryOperator::LeftShift _)
              _))
          (BinaryOperator::BitwiseOr _)
          (Expression::Binary
            (BinaryOperation
              _
              (BinaryOperator::BitwiseXor _)
              (Expression::Binary
                (BinaryOperation
                  _
                  (BinaryOperator::RightShift _)
                  _))))))");
    test_parse!(expr_prec_app_pipe: "f x |> map (fn y. 2 * y) |> print" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Application _)
              (BinaryOperator::Pipe _)
              (Expression::Application _)))
          (BinaryOperator::Pipe _)
          (Expression::Reference _)))");
    test_parse!(expr_prec_pipe_rpipe: "x |> f <| g <| y |> h" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Reference _)
              (BinaryOperator::Pipe _)
              (Expression::Binary
                (BinaryOperation
                  (Expression::Binary
                    (BinaryOperation
                      (Expression::Reference _)
                      (BinaryOperator::RPipe _)
                      (Expression::Reference _)))
                  (BinaryOperator::RPipe _)
                  (Expression::Reference _)))))
          (BinaryOperator::Pipe _)
          (Expression::Reference _)))");
    test_parse!(expr_prec_compose_rcompose: "x >> f << g << y >> h" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Reference _)
              (BinaryOperator::Compose _)
              (Expression::Binary
                (BinaryOperation
                  (Expression::Binary
                    (BinaryOperation
                      (Expression::Reference _)
                      (BinaryOperator::RCompose _)
                      (Expression::Reference _)))
                  (BinaryOperator::RCompose _)
                  (Expression::Reference _)))))
          (BinaryOperator::Compose _)
          (Expression::Reference _)))");
    test_parse!(expr_prec_application: "x - f y + f y <| -z" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Binary
                (BinaryOperation
                  (Expression::Reference _)
                  (BinaryOperator::Subtract _)
                  (Expression::Application _)))
              (BinaryOperator::Add _)
              (Expression::Application _)))
          (BinaryOperator::RPipe _)
          (Expression::Unary _)))");
    test_parse!(expr_prec_if_else: "if true then 5 + 6 else 7 + 8" => Expression::parse => "
      (Expression::IfElse
        (IfElseExpression
          (Expression::Boolean _)
          (Expression::Binary _)
          (Expression::Binary _)))");

    test_parse_error!(expr_eq_without_parens: "x == y == z" => Expression::parse => "equality operators cannot be chained, use parentheses to disambiguate");
    test_parse_error!(expr_cmp_without_parens: "x <= y <= z" => Expression::parse => "comparison operators cannot be chained, use parentheses to disambiguate");
    test_parse!(expr_prec_cmp_eq: "x < y == y > z" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary _)
          (BinaryOperator::StructuralEquality _)
          (Expression::Binary _)))");

    test_parse_error!(expr_multiline_application: "f\nx" => Expression::parse);
    test_parse_error!(expr_application_no_space: "f(x)" => Expression::parse);
    test_parse!(expr_multiline_operators: "f\n<| x" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Reference _)
          (BinaryOperator::RPipe _)
          (Expression::Reference _)))");

    test_parse!(expr_is: "true && is check(y) and also(z) && false" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Boolean _)
              (BinaryOperator::And _)
              (Expression::Is _)))
          (BinaryOperator::And _)
          (Expression::Boolean _)))");
    test_parse!(expr_is_prec: "false || is x = y && z" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Boolean _)
          (BinaryOperator::Or _)
          (Expression::Is _)))");

    test_parse!(expr_prec_glue_cons: "\"hello\" : \"hello\" <> \"world\" : 3 + 3 : \"world\"" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::String _)
          (BinaryOperator::Cons _)
          (Expression::Binary
            (BinaryOperation
              (Expression::Binary
                (BinaryOperation
                  (Expression::String _)
                  (BinaryOperator::Glue _)
                  (Expression::String _)))
              (BinaryOperator::Cons _)
              (Expression::Binary
                (BinaryOperation
                  (Expression::Binary
                    (BinaryOperation
                      (Expression::Number _)
                      (BinaryOperator::Add _)
                      (Expression::Number _)))
                  (BinaryOperator::Cons _)
                  (Expression::String _)))))))");

    test_parse!(expr_prec_call: "a + b!() + c!()" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary
            (BinaryOperation
              (Expression::Reference _)
              (BinaryOperator::Add _)
              (Expression::Call _)))
          (BinaryOperator::Add _)
          (Expression::Call _)))");

    test_parse!(expr_prec_access: "a.1!() + b.'hello 3" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Call
            (CallExpression
              (Expression::Binary
                (BinaryOperation
                  (Expression::Reference _)
                  (BinaryOperator::Access _)
                  (Expression::Number _)))
              []))
          (BinaryOperator::Add _)
          (Expression::Application
            (Application
              (Expression::Binary
                (BinaryOperation
                  (Expression::Reference _)
                  (BinaryOperator::Access _)
                  (Expression::Atom _)))
              (Expression::Number _)))))");
    test_parse!(expr_prec_paths: "@a b::@c d::e f" => Expression::parse => "
      (Expression::Application
        (Application
          (Expression::Reference _)
          (Expression::Reference _)))");
    test_parse!(expr_let_seq: "b * 2, let a = b, a * 2, b * 2" => Expression::parse => "
      (Expression::Binary
        (BinaryOperation
          (Expression::Binary _)
          (BinaryOperator::Sequence _)
          (Expression::Let
            (LetExpression
              (Query::Direct _)
              (Expression::Binary
                (BinaryOperation
                  (Expression::Binary _)
                  (BinaryOperator::Sequence _)
                  (Expression::Binary _)))))))");
}
