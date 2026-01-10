//! Unit test helpers for the Trilogy AST and parser.

#[macro_export]
macro_rules! test_parse {
    ($name:ident : $src:literal => $path:path => $pattern:pat) => { test_parse!($name : $src |parser| $path(&mut parser).unwrap() => $pattern); };

    ($name:ident : $src:literal |$parser:ident| $parse:expr => $pattern:pat) => {
        #[test]
        fn $name() {
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = $crate::Parser::new(scanner);
            $parser.expect(trilogy_scanner::TokenType::StartOfFile).unwrap();
            let parsed = $parse;
            $parser.expect(trilogy_scanner::TokenType::EndOfFile).unwrap();
            assert!($parser.errors.is_empty());
            assert!(matches!(parsed, $pattern));
        }
    };
}

#[macro_export]
macro_rules! parse {
    ($src:literal => $path:path) => {{
        use trilogy_scanner::TokenType::*;
        use $crate::Parser;
        let scanner = trilogy_scanner::Scanner::new($src);
        let mut parser = Parser::new(scanner);
        parser.expect(StartOfFile).unwrap();
        let parse = $path(&mut parser).unwrap();
        parser
            .expect(EndOfFile)
            .expect("whole source should be parsed");
        parse
    }};
}

#[macro_export]
macro_rules! test_parse_error {
    ($name:ident : $src:literal => $path:path => $error:literal) => { test_parse_error!($name : $src |parser| $path(&mut parser) => $error); };
    ($name:ident : $src:literal => $path:path) => { test_parse_error!($name : $src |parser| $path(&mut parser)); };

    ($name:ident : $src:literal |$parser:ident| $parse:expr => $error:literal) => {
        #[test]
        fn $name() {
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = $crate::Parser::new(scanner);
            $parser.expect(trilogy_scanner::TokenType::StartOfFile).unwrap();
            let _ = $parse;
            assert!($parser.errors.first().expect("parse should have reported an error message").to_string().ends_with($error));
        }
    };

    ($name:ident : $src:literal |$parser:ident| $parse:expr) => {
        #[test]
        fn $name() {
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = $crate::Parser::new(scanner);
            $parser.expect(trilogy_scanner::TokenType::StartOfFile).unwrap();
            let result = $parse;
            if result.is_ok() && $parser.errors.is_empty() {
                assert!($parser.expect(trilogy_scanner::TokenType::EndOfFile).is_err());
            } else {
                assert!(!$parser.errors.is_empty());
            }
        }
    };
}

// Special for testing Document, which takes the start/end into account.
// Maybe a sign that Document should be wrapped by something else...

#[macro_export]
macro_rules! test_parse_whole {
    ($name:ident : $src:literal => $path:path => $pattern:pat) => { test_parse_whole!($name : $src |parser| $path(&mut parser) => $pattern); };

    ($name:ident : $src:literal |$parser:ident| $parse:expr => $pattern:pat) => {
        #[test]
        fn $name() {
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = $crate::Parser::new(scanner);
            let parsed = $parse;
            assert!($parser.errors.is_empty());
            assert!(matches!(parsed, $pattern));
        }
    };
}

#[macro_export]
macro_rules! test_parse_whole_error {
    ($name:ident : $src:literal => $path:path => $error:literal) => { test_parse_whole_error!($name : $src |parser| $path(&mut parser) => $error); };
    ($name:ident : $src:literal => $path:path) => { test_parse_whole_error!($name : $src |parser| $path(&mut parser)); };

    ($name:ident : $src:literal |$parser:ident| $parse:expr => $error:literal) => {
        #[test]
        fn $name() {
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = $crate::Parser::new(scanner);
            $parse;
            assert!($parser.errors.first().expect("parse should have reported an error message").to_string().ends_with($error));
        }
    };

    ($name:ident : $src:literal |$parser:ident| $parse:expr) => {
        #[test]
        fn $name() {
            use trilogy_scanner::TokenType::*;
            let scanner = trilogy_scanner::Scanner::new($src);
            let mut $parser = $crate::Parser::new(scanner);
            $parse;
            assert!(!$parser.errors.is_empty());
        }
    };
}
