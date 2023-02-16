#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenType {
    // Keywords
    KwAnd,
    KwAssert,
    KwAs,
    KwAt,
    KwBreak,
    KwCancel,
    KwContinue,
    KwCase,
    KwDo,
    KwElse,
    KwEnd,
    KwExit,
    KwExport,
    KwFalse,
    KwFn,
    KwFor,
    KwFrom,
    KwFunc,
    KwGiven,
    KwIf,
    KwImport,
    KwInvert,
    KwIn,
    KwIs,
    KwLet,
    KwMatch,
    KwModule,
    KwMut,
    KwNext,
    KwNot,
    KwOr,
    KwPass,
    KwProc,
    KwResume,
    KwReturn,
    KwRule,
    KwTest,
    KwThen,
    KwTrue,
    KwUnit,
    KwUse,
    KwWhen,
    KwWhile,
    KwWith,
    KwYield,

    // Reserved Words
    KwAsync,
    KwAwait,
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
    KwLoop,
    KwMacro,
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
    Identifier,

    // Literals
    Atom,
    Numeric,
    String,
    DollarString,
    TemplateStart,
    TemplateContinue,
    TemplateEnd,
    Character,
    Bits,

    // Whitespace
    EndOfLine,
    Space,

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
    OpEqEqEq,
    OpLtEq,
    OpGtEq,
    OpAmp,
    OpPipe,
    OpCaret,
    OpTilde,
    OpShr,
    OpShl,
    OpBang,
    OpBangEq,
    OpBangEqEq,
    OpAmpAmp,
    OpAmpAmpEq,
    OpPipePipe,
    OpPipePipeEq,
    OpAmpEq,
    OpPipeEq,
    OpCaretEq,
    OpShrEq,
    OpShlEq,
    OpDot,
    OpDotEq,
    OpDotDot,
    OpComma,
    OpColon,
    OpColonEq,
    OpColonColon,
    OpLeftArrow,
    OpRightArrow,
    OpFatArrow,
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
    OpStarStarEq,
    OpLtLt,
    OpLtLtEq,
    OpGtGt,
    OpGtGtEq,
    OpPipeGt,
    OpLtPipe,
    OpGlue,
    OpGlueEq,
    OpTildeEq,

    // Unused punctuation
    OpAt,
    OpQuestion,

    // Delimiters
    OBrace,
    CBrace,
    OBracePipe,
    CBracePipe,
    DollarOParen,
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
