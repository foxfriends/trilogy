use trilogy_ir::{ir::Identifier, Id};

#[derive(Clone, Debug)]
pub(crate) struct Labeler {
    location: String,
    context: Vec<String>,
    counter: usize,
}

impl Labeler {
    pub fn new(location: String) -> Self {
        Self {
            location,
            context: vec![],
            counter: 0,
        }
    }

    pub fn unique(&mut self) -> String {
        self.counter += 1;
        format!("#temp::{}", self.counter)
    }

    pub fn label(&self, suffix: &str) -> String {
        format!("{}#{}${suffix}", self.location, self.context.join("::"))
    }

    pub fn for_id(&self, id: &Id) -> String {
        id.symbol()
    }

    pub fn begin_procedure(&mut self, identifier: &Identifier) -> String {
        self.label(&identifier.to_string())
    }
}
