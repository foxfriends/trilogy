use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct KeywordReference {
    pub open_paren: Token,
    pub keyword: Keyword,
    pub close_paren: Token,
}

impl Spanned for KeywordReference {
    fn span(&self) -> Span {
        self.open_paren.span.union(self.close_paren.span)
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Keyword {
    And(Token),
    Or(Token),
    Add(Token),
    Subtract(Token),
    Multiply(Token),
    Divide(Token),
    Remainder(Token),
    Power(Token),
    IntDivide(Token),
    StructuralEquality(Token),
    ReferenceEquality(Token),
    StructuralInequality(Token),
    ReferenceInequality(Token),
    Lt(Token),
    Gt(Token),
    Leq(Token),
    Geq(Token),
    BitwiseAnd(Token),
    BitwiseOr(Token),
    BitwiseXor(Token),
    LeftShift(Token),
    RightShift(Token),
    Sequence(Token),
    Cons(Token),
    Glue(Token),
    Compose(Token),
    RCompose(Token),
    Pipe(Token),
    RPipe(Token),
    Not(Token),
    Invert(Token),
    Yield(Token),
    Resume(Token),
    Cancel(Token),
    Return(Token),
    Break(Token),
    Continue(Token),
    Typeof(Token),
}

impl KeywordReference {
    pub(crate) fn try_parse(parser: &mut Parser) -> Option<Self> {
        let tokens = parser.peekn(3)?;
        if tokens[0].token_type != OParen || tokens[2].token_type != CParen {
            return None;
        }
        let constructor = match tokens[1].token_type {
            OpBang => Keyword::Not,
            OpTilde => Keyword::Invert,
            KwYield => Keyword::Yield,
            OpAmpAmp => Keyword::And,
            OpPipePipe => Keyword::Or,
            OpPlus => Keyword::Add,
            OpMinus => Keyword::Subtract,
            OpStar => Keyword::Multiply,
            OpSlash => Keyword::Divide,
            OpSlashSlash => Keyword::IntDivide,
            OpPercent => Keyword::Remainder,
            OpStarStar => Keyword::Power,
            OpEqEq => Keyword::StructuralEquality,
            OpBangEq => Keyword::StructuralInequality,
            OpEqEqEq => Keyword::ReferenceEquality,
            OpBangEqEq => Keyword::ReferenceInequality,
            OpLt => Keyword::Lt,
            OpGt => Keyword::Gt,
            OpLtEq => Keyword::Leq,
            OpGtEq => Keyword::Geq,
            OpAmp => Keyword::BitwiseAnd,
            OpPipe => Keyword::BitwiseOr,
            OpCaret => Keyword::BitwiseXor,
            OpShl => Keyword::LeftShift,
            OpShr => Keyword::RightShift,
            OpComma => Keyword::Sequence,
            OpColon => Keyword::Cons,
            OpGlue => Keyword::Glue,
            OpGtGt => Keyword::Compose,
            OpLtLt => Keyword::RCompose,
            OpPipeGt => Keyword::Pipe,
            OpLtPipe => Keyword::RPipe,
            KwBreak => Keyword::Break,
            KwContinue => Keyword::Continue,
            KwResume => Keyword::Resume,
            KwCancel => Keyword::Cancel,
            KwReturn => Keyword::Return,
            KwTypeof => Keyword::Typeof,
            _ => return None,
        };
        let open_paren = parser.consume();
        let keyword = constructor(parser.consume());
        let close_paren = parser.consume();
        Some(Self {
            open_paren,
            keyword,
            close_paren,
        })
    }
}
