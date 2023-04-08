use trilogy_parser::syntax;

use super::*;
use crate::{Analyzer, Id};

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

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
enum ItemType {
    Proc,
    Func,
    Rule,
}

impl FromIterator<Definition> for Definitions {
    fn from_iter<T: IntoIterator<Item = Definition>>(iter: T) -> Self {
        use std::collections::hash_map::Entry;
        use std::collections::HashMap;

        let mut simple = Vec::new();
        let mut definitions = HashMap::new();
        for mut definition in iter {
            match &mut definition.item {
                DefinitionItem::Procedure(proc) => {
                    let key = (
                        ItemType::Proc,
                        proc.name.id.clone(),
                        proc.overloads.first().unwrap().parameters.len(),
                    );
                    match definitions.entry(key) {
                        Entry::Vacant(slot) => {
                            slot.insert(definition);
                        }
                        Entry::Occupied(mut slot) => {
                            let def = slot.get_mut();
                            let DefinitionItem::Procedure(def) = &mut def.item else { unreachable!() };
                            def.overloads.extend(proc.overloads.drain(..))
                        }
                    }
                }
                DefinitionItem::Function(func) => {
                    let key = (
                        ItemType::Func,
                        func.name.id.clone(),
                        func.overloads.first().unwrap().parameters.len(),
                    );
                    match definitions.entry(key) {
                        Entry::Vacant(slot) => {
                            slot.insert(definition);
                        }
                        Entry::Occupied(mut slot) => {
                            let def = slot.get_mut();
                            let DefinitionItem::Function(def) = &mut def.item else { unreachable!() };
                            def.overloads.extend(func.overloads.drain(..))
                        }
                    }
                }
                DefinitionItem::Rule(rule) => {
                    let key = (
                        ItemType::Rule,
                        rule.name.id.clone(),
                        rule.overloads.first().unwrap().parameters.len(),
                    );
                    match definitions.entry(key) {
                        Entry::Vacant(slot) => {
                            slot.insert(definition);
                        }
                        Entry::Occupied(mut slot) => {
                            let def = slot.get_mut();
                            let DefinitionItem::Rule(def) = &mut def.item else { unreachable!() };
                            def.overloads.extend(rule.overloads.drain(..))
                        }
                    }
                }
                _ => simple.push(definition),
            }
        }
        simple.extend(definitions.into_values());
        Self(simple)
    }
}
