use super::Ghost;
use crate::vm::execution::Cont;
use crate::{InternalRuntimeError, Value};
use std::fmt::{self, Debug, Display};

#[derive(Clone, Debug)]
pub(crate) struct Return {
    pub cont: Cont,
    pub frame: usize,
    pub ghost_frame: usize,
    pub ghost: Option<Ghost>,
}

#[derive(Clone, Debug)]
pub(crate) enum InternalValue {
    Unset,
    Value(Value),
    Return(Return),
}

impl InternalValue {
    pub fn try_into_value(self) -> Result<Value, InternalRuntimeError> {
        match self {
            InternalValue::Value(value) => Ok(value),
            InternalValue::Unset => Err(InternalRuntimeError::ExpectedValue("empty cell")),
            InternalValue::Return(..) => Err(InternalRuntimeError::ExpectedValue("return pointer")),
        }
    }

    pub(super) fn try_into_value_maybe(self) -> Result<Option<Value>, InternalRuntimeError> {
        match self {
            InternalValue::Value(value) => Ok(Some(value)),
            InternalValue::Unset => Ok(None),
            InternalValue::Return(..) => Err(InternalRuntimeError::ExpectedValue("return pointer")),
        }
    }

    pub(super) fn is_set(&self) -> Result<bool, InternalRuntimeError> {
        match self {
            InternalValue::Value(..) => Ok(true),
            InternalValue::Unset => Ok(false),
            InternalValue::Return(..) => Err(InternalRuntimeError::ExpectedValue("return pointer")),
        }
    }

    pub(super) fn as_return(&self) -> Option<&Return> {
        match self {
            InternalValue::Return(ret) => Some(ret),
            _ => None,
        }
    }

    pub(super) fn into_return(self) -> Option<Return> {
        match self {
            InternalValue::Return(ret) => Some(ret),
            _ => None,
        }
    }
}

impl From<Value> for InternalValue {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl Display for InternalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalValue::Unset => write!(f, "<unset>"),
            InternalValue::Value(value) => write!(f, "{value}"),
            InternalValue::Return(ret) => match &ret.ghost {
                None => write!(f, "-> {:?}", ret.cont),
                Some(ghost) => {
                    let ghost_str = format!("{}", ghost.stack)
                        .lines()
                        .map(|line| format!("\t{line}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    writeln!(f, "{}", ghost_str)?;
                    write!(f, "-> {:?}\t[closure]", ret.cont)
                }
            },
        }
    }
}
