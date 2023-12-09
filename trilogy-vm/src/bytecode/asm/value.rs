use super::string::{escape_sequence, extract_string_prefix};
use crate::atom::AtomInterner;
use crate::{Array, Bits, Number, Record, Set, Struct, Tuple, Value};
use std::collections::{HashMap, HashSet};

impl Value {
    pub(super) fn parse_prefix<'a>(
        s: &'a str,
        atom_interner: &AtomInterner,
    ) -> Option<(Self, &'a str)> {
        match s {
            _ if s.starts_with("unit") => Some((Value::Unit, &s[4..])),
            _ if s.starts_with("true") => Some((Value::Bool(true), &s[4..])),
            _ if s.starts_with("false") => Some((Value::Bool(false), &s[5..])),
            _ if s.starts_with('\'') => {
                if s.starts_with("'\\") {
                    let (ch, s) = escape_sequence(&s[1..])?;
                    let s = s.strip_prefix('\'')?;
                    Some((Value::Char(ch), s))
                } else if &s[2..3] == "'" {
                    Some((Value::Char(s[1..2].parse().ok()?), &s[3..]))
                } else {
                    let s = &s[1..];
                    let atom: String = s
                        .chars()
                        .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
                        .collect();
                    if atom.is_empty() {
                        None
                    } else {
                        let s = &s[atom.len()..];
                        let atom = atom_interner.intern(&atom);
                        if let Some(s) = s.strip_prefix('(') {
                            let (value, s) = Value::parse_prefix(s, atom_interner)?;
                            let s = s.strip_prefix(')')?;
                            Some((Value::from(Struct::new(atom, value)), s))
                        } else {
                            Some((Value::from(atom), s))
                        }
                    }
                }
            }
            _ if s.starts_with('(') => {
                let s = &s[1..];
                let (lhs, s) = Value::parse_prefix(s, atom_interner)?;
                let s = s.strip_prefix(':')?;
                let (rhs, s) = Value::parse_prefix(s, atom_interner)?;
                let s = s.strip_prefix(')')?;
                Some((Value::from(Tuple::new(lhs, rhs)), s))
            }
            _ if s.starts_with('"') => extract_string_prefix(s).map(|(v, s)| (Value::from(v), s)),
            _ if s.starts_with("[|") => {
                let mut set = HashSet::new();
                let mut s = &s[2..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix("|]") {
                        break rest;
                    }
                    if s.is_empty() {
                        return None;
                    }
                    let (value, rest) = Value::parse_prefix(s, atom_interner)?;
                    set.insert(value);
                    if let Some(rest) = rest.strip_prefix("|]") {
                        break rest;
                    }
                    s = rest.strip_prefix(',')?;
                };
                Some((Value::Set(Set::from(set)), s))
            }
            _ if s.starts_with("{|") => {
                let mut map = HashMap::new();
                let mut s = &s[2..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix("|}") {
                        break rest;
                    }
                    if s.is_empty() {
                        return None;
                    }
                    let (key, rest) = Value::parse_prefix(s, atom_interner)?;
                    let rest = rest.strip_prefix("=>")?;
                    let (value, rest) = Value::parse_prefix(rest, atom_interner)?;
                    map.insert(key, value);
                    if let Some(rest) = rest.strip_prefix("|}") {
                        break rest;
                    }
                    s = rest.strip_prefix(',')?;
                };
                Some((Value::Record(Record::from(map)), s))
            }
            _ if s.starts_with('[') => {
                let mut array = vec![];
                let mut s = &s[1..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix(']') {
                        break rest;
                    }
                    if s.is_empty() {
                        return None;
                    }
                    let (value, rest) = Value::parse_prefix(s, atom_interner)?;
                    array.push(value);
                    if let Some(rest) = rest.strip_prefix(']') {
                        break rest;
                    }
                    s = rest.strip_prefix(',')?;
                };
                Some((Value::from(Array::from(array)), s))
            }
            _ if s.starts_with("0b") => {
                let bits: Bits = s[2..]
                    .chars()
                    .take_while(|&c| c == '0' || c == '1')
                    .map(|ch| ch == '1')
                    .collect();
                let s = &s[bits.len() + 2..];
                Some((Value::from(bits), s))
            }
            s => {
                let numberlike: String = s
                    .chars()
                    .take_while(|&c| {
                        // All the valid characters of these complex big rationals
                        c.is_numeric()
                            || c == '-'
                            || c == '+'
                            || c == 'i'
                            || c == 'e'
                            || c == '.'
                            || c == 'E'
                            || c == '/'
                    })
                    .collect();
                Some((
                    Value::from(numberlike.parse::<Number>().ok()?),
                    &s[numberlike.len()..],
                ))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::StructuralEq;
    use num::{BigRational, Complex};

    macro_rules! test {
        ($name:ident =>  $input:literal, $value:expr, $tail:literal) => {
            #[test]
            fn $name() {
                let (value, tail) =
                    Value::parse_prefix($input, &mut AtomInterner::default()).unwrap();
                assert!(StructuralEq::eq(&value, &$value.into()));
                assert_eq!(tail, $tail);
            }
        };
    }

    test!(parse_string => r#""hello""#, "hello", "");
    test!(parse_string_escapes => r#""hel\\\x15\u{ff00}lo""#, "hel\\\x15\u{ff00}lo", "");
    test!(parse_string_trailing => r#""hello"123"#, "hello", "123");
    test!(parse_char => "'a'", 'a', "");
    test!(parse_char_escape => r"'\\'", '\\', "");
    test!(parse_number => "123", 123, "");
    test!(parse_number_neg => "-123", -123, "");
    test!(parse_number_rational => "123/123", 1, "");
    test!(parse_number_complex => "123+3i", Complex::new(BigRational::new(123.into(), 1.into()), BigRational::new(3.into(), 1.into())), "");
    test!(parse_number_complex_neg => "123-3i", Complex::new(BigRational::new(123.into(), 1.into()), -BigRational::new(3.into(), 1.into())), "");
    test!(parse_true => "true", true, "");
    test!(parse_false => "false", false, "");
    test!(parse_unit => "unit", (), "");
    test!(parse_array => r#"[1,true,"hello"]"#, vec![Value::from(1), true.into(), "hello".into()], "");
    test!(parse_array_empty => "[]", Vec::<Value>::new(), "");
    test!(parse_set => r#"[|1,2,1|]"#, {
        let mut set = HashSet::<Value>::new();
        set.insert(1.into());
        set.insert(2.into());
        set
    }, "");
    test!(parse_set_empty => r#"[||]"#, HashSet::<Value>::new(), "");
    test!(parse_record => r#"{|"x"=>true,"y"=>5|}"#, {
        let mut map = HashMap::<Value, Value>::new();
        map.insert("x".into(), true.into());
        map.insert("y".into(), 5.into());
        map
    }, "");
    test!(parse_record_empty => r#"{||}"#, HashMap::<Value, Value>::new(), "");
    test!(parse_bits => "0b111011", vec![true, true, true, false, true, true] , "");

    #[test]
    fn parse_atom() {
        let mut atom_interner = AtomInterner::default();
        let atom = atom_interner.intern("hello");
        let (value, tail) = Value::parse_prefix("'hello", &mut atom_interner).unwrap();
        assert!(StructuralEq::eq(&value, &atom.into()));
        assert_eq!(tail, "");
    }

    #[test]
    fn parse_struct() {
        let mut atom_interner = AtomInterner::default();
        let atom = atom_interner.intern("hello");
        let (value, tail) = Value::parse_prefix("'hello(123)", &mut atom_interner).unwrap();
        assert!(StructuralEq::eq(&value, &Struct::new(atom, 123).into()));
        assert_eq!(tail, "");
    }
}
