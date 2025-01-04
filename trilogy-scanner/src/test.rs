use crate::{Scanner, Token, TokenType::*, TokenValue};
use source_span::{Position, Span};

#[test]
fn spans_accurate() {
    let src = "one two\n    three\nfour";
    let mut scanner = Scanner::new(src);
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: StartOfFile,
            value: None,
            span: Span::new(
                Position::new(0, 0),
                Position::new(0, 0),
                Position::new(0, 0),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: Identifier,
            value: Some(TokenValue::String("one".to_owned())),
            span: Span::new(
                Position::new(0, 0),
                Position::new(0, 2),
                Position::new(0, 3),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: Space,
            value: None,
            span: Span::new(
                Position::new(0, 3),
                Position::new(0, 3),
                Position::new(0, 4),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: Identifier,
            value: Some(TokenValue::String("two".to_owned())),
            span: Span::new(
                Position::new(0, 4),
                Position::new(0, 6),
                Position::new(0, 7),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: EndOfLine,
            value: None,
            span: Span::new(
                Position::new(0, 7),
                Position::new(0, 7),
                Position::new(1, 0),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: Space,
            value: None,
            span: Span::new(
                Position::new(1, 0),
                Position::new(1, 3),
                Position::new(1, 4),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: Identifier,
            value: Some(TokenValue::String("three".to_owned())),
            span: Span::new(
                Position::new(1, 4),
                Position::new(1, 8),
                Position::new(1, 9),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: EndOfLine,
            value: None,
            span: Span::new(
                Position::new(1, 9),
                Position::new(1, 9),
                Position::new(2, 0),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: Identifier,
            value: Some(TokenValue::String("four".to_owned())),
            span: Span::new(
                Position::new(2, 0),
                Position::new(2, 3),
                Position::new(2, 4),
            ),
        },
    );
    assert_eq!(
        scanner.next().unwrap(),
        Token {
            token_type: EndOfFile,
            value: None,
            span: Span::new(
                Position::new(2, 4),
                Position::new(2, 4),
                Position::new(2, 4),
            ),
        },
    );
}

macro_rules! test_tokenize {
    ($name:ident => $src:literal = [$($tok:ident)+]) => {
        #[test]
        fn $name() {
            let tokens = Scanner::new($src)
                .into_iter()
                .map(|token| token.token_type)
                .collect::<Vec<_>>();
            assert_eq!(tokens.as_slice(), [StartOfFile, $($tok),+, EndOfFile]);
        }
    };
}

#[test]
fn identifier_eq_keyword_error() {
    let tokens = Scanner::new("for=")
        .map(|token| token.token_type)
        .collect::<Vec<_>>();
    assert_eq!(tokens.as_slice(), [StartOfFile, Error, EndOfFile]);
}

test_tokenize!(discard => "_" = [Discard]);
test_tokenize!(underscore_ident => "_abc" = [Identifier]);

test_tokenize!(identifier => "hello_world" = [Identifier]);
test_tokenize!(identifier_upper => "HELLO_world" = [Identifier]);
test_tokenize!(identifier_num => "hello123" = [Identifier]);
test_tokenize!(identifiers => "hello world" = [Identifier Space Identifier]);
test_tokenize!(partial_keyword => "an" = [Identifier]);
test_tokenize!(extended_keyword => "matches" = [Identifier]);
test_tokenize!(identifier_eq => "push=" = [IdentifierEq]);
test_tokenize!(identifier_bang => "push!" = [Identifier OpBang]);
test_tokenize!(identifier_paren => "push(" = [Identifier OParen]);

test_tokenize!(kw_and => "and" = [KwAnd]);
test_tokenize!(kw_assert => "assert" = [KwAssert]);
test_tokenize!(kw_as => "as" = [KwAs]);
test_tokenize!(kw_at => "at" = [KwAt]);
test_tokenize!(kw_become => "become" = [KwBecome]);
test_tokenize!(kw_break => "break" = [KwBreak]);
test_tokenize!(kw_cancel => "cancel" = [KwCancel]);
test_tokenize!(kw_continue => "continue" = [KwContinue]);
test_tokenize!(kw_case => "case" = [KwCase]);
test_tokenize!(kw_do => "do" = [KwDo]);
test_tokenize!(kw_else => "else" = [KwElse]);
test_tokenize!(kw_end => "end" = [KwEnd]);
test_tokenize!(kw_exit => "exit" = [KwExit]);
test_tokenize!(kw_export => "export" = [KwExport]);
test_tokenize!(kw_false => "false" = [KwFalse]);
test_tokenize!(kw_fn => "fn" = [KwFn]);
test_tokenize!(kw_for => "for" = [KwFor]);
test_tokenize!(kw_func => "func" = [KwFunc]);
test_tokenize!(kw_if => "if" = [KwIf]);
test_tokenize!(kw_in => "in" = [KwIn]);
test_tokenize!(kw_is => "is" = [KwIs]);
test_tokenize!(kw_let => "let" = [KwLet]);
test_tokenize!(kw_match => "match" = [KwMatch]);
test_tokenize!(kw_module => "module" = [KwModule]);
test_tokenize!(kw_mut => "mut" = [KwMut]);
test_tokenize!(kw_next => "next" = [KwNext]);
test_tokenize!(kw_not => "not" = [KwNot]);
test_tokenize!(kw_or => "or" = [KwOr]);
test_tokenize!(kw_pass => "pass" = [KwPass]);
test_tokenize!(kw_proc => "proc" = [KwProc]);
test_tokenize!(kw_qy => "qy" = [KwQy]);
test_tokenize!(kw_resume => "resume" = [KwResume]);
test_tokenize!(kw_return => "return" = [KwReturn]);
test_tokenize!(kw_rule => "rule" = [KwRule]);
test_tokenize!(kw_super => "super" = [KwSuper]);
test_tokenize!(kw_test => "test" = [KwTest]);
test_tokenize!(kw_then => "then" = [KwThen]);
test_tokenize!(kw_true => "true" = [KwTrue]);
test_tokenize!(kw_typeof => "typeof" = [KwTypeof]);
test_tokenize!(kw_unit => "unit" = [KwUnit]);
test_tokenize!(kw_use => "use" = [KwUse]);
test_tokenize!(kw_when => "when" = [KwWhen]);
test_tokenize!(kw_while => "while" = [KwWhile]);
test_tokenize!(kw_with => "with" = [KwWith]);
test_tokenize!(kw_yield => "yield" = [KwYield]);

test_tokenize!(op_eq => "=" = [OpEq]);
test_tokenize!(op_lt => "<" = [OpLt]);
test_tokenize!(op_gt => ">" = [OpGt]);
test_tokenize!(op_eq_eq => "==" = [OpEqEq]);
test_tokenize!(op_bang_eq => "!=" = [OpBangEq]);
test_tokenize!(op_eq_eq_eq => "===" = [OpEqEqEq]);
test_tokenize!(op_bang_eq_eq => "!==" = [OpBangEqEq]);
test_tokenize!(op_lt_eq => "<=" = [OpLtEq]);
test_tokenize!(op_gt_eq => ">=" = [OpGtEq]);
test_tokenize!(op_bang => "!" = [OpBang]);
test_tokenize!(op_amp_amp => "&&" = [OpAmpAmp]);
test_tokenize!(op_pipe_pipe => "||" = [OpPipePipe]);
test_tokenize!(op_amp_amp_eq => "&&=" = [OpAmpAmpEq]);
test_tokenize!(op_pipe_pipe_eq => "||=" = [OpPipePipeEq]);
test_tokenize!(op_amp => "&" = [OpAmp]);
test_tokenize!(op_pipe => "|" = [OpPipe]);
test_tokenize!(op_caret => "^" = [OpCaret]);
test_tokenize!(op_tilde => "~" = [OpTilde]);
test_tokenize!(op_shr => "~>" = [OpShr]);
test_tokenize!(op_shl => "<~" = [OpShl]);
test_tokenize!(op_amp_eq => "&=" = [OpAmpEq]);
test_tokenize!(op_pipe_eq => "|=" = [OpPipeEq]);
test_tokenize!(op_caret_eq => "^=" = [OpCaretEq]);
test_tokenize!(op_shr_eq => "~>=" = [OpShrEq]);
test_tokenize!(op_shl_eq => "<~=" = [OpShlEq]);
test_tokenize!(op_glue => "<>" = [OpGlue]);
test_tokenize!(op_glue_eq => "<>=" = [OpGlueEq]);
test_tokenize!(op_colon => ":" = [OpColon]);
test_tokenize!(op_colon_eq => ":=" = [OpColonEq]);
test_tokenize!(op_colon_colon => "::" = [OpColonColon]);
test_tokenize!(op_plus => "+" = [OpPlus]);
test_tokenize!(op_minus => "-" = [OpMinus]);
test_tokenize!(op_star => "*" = [OpStar]);
test_tokenize!(op_slash => "/" = [OpSlash]);
test_tokenize!(op_slash_slash => "//" = [OpSlashSlash]);
test_tokenize!(op_percent => "%" = [OpPercent]);
test_tokenize!(op_star_star => "**" = [OpStarStar]);
test_tokenize!(op_plus_eq => "+=" = [OpPlusEq]);
test_tokenize!(op_minus_eq => "-=" = [OpMinusEq]);
test_tokenize!(op_star_eq => "*=" = [OpStarEq]);
test_tokenize!(op_slash_eq => "/=" = [OpSlashEq]);
test_tokenize!(op_slash_slash_eq => "//=" = [OpSlashSlashEq]);
test_tokenize!(op_percent_eq => "%=" = [OpPercentEq]);
test_tokenize!(op_star_star_eq => "**=" = [OpStarStarEq]);
test_tokenize!(op_dot => "." = [OpDot]);
test_tokenize!(op_dot_eq => ".=" = [OpDotEq]);
test_tokenize!(op_dot_dot => ".." = [OpDotDot]);
test_tokenize!(op_comma => "," = [OpComma]);
test_tokenize!(op_semi => ";" = [OpSemi]);
test_tokenize!(op_left_arrow => "<-" = [OpLeftArrow]);
test_tokenize!(op_fat_arrow => "=>" = [OpFatArrow]);
test_tokenize!(op_lt_lt => "<<" = [OpLtLt]);
test_tokenize!(op_gt_gt => ">>" = [OpGtGt]);
test_tokenize!(op_lt_lt_eq => "<<=" = [OpLtLtEq]);
test_tokenize!(op_gt_gt_eq => ">>=" = [OpGtGtEq]);
test_tokenize!(op_pipe_gt => "|>" = [OpPipeGt]);
test_tokenize!(op_lt_pipe => "<|" = [OpLtPipe]);
test_tokenize!(op_question => "?" = [OpQuestion]);
test_tokenize!(op_tilde_eq => "~=" = [OpTildeEq]);
test_tokenize!(op_right_arrow => "->" = [OpRightArrow]);

test_tokenize!(integer => "1234567890" = [Numeric]);
test_tokenize!(integer_underscores => "123_456_789_0" = [Numeric]);
test_tokenize!(zero => "0" = [Numeric]);
test_tokenize!(hex => "0x0123456789abcdef" = [Numeric]);
test_tokenize!(hex_underscores => "0x0123_456_789_abc_def" = [Numeric]);
test_tokenize!(hex_upper => "0x0123456789ABCDEF" = [Numeric]);
test_tokenize!(no_hex_float => "0x0123.12345" = [Numeric OpDot Numeric]);
test_tokenize!(hex_rational => "0x123/0x456" = [Numeric]);
test_tokenize!(oct => "0o01234567" = [Numeric]);
test_tokenize!(oct_underscores => "0o012_34_567" = [Numeric]);
test_tokenize!(no_oct_float => "0o01.456" = [Numeric OpDot Numeric]);
test_tokenize!(oct_rational => "0o123/0o456" = [Numeric]);
test_tokenize!(bin => "0b01" = [Numeric]);
test_tokenize!(bin_underscores => "0b0_1" = [Numeric]);
test_tokenize!(no_bin_float => "0b01.10" = [Numeric OpDot Numeric]);
test_tokenize!(bin_rational => "0b01/0b10" = [Numeric]);
test_tokenize!(float => "123.456" = [Numeric]);
test_tokenize!(float_zero => "0.0" = [Numeric]);
test_tokenize!(rational => "1/1234" = [Numeric]);
test_tokenize!(rational_zero => "0/1" = [Numeric]);
test_tokenize!(rational_div_by_zero => "1/0" = [Error]);
test_tokenize!(complex => "5i1" = [Numeric]);
test_tokenize!(complex_zero => "0i0" = [Numeric]);
test_tokenize!(complex_float => "0.5i3" = [Numeric]);
test_tokenize!(complex_rational => "1/5i1/3" = [Numeric]);
test_tokenize!(no_rational_complex => "1i5/1i3" = [Error Identifier]);
test_tokenize!(complex_no_prefix => "i123" = [Identifier]);
test_tokenize!(no_float_rational => "0.5/1.3" = [Numeric OpSlash Numeric]);

test_tokenize!(no_float_start_dot => ".5" = [OpDot Numeric]);
test_tokenize!(no_float_end_dot => "5." = [Numeric OpDot]);

test_tokenize!(bits_bin => "0bb01" = [Bits]);
test_tokenize!(bits_bin_empty => "0bb" = [Bits]);
test_tokenize!(bits_bin_underscores => "0bb0_1" = [Bits]);
test_tokenize!(bits_hex => "0xb0123456789abcdef" = [Bits]);
test_tokenize!(bits_hex_empty => "0xb" = [Bits]);
test_tokenize!(bits_hex_underscores => "0xb012_3456_789a_bcdef" = [Bits]);
test_tokenize!(bits_hex_upper => "0xb0123456789ABCDEF" = [Bits]);
test_tokenize!(bits_oct => "0ob01234567" = [Bits]);
test_tokenize!(bits_oct_empty => "0ob" = [Bits]);
test_tokenize!(bits_oct_underscores => "0ob01_2345_67" = [Bits]);
test_tokenize!(no_bits_float => "0bb1.1" = [Bits OpDot Numeric]);
test_tokenize!(no_bits_rational => "0bb01/1" = [Bits OpSlash Numeric]);
test_tokenize!(no_bits_rational_denom => "1/0bb01" = [Error]);
test_tokenize!(no_bits_complex => "0bb01i1" = [Error Identifier]);
test_tokenize!(no_bits_complex_imaginary => "1i0bb01" = [Error]);

test_tokenize!(atom => "'hello" = [Atom]);
test_tokenize!(atom_short => "'h" = [Atom]);
test_tokenize!(atom_long => "'hello_world" = [Atom]);
test_tokenize!(atom_nums => "'hello123" = [Atom]);
test_tokenize!(atom_struct => "'hello('world)" = [Atom OParen Atom CParen]);
test_tokenize!(atom_escape_invalid => "'\\u{ffff}" = [Error]);
test_tokenize!(atom_special_char_invalid => "'{" = [Error]);

test_tokenize!(character => "'h'" = [Character]);
test_tokenize!(character_escape_n => "'\\n'" = [Character]);
test_tokenize!(character_escape_t => "'\\t'" = [Character]);
test_tokenize!(character_escape_r => "'\\r'" = [Character]);
test_tokenize!(character_escape_0 => "'\\0'" = [Character]);
test_tokenize!(character_escape_slash => "'\\\\'" = [Character]);
test_tokenize!(character_escape_apos => "'\\''" = [Character]);
test_tokenize!(character_escape_ascii => "'\\xff'" = [Character]);
test_tokenize!(character_escape_unicode_short => "'\\u{ff}'" = [Character]);
test_tokenize!(character_escape_unicode => "'\\u{ffff}'" = [Character]);
test_tokenize!(character_escape_unicode_long => "'\\u{fffff}'" = [Character]);
test_tokenize!(character_quote => "'\"'" = [Character]);
test_tokenize!(character_escape_quote => "'\\\"'" = [Character]);
test_tokenize!(character_escape_dollar => "'\\$'" = [Character]);
test_tokenize!(character_space => "' '" = [Character]);
test_tokenize!(character_escape_invalid => "'\\a'" = [Error Error]);
test_tokenize!(character_escape_incomplete => "'\\" = [Error]);
test_tokenize!(character_incomplete => "'" = [Error]);

test_tokenize!(string => r#""hello""# = [String]);
test_tokenize!(string_spaced => r#""hello world""# = [String]);
test_tokenize!(string_escape_n => r#""hello\\nworld""# = [String]);
test_tokenize!(string_escape_t => r#""hello\\tworld""# = [String]);
test_tokenize!(string_escape_r => r#""hello\\rworld""# = [String]);
test_tokenize!(string_escape_0 => r#""hello\\0world""# = [String]);
test_tokenize!(string_escape_slash => r#""hello\\world""# = [String]);
test_tokenize!(string_escape_ascii => r#""hello\xffworld""# = [String]);
test_tokenize!(string_escape_unicode => r#""hello\u{ffff}world""# = [String]);
test_tokenize!(string_escape_quote => r#""hello\"world""# = [String]);
test_tokenize!(string_escape_apos => r#""hello\'world""# = [String]);
test_tokenize!(string_escape_dollar => r#""hello\$world""# = [String]);
test_tokenize!(string_multiline => "\"hello\nworld\"" = [String]);
test_tokenize!(string_escape_invalid => r#""hello\aworld""# = [Error Identifier Error]);
test_tokenize!(string_escape_incomplete => r#""hello\"# = [Error]);
test_tokenize!(string_escape_quote_incomplete => r#""hello\""# = [Error]);
test_tokenize!(string_incomplete => r#""hello"# = [Error]);

test_tokenize!(dollar_string => r#"$"hello""# = [DollarString]);
test_tokenize!(template => r#"$"hello${3}world${4}end""# = [TemplateStart Numeric TemplateContinue Numeric TemplateEnd]);

test_tokenize!(dollar_oparen => "$(" = [DollarOParen]);
test_tokenize!(bang_oparen => "!(" = [OpBang OParen]);
test_tokenize!(spaced_bang_oparen => "! (" = [OpBang Space OParen]); // a specifically used fact
test_tokenize!(oparen => "(" = [OParen]);
test_tokenize!(cparen => ")" = [CParen]);
test_tokenize!(obrack => "[" = [OBrack]);
test_tokenize!(cbrack => "]" = [CBrack]);
test_tokenize!(obrace => "{" = [OBrace]);
test_tokenize!(cbrace => "}" = [CBrace]);
test_tokenize!(obrackpipe => "[|" = [OBrackPipe]);
test_tokenize!(cbrackpipe => "|]" = [CBrackPipe]);
test_tokenize!(obracepipe => "{|" = [OBracePipe]);
test_tokenize!(cbracepipe => "|}" = [CBracePipe]);

test_tokenize!(onespace => " " = [Space]);
test_tokenize!(manyspaces => "    " = [Space]);
test_tokenize!(newline => "\n" = [EndOfLine]);
test_tokenize!(crlf => "\r\n" = [EndOfLine]);
test_tokenize!(cr_ignored => "   \r   " = [Space Space]);
test_tokenize!(spaced_newline => "   \n   " = [Space EndOfLine Space]);

test_tokenize!(comment => "# hello\n" = [CommentLine]);
test_tokenize!(comment_ended => "# hello\nhello" = [CommentLine Identifier]);
test_tokenize!(comment_eof => "# hello" = [CommentLine]);
test_tokenize!(comment_inline => "#- Hello -#world" = [CommentInline Identifier]);
test_tokenize!(comment_block => "#- He\nllo -#" = [CommentBlock]);
test_tokenize!(comment_nested => "#- H #- i -# H -#" = [CommentInline]);
test_tokenize!(comment_fake_nested => "#- - # -#" = [CommentInline]);
test_tokenize!(comment_in_comment => "# #- -\n# -#" = [CommentLine CommentLine]);
test_tokenize!(doc_comment_inner => "#! Hello\n" = [DocInner]);
test_tokenize!(doc_comment_outer => "## Hello\n" = [DocOuter]);

test_tokenize!(kw_async => "async" = [KwAsync]);
test_tokenize!(kw_await => "await" = [KwAwait]);
test_tokenize!(kw_catch => "catch" = [KwCatch]);
test_tokenize!(kw_class => "class" = [KwClass]);
test_tokenize!(kw_const => "const" = [KwConst]);
test_tokenize!(kw_data => "data" = [KwData]);
test_tokenize!(kw_defer => "defer" = [KwDefer]);
test_tokenize!(kw_enum => "enum" = [KwEnum]);
test_tokenize!(kw_except => "except" = [KwExcept]);
test_tokenize!(kw_extends => "extends" = [KwExtends]);
test_tokenize!(kw_implements => "implements" = [KwImplements]);
test_tokenize!(kw_import => "import" = [KwImport]);
test_tokenize!(kw_inline => "inline" = [KwInline]);
test_tokenize!(kw_instanceof => "instanceof" = [KwInstanceof]);
test_tokenize!(kw_interface => "interface" = [KwInterface]);
test_tokenize!(kw_lazy => "lazy" = [KwLazy]);
test_tokenize!(kw_lens => "lens" = [KwLens]);
test_tokenize!(kw_loop => "loop" = [KwLoop]);
test_tokenize!(kw_macro => "macro" = [KwMacro]);
test_tokenize!(kw_oper => "oper" = [KwOper]);
test_tokenize!(kw_prec => "prec" = [KwPrec]);
test_tokenize!(kw_protocol => "protocol" = [KwProtocol]);
test_tokenize!(kw_static => "static" = [KwStatic]);
test_tokenize!(kw_struct => "struct" = [KwStruct]);
test_tokenize!(kw_switch => "switch" = [KwSwitch]);
test_tokenize!(kw_tag => "tag" = [KwTag]);
test_tokenize!(kw_trait => "trait" = [KwTrait]);
test_tokenize!(kw_try => "try" = [KwTry]);
test_tokenize!(kw_type => "type" = [KwType]);
test_tokenize!(kw_unless => "unless" = [KwUnless]);
test_tokenize!(kw_until => "until" = [KwUntil]);
test_tokenize!(kw_where => "where" = [KwWhere]);

test_tokenize!(invalid_nesting => "(]" = [OParen CBrack]);
test_tokenize!(invalid_char => "`" = [Error]);

test_tokenize!(byte_order_mark => "\u{feff}" = [ByteOrderMark]);
