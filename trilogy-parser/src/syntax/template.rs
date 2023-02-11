use super::{Identifier, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{
    Token,
    TokenType::{self, *},
};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Template {
    start: Token,
    pub segments: Vec<TemplateSegment>,
    pub tag: Option<Identifier>,
}

impl Template {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if let Ok(start) = parser.expect(DollarString) {
            let tag = parser
                .check(TokenType::Identifier)
                .ok()
                .map(|_| ())
                .map(|_| Identifier::parse(parser))
                .transpose()?;
            return Ok(Self {
                start,
                segments: vec![],
                tag,
            });
        }

        let start = parser
            .expect(TemplateStart)
            .expect("Caller should have found this");
        let mut segments = vec![];
        loop {
            let interpolation = Expression::parse(parser)?;
            if let Ok(end) = parser.expect(TemplateContinue) {
                segments.push(TemplateSegment { interpolation, end });
                continue;
            }
            let end = parser
                .expect(TemplateEnd)
                .map_err(|token| parser.expected(token, "incomplete template string"))?;
            segments.push(TemplateSegment { interpolation, end });
            break;
        }
        let tag = parser
            .check(TokenType::Identifier)
            .ok()
            .map(|_| ())
            .map(|_| Identifier::parse(parser))
            .transpose()?;
        Ok(Self {
            start,
            segments,
            tag,
        })
    }
}

impl Spanned for Template {
    fn span(&self) -> Span {
        let mut span = self.start.span;
        if !self.segments.is_empty() {
            span = span.union(self.segments.span());
        }
        if let Some(tag) = &self.tag {
            span = span.union(tag.span());
        }
        span
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TemplateSegment {
    pub interpolation: Expression,
    end: Token,
}
