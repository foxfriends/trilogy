use trilogy_ir::Id;

#[derive(Clone, Debug)]
pub(crate) struct Labeler {
    counter: usize,
}

impl Labeler {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn unique_hint(&mut self, hint: &str) -> String {
        self.counter += 1;
        format!("#temp::{}::{hint}", self.counter)
    }

    pub fn var(&mut self, var: &Id) -> String {
        self.counter += 1;
        format!("#var::{}::{}::{:#?}", self.counter, var, var.as_ptr())
    }

    pub fn unvar(&mut self, var: &Id) -> String {
        self.counter += 1;
        format!("#unvar::{}::{}::{:#?}", self.counter, var, var.as_ptr())
    }

    pub fn for_id(&self, id: &Id) -> String {
        id.symbol()
    }
}
