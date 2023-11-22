#[trilogy_derive::module(crate_name=crate)]
pub mod regex {
    use crate::Runtime;
    use std::collections::HashMap;
    use trilogy_vm::Value;

    #[derive(Clone)]
    pub struct Regex(::regex::Regex);

    #[trilogy_derive::module(crate_name=crate)]
    impl Regex {
        #[trilogy_derive::func(crate_name=crate)]
        fn is_match(self, rt: Runtime, value: Value) -> crate::Result<()> {
            match &value {
                Value::String(string) => rt.r#return(self.0.is_match(string)),
                _ => Err(rt.runtime_type_error(value)),
            }
        }

        #[trilogy_derive::func(crate_name=crate)]
        fn matches(self, rt: Runtime, value: Value) -> crate::Result<()> {
            match &value {
                Value::String(string) => {
                    let Some(captures) = self.0.captures(string) else {
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
                _ => Err(rt.runtime_type_error(value)),
            }
        }
    }

    #[trilogy_derive::func(crate_name=crate)]
    pub fn regex(rt: Runtime, value: Value) -> crate::Result<()> {
        match &value {
            Value::String(string) => match ::regex::Regex::new(string) {
                Ok(regex) => rt.r#return(Regex(regex)),
                Err(..) => Err(rt.runtime_type_error(value)),
            },
            _ => Err(rt.runtime_type_error(value)),
        }
    }
}
