#[trilogy_derive::module(crate_name=crate)]
pub mod regex {
    use crate::Runtime;
    use regex::RegexBuilder;
    use std::collections::HashMap;
    use trilogy_vm::{Array, Tuple, Value};

    #[derive(Clone)]
    pub struct Regex(::regex::Regex);

    #[trilogy_derive::module(crate_name=crate)]
    impl Regex {
        #[trilogy_derive::func(crate_name=crate)]
        fn is_match(self, rt: Runtime, value: Value) -> crate::Result<()> {
            let value = rt.typecheck::<String>(value)?;
            rt.r#return(self.0.is_match(&value))
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn matches(self, rt: Runtime, value: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(value)?;
            let Some(captures) = self.0.captures(&string) else {
                let atom = rt.atom("NoMatch");
                return rt.r#yield(atom, |rt, val| rt.r#return(val));
            };
            let captures = self.0.capture_names().enumerate().fold(
                HashMap::<Value, Value>::new(),
                |mut map, (i, name)| {
                    if let Some(capture) = captures.get(i) {
                        map.insert(i.into(), capture.clone().as_str().into());
                        if let Some(name) = name {
                            map.insert(name.into(), capture.clone().as_str().into());
                        }
                    }
                    map
                },
            );
            rt.r#return(captures)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn all_matches(self, rt: Runtime, value: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(value)?;
            let captures = self
                .0
                .captures_iter(&string)
                .map(|captures| {
                    self.0.capture_names().enumerate().fold(
                        HashMap::<Value, Value>::new(),
                        |mut map, (i, name)| {
                            if let Some(capture) = captures.get(i) {
                                map.insert(i.into(), capture.clone().as_str().into());
                                if let Some(name) = name {
                                    map.insert(
                                        name.to_owned().into(),
                                        capture.clone().as_str().into(),
                                    );
                                }
                            }
                            map
                        },
                    )
                })
                .map(Value::from)
                .collect::<Array>();
            rt.r#return(captures)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn positions(self, rt: Runtime, value: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(value)?;
            let Some(captures) = self.0.captures(&string) else {
                let atom = rt.atom("NoMatch");
                return rt.r#yield(atom, |rt, val| rt.r#return(val));
            };
            let captures = self.0.capture_names().enumerate().fold(
                HashMap::<Value, Value>::new(),
                |mut map, (i, name)| {
                    if let Some(capture) = captures.get(i) {
                        map.insert(
                            i.into(),
                            Tuple::from((capture.start(), capture.end())).into(),
                        );
                        if let Some(name) = name {
                            map.insert(
                                name.into(),
                                Tuple::from((capture.start(), capture.end())).into(),
                            );
                        }
                    }
                    map
                },
            );
            rt.r#return(captures)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn all_positions(self, rt: Runtime, value: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(value)?;
            let captures = self
                .0
                .captures_iter(&string)
                .map(|captures| {
                    self.0.capture_names().enumerate().fold(
                        HashMap::<Value, Value>::new(),
                        |mut map, (i, name)| {
                            if let Some(capture) = captures.get(i) {
                                map.insert(
                                    i.into(),
                                    Tuple::from((capture.start(), capture.end())).into(),
                                );
                                if let Some(name) = name {
                                    map.insert(
                                        name.into(),
                                        Tuple::from((capture.start(), capture.end())).into(),
                                    );
                                }
                            }
                            map
                        },
                    )
                })
                .map(Value::from)
                .collect::<Array>();
            rt.r#return(captures)
        }
    }

    fn construct(rt: Runtime, builder: &RegexBuilder) -> crate::Result<()> {
        match builder.build() {
            Ok(regex) => rt.r#return(Regex(regex)),
            Err(::regex::Error::Syntax(value)) => {
                Err(rt.runtime_error(rt.r#struct("RegexError", value)))
            }
            Err(..) => Err(rt.runtime_error(
                rt.r#struct("RegexError", "failed to safely compile regular expression"),
            )),
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn new(rt: Runtime, flags: Value, pattern: Value) -> crate::Result<()> {
        let pattern = rt.typecheck::<String>(pattern)?;
        let flags = rt.typecheck::<Array>(flags)?;
        let mut builder = ::regex::RegexBuilder::new(&pattern);
        let builder = flags
            .into_iter()
            .try_fold(builder.octal(true), |rx, opt| match opt {
                Value::Atom(atom) if atom.as_ref() == "i" => Ok(rx.case_insensitive(true)),
                Value::Atom(atom) if atom.as_ref() == "m" => Ok(rx.multi_line(true)),
                Value::Atom(atom) if atom.as_ref() == "s" => Ok(rx.dot_matches_new_line(true)),
                Value::Atom(atom) if atom.as_ref() == "x" => Ok(rx.ignore_whitespace(true)),
                Value::Atom(atom) if atom.as_ref() == "u" => Ok(rx.unicode(true)),
                Value::Atom(atom) if atom.as_ref() == "U" => Ok(rx.swap_greed(true)),
                Value::Atom(atom) if atom.as_ref() == "R" => Ok(rx.crlf(true)),
                Value::Struct(opt) if opt.name().as_ref() == "n" => {
                    let ch: char = opt.into_value().try_into()?;
                    let ch = ch as u8;
                    Ok(rx.line_terminator(ch))
                }
                _ => Err(opt),
            })
            .map_err(|er| {
                rt.runtime_error(rt.r#struct(
                    "RegexError",
                    format!("invalid option `{er}` in regex flags"),
                ))
            })?;
        construct(rt, builder)
    }
}
