use std::cmp::PartialEq;
use std::fmt::{self, Debug};

pub(crate) enum SExpr<'a> {
    Wildcard,
    Label(&'a str),
    Container(Vec<SExpr<'a>>),
}

impl<'a> PartialEq for SExpr<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Wildcard, _) | (_, Self::Wildcard) => true,
            (Self::Label(a), Self::Label(b)) => a == b,
            (Self::Container(a), Self::Container(b)) => a == b,
            _ => false,
        }
    }
}

impl Debug for SExpr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SExpr::Wildcard => write!(f, "_")?,
            SExpr::Container(items) => {
                write!(f, "(")?;
                let mut first = true;
                for item in items {
                    if !first {
                        write!(f, " ")?;
                    }
                    first = false;
                    write!(f, "{:?}", item)?;
                }
                write!(f, ")")?;
            }
            Self::Label(label) => write!(f, "{}", label)?,
        }
        Ok(())
    }
}

impl<'a> SExpr<'a> {
    fn new(iter: &mut impl Iterator<Item = &'a str>) -> Self {
        let mut container = vec![];
        loop {
            match iter.next().unwrap() {
                "_" => container.push(Self::Wildcard),
                "(" => container.push(Self::new(iter)),
                ")" => return Self::Container(container),
                ident => container.push(Self::Label(ident)),
            }
        }
    }
}

impl<'a> FromIterator<&'a str> for SExpr<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        assert_eq!("(", iter.next().unwrap());
        let sexpr = SExpr::new(&mut iter);
        assert_eq!(None, iter.next());
        sexpr
    }
}

#[allow(dead_code)]
pub(crate) fn normalize_sexpr(expr: &str) -> String {
    expr.replace('[', "(")
        .replace(']', ")")
        .replace('(', " ( ")
        .replace(')', " ) ")
}

#[macro_export]
macro_rules! test_parse {
    ($name:ident : $src:literal => $path:path => $sexp:literal) => { test_parse!($name : $src |parser| $path(&mut parser) => $sexp); };

    ($name:ident : $src:literal |$parser:ident| $parse:expr => $sexp:literal) => {
        #[test]
        fn $name() {
            use crate::{Parser, PrettyPrintSExpr};
            use trilogy_scanner::TokenType::*;
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = Parser::new(scanner);
            $parser.expect(StartOfFile).unwrap();
            let parse = $parse.unwrap();
            let mut allocator = pretty::RcAllocator;
            let sexpr = format!("{}", parse.pretty_print_sexpr(&mut allocator).pretty(100));
            let parsed = crate::test::normalize_sexpr(&sexpr);
            let expected = crate::test::normalize_sexpr($sexp);
            $parser.expect(EndOfFile).unwrap();
            assert!($parser.errors.is_empty());
            assert_eq!(parsed.split_ascii_whitespace().collect::<crate::test::SExpr>(), expected.split_ascii_whitespace().collect::<crate::test::SExpr>());
        }
    };
}

#[macro_export]
macro_rules! test_parse_error {
    ($name:ident : $src:literal => $path:path => $error:literal) => { test_parse_error!($name : $src |parser| $path(&mut parser) => $error); };
    ($name:ident : $src:literal => $path:path) => { test_parse_error!($name : $src |parser| $path(&mut parser)); };

    ($name:ident : $src:literal |$parser:ident| $parse:expr => $error:literal) => {
        #[test]
        fn $name() {
            use crate::Parser;
            use trilogy_scanner::TokenType::*;
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = Parser::new(scanner);
            $parser.expect(StartOfFile).unwrap();
            let result = $parse;
            if result.is_ok() && $parser.errors.is_empty() {
                assert!($parser.expect(EndOfFile).is_err());
            } else {
                assert_eq!($parser.errors.first().unwrap().message(), $error);
            }
        }
    };

    ($name:ident : $src:literal |$parser:ident| $parse:expr) => {
        #[test]
        fn $name() {
            use crate::Parser;
            use trilogy_scanner::TokenType::*;
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = Parser::new(scanner);
            $parser.expect(StartOfFile).unwrap();
            let result = $parse;
            if result.is_ok() && $parser.errors.is_empty() {
                assert!($parser.expect(EndOfFile).is_err());
            } else {
                assert!(!$parser.errors.is_empty());
            }
        }
    };
}
