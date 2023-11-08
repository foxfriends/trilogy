use super::*;
use crate::Id;

#[derive(Clone, Debug)]
pub enum DefinitionItem {
    Procedure(Box<ProcedureDefinition>),
    Function(Box<FunctionDefinition>),
    Rule(Box<RuleDefinition>),
    Test(Box<TestDefinition>),
    Constant(Box<ConstantDefinition>),
    Module(Box<ModuleDefinition>),
}

impl From<ProcedureDefinition> for DefinitionItem {
    fn from(value: ProcedureDefinition) -> Self {
        Self::Procedure(Box::new(value))
    }
}

impl From<FunctionDefinition> for DefinitionItem {
    fn from(value: FunctionDefinition) -> Self {
        Self::Function(Box::new(value))
    }
}

impl From<RuleDefinition> for DefinitionItem {
    fn from(value: RuleDefinition) -> Self {
        Self::Rule(Box::new(value))
    }
}

impl From<TestDefinition> for DefinitionItem {
    fn from(value: TestDefinition) -> Self {
        Self::Test(Box::new(value))
    }
}

impl From<ConstantDefinition> for DefinitionItem {
    fn from(value: ConstantDefinition) -> Self {
        Self::Constant(Box::new(value))
    }
}

impl From<ModuleDefinition> for DefinitionItem {
    fn from(value: ModuleDefinition) -> Self {
        Self::Module(Box::new(value))
    }
}

impl DefinitionItem {
    /// Returns the item if it is a procedure, or None otherwise.
    pub fn as_procedure(&self) -> Option<&ProcedureDefinition> {
        match self {
            Self::Procedure(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a procedure, or None otherwise.
    pub fn as_function(&self) -> Option<&FunctionDefinition> {
        match self {
            Self::Function(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a rule, or None otherwise
    pub fn as_rule(&self) -> Option<&RuleDefinition> {
        match self {
            Self::Rule(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a test, or None otherwise
    pub fn as_test(&self) -> Option<&TestDefinition> {
        match self {
            Self::Test(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a constant, or None otherwise
    pub fn as_constant(&self) -> Option<&ConstantDefinition> {
        match self {
            Self::Constant(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    /// Returns the item if it is a module, or None otherwise
    pub fn as_module(&self) -> Option<&ModuleDefinition> {
        match self {
            Self::Module(def) => Some(def.as_ref()),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<&Id> {
        match self {
            DefinitionItem::Procedure(def) => Some(&def.name.id),
            DefinitionItem::Function(def) => Some(&def.name.id),
            DefinitionItem::Rule(def) => Some(&def.name.id),
            DefinitionItem::Constant(def) => Some(&def.name.id),
            DefinitionItem::Test(..) => None,
            DefinitionItem::Module(def) => Some(&def.name.id),
        }
    }
}
