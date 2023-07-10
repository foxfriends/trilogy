use trilogy_ir::ir::Identifier;

#[derive(Clone, Debug)]
pub(crate) struct Labeler {
    location: String,
    context: Vec<String>,
}

impl Labeler {
    pub fn new(location: String) -> Self {
        Self {
            location,
            context: vec![],
        }
    }

    pub fn label(&self, suffix: &str) -> String {
        format!("{}#{}${suffix}", self.location, self.context.join("::"))
    }

    pub fn begin_procedure(&mut self, identifier: &Identifier) -> String {
        self.context.push(identifier.to_string());
        self.context.join("::")
    }

    pub fn begin_overload(&mut self, index: usize) -> String {
        self.context.push(index.to_string());
        self.context.join("::")
    }

    pub fn end(&mut self) -> String {
        let label = self.to_end();
        self.context.pop();
        label
    }

    pub fn to_end(&self) -> String {
        self.label("end")
    }
}
