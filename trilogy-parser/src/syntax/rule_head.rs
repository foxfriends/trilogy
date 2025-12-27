use super::*;
use crate::{Parser, Spanned};
use source_span::Span;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RuleHead {
    pub name: Identifier,
    pub parameter_list: ParameterList,
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
            name,
            parameter_list,
        })
    }
}
