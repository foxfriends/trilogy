use super::{pattern::Precedence, *};
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct GluePattern {
    pub lhs: Pattern,
    glue_token: Token,
    pub rhs: Pattern,
}

impl GluePattern {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        let glue_token = parser
            .expect(OpGlue)
            .expect("Caller should have found this");
        let rhs = Pattern::parse_precedence(parser, Precedence::Glue)?;
        Ok(Self {
            lhs,
            glue_token,
            rhs,
        })
    }

    pub fn glue_token(&self) -> &Token {
        &self.glue_token
    }
}

impl TryFrom<BinaryOperation> for GluePattern {
    type Error = SyntaxError;

    fn try_from(value: BinaryOperation) -> Result<Self, Self::Error> {
        match value.operator {
            BinaryOperator::Glue(token) => Ok(Self {
                lhs: value.lhs.try_into()?,
                glue_token: token,
                rhs: value.rhs.try_into()?,
            }),
            _ => Err(SyntaxError::new(
                value.span(),
                "incorrect operator for glue pattern",
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(glue_pattern_left_str: r#""hello" <> x"# => Pattern::parse => "(Pattern::Glue (GluePattern _ _))");
    test_parse!(glue_pattern_right_str: r#"x <> "hello""# => Pattern::parse => "(Pattern::Glue (GluePattern _ _))");
    test_parse!(glue_pattern_no_str: r#"x <> y"# => Pattern::parse => "(Pattern::Glue (GluePattern _ _))");
    test_parse!(glue_pattern_both_str: r#""x" <> "y""# => Pattern::parse => "(Pattern::Glue (GluePattern _ _))");
    test_parse!(glue_pattern_not_str: r#"1 <> x"# => Pattern::parse => "(Pattern::Glue (GluePattern _ _))");
    test_parse_error!(glue_pattern_incomplete: r#"x <>"# => Pattern::parse);
    test_parse_error!(glue_pattern_invalid_expr: r#"x <> {}"# => Pattern::parse);
}
