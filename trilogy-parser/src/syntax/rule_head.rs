use super::*;
use crate::{Parser, Spanned};
use source_span::Span;

#[derive(Clone, Debug)]
pub struct RuleHead {
    pub name: Identifier,
    pub parameter_list: ParameterList,
    pub span: Span,
}

impl Spanned for RuleHead {
    fn span(&self) -> Span {
        self.name.span().union(self.parameter_list.span())
    }
}

impl RuleHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let name = Identifier::parse(parser)?;
        let parameter_list = ParameterList::parse(parser)?;
        Ok(Self {
            span: name.span.union(parameter_list.span),
            name,
            parameter_list,
        })
    }
}
