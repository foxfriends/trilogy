use super::{Identifier, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Template {
    start: Token,
    pub segments: Vec<TemplateSegment>,
    pub tag: Option<Identifier>,
}

impl Template {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if let Ok(start) = parser.expect(TokenType::DollarString) {
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
            .expect(TokenType::TemplateStart)
            .expect("Caller should have found this");
        let mut segments = vec![];
        loop {
            let interpolation = Expression::parse(parser)?;
            if let Ok(end) = parser.expect(TokenType::TemplateContinue) {
                segments.push(TemplateSegment { interpolation, end });
                continue;
            }
            let end = parser
                .expect(TokenType::TemplateEnd)
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

    pub fn prefix(&self) -> String {
        let TokenValue::String(value) = self.start.value.as_ref().unwrap() else { unreachable!() };
        value.to_owned()
    }

    pub fn prefix_token(&self) -> &Token {
        &self.start
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

impl TemplateSegment {
    pub fn suffix(&self) -> String {
        let TokenValue::String(value) = self.end.value.as_ref().unwrap() else { unreachable!() };
        value.to_owned()
    }

    pub fn suffix_token(&self) -> &Token {
        &self.end
    }
}
