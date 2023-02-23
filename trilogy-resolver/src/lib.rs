use trilogy_parser::syntax::Document;

pub struct Resolver {
    #[allow(dead_code)]
    document: Document,
}

impl Resolver {
    pub fn new(document: Document) -> Self {
        Self { document }
    }
}
