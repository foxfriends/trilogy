use super::token_type::TokenType;
use super::token_value::TokenValue;
use source_span::Span;

/// A single lexeme of the Trilogy language.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Token {
    /// The type of token this is.
    pub token_type: TokenType,
    /// The resolved value of the token.
    pub value: Option<TokenValue>,
    /// Location in original source file.
    pub span: Span,
}

impl Token {
    pub(crate) fn new(token_type: TokenType, span: Span) -> Self {
        Self {
            token_type,
            value: None,
            span,
        }
    }

    pub(crate) fn with_value<V>(mut self, value: V) -> Self
    where
        TokenValue: From<V>,
    {
        self.value = Some(value.into());
        self
    }

    pub(crate) fn resolve_keywords(mut self) -> Result<Self, &'static str> {
        if self.token_type != TokenType::Identifier && self.token_type != TokenType::IdentifierEq {
            return Ok(self);
        }
        let is_eq = self.token_type == TokenType::IdentifierEq;

        self.token_type = match self.value.as_ref().unwrap().as_str().unwrap() {
            "_" => TokenType::Discard,
            "and" => TokenType::KwAnd,
            "assert" => TokenType::KwAssert,
            "async" => TokenType::KwAsync,
            "as" => TokenType::KwAs,
            "at" => TokenType::KwAt,
            "await" => TokenType::KwAwait,
            "become" => TokenType::KwBecome,
            "break" => TokenType::KwBreak,
            "cancel" => TokenType::KwCancel,
            "case" => TokenType::KwCase,
            "catch" => TokenType::KwCatch,
            "class" => TokenType::KwClass,
            "const" => TokenType::KwConst,
            "continue" => TokenType::KwContinue,
            "data" => TokenType::KwData,
            "defer" => TokenType::KwDefer,
            "do" => TokenType::KwDo,
            "else" => TokenType::KwElse,
            "enum" => TokenType::KwEnum,
            "end" => TokenType::KwEnd,
            "except" => TokenType::KwExcept,
            "exit" => TokenType::KwExit,
            "export" => TokenType::KwExport,
            "extends" => TokenType::KwExtends,
            "false" => TokenType::KwFalse,
            "fn" => TokenType::KwFn,
            "for" => TokenType::KwFor,
            "func" => TokenType::KwFunc,
            "if" => TokenType::KwIf,
            "implements" => TokenType::KwImplements,
            "import" => TokenType::KwImport,
            "in" => TokenType::KwIn,
            "inline" => TokenType::KwInline,
            "instanceof" => TokenType::KwInstanceof,
            "interface" => TokenType::KwInterface,
            "is" => TokenType::KwIs,
            "lazy" => TokenType::KwLazy,
            "lens" => TokenType::KwLens,
            "let" => TokenType::KwLet,
            "loop" => TokenType::KwLoop,
            "macro" => TokenType::KwMacro,
            "match" => TokenType::KwMatch,
            "module" => TokenType::KwModule,
            "mut" => TokenType::KwMut,
            "next" => TokenType::KwNext,
            "not" => TokenType::KwNot,
            "oper" => TokenType::KwOper,
            "or" => TokenType::KwOr,
            "pass" => TokenType::KwPass,
            "prec" => TokenType::KwPrec,
            "proc" => TokenType::KwProc,
            "protocol" => TokenType::KwProtocol,
            "qy" => TokenType::KwQy,
            "resume" => TokenType::KwResume,
            "return" => TokenType::KwReturn,
            "rule" => TokenType::KwRule,
            "static" => TokenType::KwStatic,
            "struct" => TokenType::KwStruct,
            "super" => TokenType::KwSuper,
            "switch" => TokenType::KwSwitch,
            "tag" => TokenType::KwTag,
            "test" => TokenType::KwTest,
            "then" => TokenType::KwThen,
            "trait" => TokenType::KwTrait,
            "true" => TokenType::KwTrue,
            "try" => TokenType::KwTry,
            "type" => TokenType::KwType,
            "typeof" => TokenType::KwTypeof,
            "unit" => TokenType::KwUnit,
            "unless" => TokenType::KwUnless,
            "until" => TokenType::KwUntil,
            "use" => TokenType::KwUse,
            "when" => TokenType::KwWhen,
            "where" => TokenType::KwWhere,
            "while" => TokenType::KwWhile,
            "with" => TokenType::KwWith,
            "yield" => TokenType::KwYield,
            _ => self.token_type,
        };

        if is_eq && self.token_type != TokenType::IdentifierEq {
            return Err("cannot use keyword as an assignment function");
        }

        Ok(self)
    }
}
