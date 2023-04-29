use trilogy_parser::syntax;

use super::*;
use crate::{Analyzer, Id};

#[derive(Clone, Debug)]
pub struct Definitions(pub(crate) Vec<Definition>);

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

    pub(super) fn convert(analyzer: &mut Analyzer, ast: Vec<syntax::Definition>) -> Self {
        let mut definitions = ast
            .iter()
            .flat_map(|ast| Definition::declare(analyzer, ast))
            .collect::<Self>();
        for definition in ast {
            Definition::convert_into(analyzer, definition, &mut definitions);
        }
        definitions
    }
}

impl FromIterator<Definition> for Definitions {
    fn from_iter<T: IntoIterator<Item = Definition>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
