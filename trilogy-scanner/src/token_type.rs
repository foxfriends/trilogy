/// All the types of tokens recorgnized by the [`Scanner`][crate::Scanner]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TokenType {
    // Keywords
    /// `and`
    KwAnd,
    /// `assert`
    KwAssert,
    /// `as`
    KwAs,
    /// `at`
    KwAt,
    /// `become`
    KwBecome,
    /// `break`
    KwBreak,
    /// `cancel`
    KwCancel,
    /// `continue`
    KwContinue,
    /// `case`
    KwCase,
    /// `const`
    KwConst,
    /// `do`
    KwDo,
    /// `else`
    KwElse,
    /// `end`
    KwEnd,
    /// `exit`
    KwExit,
    /// `export`
    KwExport,
    /// `extern`
    KwExtern,
    /// `false`
    KwFalse,
    /// `fn`
    KwFn,
    /// `for`
    KwFor,
    /// `func`
    KwFunc,
    /// `if`
    KwIf,
    /// `in`
    KwIn,
    /// `is`
    KwIs,
    /// `let`
    KwLet,
    /// `match`
    KwMatch,
    /// `module`
    KwModule,
    /// `mut`
    KwMut,
    /// `next`
    KwNext,
    /// `not`
    KwNot,
    /// `or`
    KwOr,
    /// `pass`
    KwPass,
    /// `proc`
    KwProc,
    /// `qy`
    KwQy,
    /// `resume`
    KwResume,
    /// `return`
    KwReturn,
    /// `rule`
    KwRule,
    /// `test`
    KwTest,
    /// `then`
    KwThen,
    /// `true`
    KwTrue,
    /// `unit`
    KwUnit,
    /// `when`
    KwWhen,
    /// `while`
    KwWhile,
    /// `with`
    KwWith,
    /// `yield`
    KwYield,

    // Reserved Words
    /// `async`
    KwAsync,
    /// `await`
    KwAwait,
    /// `catch`
    KwCatch,
    /// `class`
    KwClass,
    /// `data`
    KwData,
    /// `defer`
    KwDefer,
    /// `enum`
    KwEnum,
    /// `except`
    KwExcept,
    /// `extends`
    KwExtends,
    /// `implements`
    KwImplements,
    /// `import`
    KwImport,
    /// `inline`
    KwInline,
    /// `instanceof`
    KwInstanceof,
    /// `interface`
    KwInterface,
    /// `lazy`
    KwLazy,
    /// `lens`
    KwLens,
    /// `loop`
    KwLoop,
    /// `macro`
    KwMacro,
    /// `oper`
    KwOper,
    /// `prec`
    KwPrec,
    /// `protocol`
    KwProtocol,
    /// `static`
    KwStatic,
    /// `struct`
    KwStruct,
    /// `super`
    KwSuper,
    /// `switch`
    KwSwitch,
    /// `tag`
    KwTag,
    /// `trait`
    KwTrait,
    /// `try`
    KwTry,
    /// `type`
    KwType,
    /// `typeof`
    KwTypeof,
    /// `unless`
    KwUnless,
    /// `until`
    KwUntil,
    /// `use`
    KwUse,
    /// `where`
    KwWhere,

    // Identifiers
    /// `_`
    Discard,
    /// An identifier followed by `=` (e.g. `push=`)
    IdentifierEq,
    /// Any single bare word (e.g. `hello`)
    Identifier,

    // Literals
    /// An atom literal (e.g. `'hello`)
    Atom,
    /// A number literal (e.g. `123` or `0.5` or `-5/4i-1/2`)
    Numeric,
    /// A string literal (e.g. `"hello world"`)
    String,
    /// A template string with no interpolations (e.g. `$"hello"`)
    DollarString,
    /// The beginning of a template string (e.g. `$"hello{`)
    TemplateStart,
    /// The middle of a template string (e.g. `}hello{`)
    TemplateContinue,
    /// The end of a template string (e.g. `}hello"`)
    TemplateEnd,
    /// A character literal (e.g. `'a'`)
    Character,
    /// A bits literal (e.g. `0b10101`)
    Bits,

    // Whitespace
    /// The end of a line (the `\n` character)
    EndOfLine,
    /// Any contiguous string of whitespace characters
    Space,

    // Comments
    /// An inline block comment (e.g. `#- hello -#`)
    CommentInline,
    /// A multiline block comment
    /// ```txt
    /// #-
    ///   hello
    /// -#
    /// ```
    CommentBlock,
    /// A single line comment (e.g. `# hello`)
    CommentLine,
    /// An outer documentation comment (e.g. `## hello`)
    DocOuter,
    /// An inner documentation comment (e.g. `#! hello`)
    DocInner,

    // Punctuation
    /// `=`
    OpEq,
    /// `<`
    OpLt,
    /// `>`
    OpGt,
    /// `==`
    OpEqEq,
    /// `===`
    OpEqEqEq,
    /// `<=`
    OpLtEq,
    /// `>=`
    OpGtEq,
    /// `&`
    OpAmp,
    /// `|`
    OpPipe,
    /// `^`
    OpCaret,
    /// `~`
    OpTilde,
    /// `~>`
    OpShr,
    /// `<~`
    OpShl,
    /// `!`
    OpBang,
    /// `!=`
    OpBangEq,
    /// `!==`
    OpBangEqEq,
    /// `&&`
    OpAmpAmp,
    /// `&&=`
    OpAmpAmpEq,
    /// `||`
    OpPipePipe,
    /// `||=`
    OpPipePipeEq,
    /// `&=`
    OpAmpEq,
    /// `|=`
    OpPipeEq,
    /// `^=`
    OpCaretEq,
    /// `~>=`
    OpShrEq,
    /// `<~=`
    OpShlEq,
    /// `.`
    OpDot,
    /// `.=`
    OpDotEq,
    /// `..`
    OpDotDot,
    /// `,`
    OpComma,
    /// `:`
    OpColon,
    /// `:=`
    OpColonEq,
    /// `::`
    OpColonColon,
    /// `<-`
    OpLeftArrow,
    /// `->`
    OpRightArrow,
    /// `=>`
    OpFatArrow,
    /// `;`
    OpSemi,
    /// `+`
    OpPlus,
    /// `-`
    OpMinus,
    /// `*`
    OpStar,
    /// `/`
    OpSlash,
    /// `//`
    OpSlashSlash,
    /// `%`
    OpPercent,
    /// `**`
    OpStarStar,
    /// `+=`
    OpPlusEq,
    /// `-=`
    OpMinusEq,
    /// `*=`
    OpStarEq,
    /// `/=`
    OpSlashEq,
    /// `//=`
    OpSlashSlashEq,
    /// `%=`
    OpPercentEq,
    /// `**=`
    OpStarStarEq,
    /// `<<`
    OpLtLt,
    /// `<<=`
    OpLtLtEq,
    /// `>>`
    OpGtGt,
    /// `>>=`
    OpGtGtEq,
    /// `|>`
    OpPipeGt,
    /// `<|`
    OpLtPipe,
    /// `<>`
    OpGlue,
    /// `<>=`
    OpGlueEq,
    /// `~=`
    OpTildeEq,

    /// `?`
    OpQuestion,

    // Delimiters
    /// `{`
    OBrace,
    /// `}`
    CBrace,
    /// `{|`
    OBracePipe,
    /// `|}`
    CBracePipe,
    /// `[`
    OBrack,
    /// `]`
    CBrack,
    /// `[|`
    OBrackPipe,
    /// `|]`
    CBrackPipe,
    /// `(`
    OParen,
    /// `)`
    CParen,

    // Special markers
    /// The beginning of the file (inserted automatically, not a visible token)
    StartOfFile,
    /// The end of the file (inserted automatically, not a visible token)
    EndOfFile,

    // For violations and other weirdness
    /// A unicode byte-order-mark found at the beginning of a file (`0xfeff`)
    ByteOrderMark,
    /// Any invalid token
    Error,
}
