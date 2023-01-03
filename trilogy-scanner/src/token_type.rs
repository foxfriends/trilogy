#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenType {
    // Keywords
    KwAnd,
    KwAssert,
    KwAt,
    KwBreak,
    KwCancel,
    KwContinue,
    KwDo,
    KwElse,
    KwExport,
    KwFalse,
    KwFn,
    KwFor,
    KwFrom,
    KwFt,
    KwFunc,
    KwGiven,
    KwIf,
    KwImport,
    KwInvert,
    KwIn,
    KwIs,
    KwLet,
    KwLoop,
    KwMatch,
    KwModule,
    KwMut,
    KwNot,
    KwOr,
    KwProc,
    KwResume,
    KwReturn,
    KwRule,
    KwTest,
    KwTrue,
    KwUnit,
    KwUse,
    KwWhen,
    KwWhile,
    KwYield,

    // Reserved Words
    KwAsync,
    KwAwait,
    KwCase,
    KwCatch,
    KwClass,
    KwConst,
    KwData,
    KwDefer,
    KwEnum,
    KwExcept,
    KwExtends,
    KwImplements,
    KwInline,
    KwInstanceof,
    KwInterface,
    KwLazy,
    KwLens,
    KwMacro,
    KwNext,
    KwOper,
    KwPrec,
    KwProtocol,
    KwStatic,
    KwStruct,
    KwSuper,
    KwSwitch,
    KwTag,
    KwTrait,
    KwTry,
    KwType,
    KwTypeof,
    KwUnless,
    KwUntil,
    KwVar,
    KwWhere,

    // Identifiers
    Discard,
    IdentifierEq,
    IdentifierBang,
    Identifier,

    // Literals
    Atom,
    Numeric,
    String,
    TemplateStart,
    TemplateContinue,
    TemplateEnd,
    Character,

    // Whitespace
    EndOfLine,

    // Comments
    CommentInline,
    CommentBlock,
    CommentLine,
    DocOuter,
    DocInner,

    // Punctuation
    OpEq,
    OpLt,
    OpGt,
    OpEqEq,
    OpLtEq,
    OpGtEq,
    OpAmp,
    OpPipe,
    OpCaret,
    OpTilde,
    OpShr,
    OpShl,
    OpAmpEq,
    OpPipeEq,
    OpCaretEq,
    OpTildeEq,
    OpShrEq,
    OpShlEq,
    OpAt,
    OpDot,
    OpDotDot,
    OpComma,
    OpColon,
    OpSemi,
    OpPlus,
    OpMinus,
    OpStar,
    OpSlash,
    OpSlashSlash,
    OpPercent,
    OpStarStar,
    OpPlusEq,
    OpMinusEq,
    OpStarEq,
    OpSlashEq,
    OpSlashSlashEq,
    OpPercentEq,
    OpStarstarEq,
    OpLtLt,
    OpGtGt,
    OpPipeGt,
    OpLtPipe,

    // Delimiters
    OBrace,
    CBrace,
    OParen,
    CParen,
    OBrack,
    CBrack,

    // Special markers
    StartOfFile,
    EndOfFile,

    // For violations and other weirdness
    ByteOrderMark,
    Error,
}