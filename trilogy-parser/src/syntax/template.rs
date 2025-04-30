use super::{Identifier, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Template {
    pub template_start: Token,
    pub segments: Vec<TemplateSegment>,
    pub tag: Option<Identifier>,
    span: Span,
}

impl Template {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if let Ok(template_start) = parser.expect(TokenType::DollarString) {
            let mut span = template_start.span;

            let tag = parser
                .check(TokenType::Identifier)
                .ok()
                .map(|_| ())
                .map(|_| Identifier::parse(parser))
                .transpose()?;
            if let Some(tag) = &tag {
                span = span.union(tag.span());
            }

            return Ok(Self {
                span,
                template_start,
                segments: vec![],
                tag,
            });
        }

        let template_start = parser
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
        parser.chomp();
        let tag = if !parser.is_line_start {
            parser
                .check(TokenType::Identifier)
                .ok()
                .map(|_| ())
                .map(|_| Identifier::parse(parser))
                .transpose()?
        } else {
            None
        };

        let mut span = template_start.span;
        if !segments.is_empty() {
            span = span.union(segments.span());
        }
        if let Some(tag) = &tag {
            span = span.union(tag.span());
        }

        Ok(Self {
            span,
            template_start,
            segments,
            tag,
        })
    }

    pub fn prefix(&self) -> String {
        let TokenValue::String(value) = self.template_start.value.as_ref().unwrap() else {
            unreachable!()
        };
        value.to_owned()
    }
}

impl Spanned for Template {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TemplateSegment {
    pub interpolation: Expression,
    end: Token,
}

impl TemplateSegment {
    pub fn suffix(&self) -> String {
        let TokenValue::String(value) = self.end.value.as_ref().unwrap() else {
            unreachable!()
        };
        value.to_owned()
    }

    pub fn suffix_token(&self) -> &Token {
        &self.end
    }
}
