mod context;
mod evaluation;
mod function;
mod helpers;
mod module;
mod operator;
mod pattern_match;
mod preamble;
mod procedure;
mod program;
mod query;

pub use program::write_program;

mod prelude {
    pub(crate) use crate::context::{Binding, Context};
    pub(crate) use crate::evaluation::{write_evaluation, write_expression};
    pub(crate) use crate::function::write_function;
    pub(crate) use crate::helpers::*;
    pub(crate) use crate::module::write_module;
    pub(crate) use crate::operator::*;
    pub(crate) use crate::pattern_match::write_pattern_match;
    pub(crate) use crate::preamble::write_preamble;
    pub(crate) use crate::procedure::write_procedure;
    pub(crate) use crate::query::*;
}
