use super::{Identifier, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Template {
    pub tag: Option<(Token, Identifier)>,
    pub template_start: Token,
    pub segments: Vec<TemplateSegment>,
    span: Span,
}

impl Template {
    fn parse(tag: Option<(Token, Identifier)>, parser: &mut Parser) -> SyntaxResult<Self> {
        let template_start = parser
            .expect(TokenType::TemplateStart)
            .expect("caller should have found this");
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

        let mut span = template_start.span;
        if !segments.is_empty() {
            span = span.union(segments.span());
        }
        if let Some(tag) = &tag {
            span = span.union(tag.0.span());
        }

        Ok(Self {
            span,
            tag,
            template_start,
            segments,
        })
    }

    pub(crate) fn parse_tagged(parser: &mut Parser) -> SyntaxResult<Self> {
        let dollar = parser
            .expect(TokenType::OpDollar)
            .expect("caller should have found this");
        let tag = Identifier::parse(parser)?;

        if let Ok(template_start) = parser.expect(TokenType::String) {
            return Ok(Self {
                span: dollar.span.union(template_start.span),
                tag: Some((dollar, tag)),
                template_start,
                segments: vec![],
            });
        }

        Self::parse(Some((dollar, tag)), parser)
    }

    pub(crate) fn parse_bare(parser: &mut Parser) -> SyntaxResult<Self> {
        Self::parse(None, parser)
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
