#[allow(clippy::module_inception)]
mod context;
mod labeler;
mod program_context;
mod scope;
mod static_member;

use labeler::Labeler;
// TODO: unpub this
pub(crate) use scope::{Binding, Scope};

pub(crate) use context::Context;
pub(crate) use program_context::ProgramContext;
pub(crate) use static_member::StaticMember;
