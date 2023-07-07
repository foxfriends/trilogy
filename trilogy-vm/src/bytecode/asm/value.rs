use super::error::ValueError;
use super::string::{escape_sequence, extract_string_prefix};
use super::AsmContext;
use crate::{runtime::Procedure, Array, Bits, Record, Set, Struct, Tuple, Value};
use std::collections::{HashMap, HashSet};

impl Value {
    pub(super) fn parse_prefix<'a>(
        s: &'a str,
        context: &mut AsmContext,
    ) -> Result<(Self, &'a str), ValueError> {
        match s {
            _ if s.starts_with("unit") => Ok((Value::Unit, &s[4..])),
            _ if s.starts_with("true") => Ok((Value::Bool(true), &s[4..])),
            _ if s.starts_with("false") => Ok((Value::Bool(false), &s[5..])),
            _ if s.starts_with('\'') => {
                if s.starts_with("'\\") {
                    let (ch, s) = escape_sequence(&s[1..]).ok_or(ValueError::InvalidCharacter)?;
                    let s = s.strip_prefix('\'').ok_or(ValueError::InvalidCharacter)?;
                    Ok((Value::Char(ch), s))
                } else if &s[2..3] == "'" {
                    Ok((
                        Value::Char(s[1..2].parse().map_err(|_| ValueError::InvalidCharacter)?),
                        &s[3..],
                    ))
                } else {
                    let s = &s[1..];
                    let atom: String = s
                        .chars()
                        .take_while(|&ch| ch.is_ascii_alphanumeric() || ch == '_')
                        .collect();
                    if atom.is_empty() {
                        Err(ValueError::InvalidAtom)
                    } else {
                        let s = &s[atom.len()..];
                        let atom = context.intern(&atom);
                        if let Some(s) = s.strip_prefix('(') {
                            let (value, s) = Value::parse_prefix(s, context)?;
                            let s = s.strip_prefix(')').ok_or(ValueError::InvalidStruct)?;
                            Ok((Value::Struct(Struct::new(atom, value)), s))
                        } else {
                            Ok((Value::Atom(atom), s))
                        }
                    }
                }
            }
            _ if s.starts_with('(') => {
                let s = &s[1..];
                let (lhs, s) = Value::parse_prefix(s, context)?;
                let s = s.strip_prefix(':').ok_or(ValueError::InvalidTuple)?;
                let (rhs, s) = Value::parse_prefix(s, context)?;
                let s = s.strip_prefix(')').ok_or(ValueError::InvalidTuple)?;
                Ok((Value::Tuple(Tuple::new(lhs, rhs)), s))
            }
            _ if s.starts_with('"') => extract_string_prefix(s)
                .map(|(v, s)| (Value::String(v), s))
                .ok_or(ValueError::InvalidString),
            _ if s.starts_with("[|") => {
                let mut set = HashSet::new();
                let mut s = &s[2..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix("|]") {
                        break rest;
                    }
                    if s.is_empty() {
                        return Err(ValueError::InvalidSet);
                    }
                    let (value, rest) = Value::parse_prefix(s, context)?;
                    set.insert(value);
                    if let Some(rest) = rest.strip_prefix("|]") {
                        break rest;
                    }
                    s = rest.strip_prefix(',').ok_or(ValueError::InvalidSet)?;
                };
                Ok((Value::Set(Set::from(set)), s))
            }
            _ if s.starts_with("{|") => {
                let mut map = HashMap::new();
                let mut s = &s[2..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix("|}") {
                        break rest;
                    }
                    if s.is_empty() {
                        return Err(ValueError::InvalidRecord);
                    }
                    let (key, rest) = Value::parse_prefix(s, context)?;
                    let rest = rest.strip_prefix("=>").ok_or(ValueError::InvalidRecord)?;
                    let (value, rest) = Value::parse_prefix(rest, context)?;
                    map.insert(key, value);
                    if let Some(rest) = rest.strip_prefix("|}") {
                        break rest;
                    }
                    s = rest.strip_prefix(',').ok_or(ValueError::InvalidRecord)?;
                };
                Ok((Value::Record(Record::from(map)), s))
            }
            _ if s.starts_with('[') => {
                let mut array = vec![];
                let mut s = &s[1..];
                let s = loop {
                    if let Some(rest) = s.strip_prefix(']') {
                        break rest;
                    }
                    if s.is_empty() {
                        return Err(ValueError::InvalidArray);
                    }
                    let (value, rest) = Value::parse_prefix(s, context)?;
                    array.push(value);
                    if let Some(rest) = rest.strip_prefix(']') {
                        break rest;
                    }
                    s = rest.strip_prefix(',').ok_or(ValueError::InvalidArray)?;
                };
                Ok((Value::Array(Array::from(array)), s))
            }
            _ if s.starts_with('&') => {
                let s = &s[1..];
                if let Some(s) = s.strip_prefix('(') {
                    let numberlike: String = s.chars().take_while(|ch| ch.is_numeric()).collect();
                    let offset = numberlike
                        .parse()
                        .map_err(|_| ValueError::InvalidProcedure)?;
                    let s = s[numberlike.len()..]
                        .strip_prefix(')')
                        .ok_or(ValueError::InvalidProcedure)?;
                    Ok((Value::Procedure(Procedure::new(offset)), s))
                } else {
                    let Some((label, s)) = AsmContext::take_label(s) else {
                        return Err(ValueError::InvalidProcedure);
                    };
                    let offset = context
                        .lookup_label(&label)
                        .ok_or(ValueError::UnresolvedLabelReference)?;
                    Ok((Value::Procedure(Procedure::new(offset)), s))
                }
            }
            _ if s.starts_with("0b") => {
                let bits: Bits = s[2..]
                    .chars()
                    .take_while(|&c| c == '0' || c == '1')
                    .map(|ch| ch == '1')
                    .collect();
                let s = &s[bits.len() + 2..];
                Ok((Value::Bits(bits), s))
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
                Ok((
                    Value::Number(numberlike.parse().map_err(|_| ValueError::InvalidNumber)?),
                    &s[numberlike.len()..],
                ))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Instruction, StructuralEq};
    use num::{BigRational, Complex};

    macro_rules! test {
        ($name:ident =>  $input:literal, $value:expr, $tail:literal) => {
            #[test]
            fn $name() {
                let (value, tail) =
                    Value::parse_prefix($input, &mut AsmContext::default()).unwrap();
                assert!(StructuralEq::eq(&value, &$value.into()));
                assert_eq!(tail, $tail);
            }
        };
    }

    test!(parse_string => r#""hello""#, "hello", "");
    test!(parse_string_escapes => r#""hel\\\x15\u{ff00}lo""#, "hel\\\x15\u{ff00}lo", "");
    test!(parse_string_trailing => r#""hello"123"#, "hello", "123");
    test!(parse_char => "'a'", 'a', "");
    test!(parse_char_escape => r#"'\\'"#, '\\', "");
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
    test!(parse_procedure => "&(123)", Procedure::new(123), "");
    test!(parse_bits => "0b111011", vec![true, true, true, false, true, true] , "");

    #[test]
    fn parse_atom() {
        let mut context = AsmContext::default();
        let atom = context.intern(&String::from("hello"));
        let (value, tail) = Value::parse_prefix("'hello", &mut context).unwrap();
        assert!(StructuralEq::eq(&value, &atom.into()));
        assert_eq!(tail, "");
    }

    #[test]
    fn parse_struct() {
        let mut context = AsmContext::default();
        let atom = context.intern(&String::from("hello"));
        let (value, tail) = Value::parse_prefix("'hello(123)", &mut context).unwrap();
        assert!(StructuralEq::eq(
            &value,
            &Struct::new(atom, 123.into()).into()
        ));
        assert_eq!(tail, "");
    }

    #[test]
    fn parse_procedure_label() {
        let mut context = AsmContext::default();
        context.parse_line::<Instruction>("label:").unwrap();
        let (value, tail) = Value::parse_prefix("&label", &mut context).unwrap();
        assert!(StructuralEq::eq(&value, &Procedure::new(0).into()));
        assert_eq!(tail, "");
    }
}
