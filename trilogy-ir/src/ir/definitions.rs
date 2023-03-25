use super::*;
use crate::Id;

#[derive(Clone, Debug)]
pub struct Definitions(Vec<Definition>);

impl Definitions {
    pub(super) fn get_mut(&mut self, id: &Id) -> Option<&mut Definition> {
        self.0.iter_mut().find(|def| match &def.item {
            DefinitionItem::Function(def) => def.name.id == *id,
            DefinitionItem::Procedure(def) => def.name.id == *id,
            DefinitionItem::Rule(def) => def.name.id == *id,
            DefinitionItem::Module(def) => def.name.id == *id,
            _ => false,
        })
    }

    pub(super) fn push(&mut self, definition: Definition) {
        self.0.push(definition);
    }
}

impl FromIterator<Definition> for Definitions {
    fn from_iter<T: IntoIterator<Item = Definition>>(_iter: T) -> Self {
        todo!("collect but merging duplicates")
    }
}
