#[trilogy_derive::module(crate_name=crate)]
pub mod regex {
    use crate::Runtime;
    use regex::RegexBuilder;
    use std::collections::HashMap;
    use trilogy_vm::{Array, Tuple, Value};

    #[derive(Clone)]
    pub struct Regex(::regex::Regex);

    #[derive(Clone)]
    pub struct Match {
        start: usize,
        end: usize,
        value: String,
    }

    impl From<regex::Match<'_>> for Match {
        fn from(value: regex::Match<'_>) -> Self {
            Self {
                start: value.start(),
                end: value.end(),
                value: value.as_str().to_owned(),
            }
        }
    }

    #[trilogy_derive::module(crate_name=crate)]
    impl Match {
        #[trilogy_derive::proc(crate_name=crate)]
        fn start(self, rt: Runtime) -> crate::Result<()> {
            rt.r#return(self.start)
        }

        #[trilogy_derive::proc(crate_name=crate)]
        fn end_(self, rt: Runtime) -> crate::Result<()> {
            rt.r#return(self.end)
        }

        #[trilogy_derive::proc(crate_name=crate)]
        fn len(self, rt: Runtime) -> crate::Result<()> {
            rt.r#return(self.end - self.start)
        }

        #[trilogy_derive::proc(crate_name=crate)]
        fn value(self, rt: Runtime) -> crate::Result<()> {
            rt.r#return(self.value)
        }
    }

    #[trilogy_derive::module(crate_name=crate)]
    impl Regex {
        #[trilogy_derive::func(crate_name=crate)]
        fn is_match(self, rt: Runtime, haystack: Value) -> crate::Result<()> {
            let haystack = rt.typecheck::<String>(haystack)?;
            rt.r#return(self.0.is_match(&haystack))
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn captures(self, rt: Runtime, haystack: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(haystack)?;
            let Some(caps) = self.0.captures(&string) else {
                let atom = rt.atom("NoMatch");
                return rt.r#yield(atom, |rt, val| rt.r#return(val));
            };
            let caps = self.0.capture_names().enumerate().fold(
                HashMap::<Value, Value>::new(),
                |mut map, (i, name)| {
                    if let Some(capture) = caps.get(i) {
                        map.insert(i.into(), Match::from(capture).into());
                        if let Some(name) = name {
                            map.insert(name.into(), Match::from(capture).into());
                        }
                    }
                    map
                },
            );
            rt.r#return(caps)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn all_captures(self, rt: Runtime, haystack: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(haystack)?;
            let caps = self
                .0
                .captures_iter(&string)
                .map(|caps| {
                    self.0.capture_names().enumerate().fold(
                        HashMap::<Value, Value>::new(),
                        |mut map, (i, name)| {
                            if let Some(capture) = caps.get(i) {
                                map.insert(i.into(), Match::from(capture).into());
                                if let Some(name) = name {
                                    map.insert(name.into(), Match::from(capture).into());
                                }
                            }
                            map
                        },
                    )
                })
                .map(Value::from)
                .collect::<Array>();
            rt.r#return(caps)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn matches(self, rt: Runtime, haystack: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(haystack)?;
            let Some(caps) = self.0.captures(&string) else {
                let atom = rt.atom("NoMatch");
                return rt.r#yield(atom, |rt, val| rt.r#return(val));
            };
            let caps = self.0.capture_names().enumerate().fold(
                HashMap::<Value, Value>::new(),
                |mut map, (i, name)| {
                    if let Some(capture) = caps.get(i) {
                        map.insert(i.into(), capture.clone().as_str().into());
                        if let Some(name) = name {
                            map.insert(name.into(), capture.clone().as_str().into());
                        }
                    }
                    map
                },
            );
            rt.r#return(caps)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn all_matches(self, rt: Runtime, haystack: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(haystack)?;
            let caps = self
                .0
                .captures_iter(&string)
                .map(|caps| {
                    self.0.capture_names().enumerate().fold(
                        HashMap::<Value, Value>::new(),
                        |mut map, (i, name)| {
                            if let Some(capture) = caps.get(i) {
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
            rt.r#return(caps)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn positions(self, rt: Runtime, haystack: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(haystack)?;
            let Some(caps) = self.0.captures(&string) else {
                let atom = rt.atom("NoMatch");
                return rt.r#yield(atom, |rt, val| rt.r#return(val));
            };
            let caps = self.0.capture_names().enumerate().fold(
                HashMap::<Value, Value>::new(),
                |mut map, (i, name)| {
                    if let Some(capture) = caps.get(i) {
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
            rt.r#return(caps)
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn all_positions(self, rt: Runtime, haystack: Value) -> crate::Result<()> {
            let string = rt.typecheck::<String>(haystack)?;
            let caps = self
                .0
                .captures_iter(&string)
                .map(|caps| {
                    self.0.capture_names().enumerate().fold(
                        HashMap::<Value, Value>::new(),
                        |mut map, (i, name)| {
                            if let Some(capture) = caps.get(i) {
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
            rt.r#return(caps)
        }
    }

    fn construct(rt: Runtime, builder: &RegexBuilder) -> crate::Result<()> {
        match builder.build() {
            Ok(regex) => rt.r#return(Regex(regex)),
            Err(::regex::Error::Syntax(haystack)) => {
                Err(rt.runtime_error(rt.r#struct("RegexError", haystack)))
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
