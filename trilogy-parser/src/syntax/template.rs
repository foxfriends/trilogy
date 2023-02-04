use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct Template {
    start: Token,
    pub segments: Vec<TemplateSegment>,
    pub tag: Option<Identifier>,
}

#[derive(Clone, Debug)]
pub struct TemplateSegment {
    pub interpolation: Expression,
    end: Token,
}
