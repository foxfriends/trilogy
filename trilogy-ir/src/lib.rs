#![allow(dead_code)] // this is all just planning anyway

use source_span::Span;
use std::collections::{HashMap, HashSet};
use trilogy_loader::Location;

pub struct Program {
    modules: HashMap<ModuleKey, Module>,
    native_modules: HashMap<NativeModuleKey, NativeModule>,
    main_module: ModuleKey,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ModuleKey {
    location: Location,
    path: Vec<String>,
    arity: usize,
}

pub struct Module {
    span: Span,
    items: HashMap<ItemKey, Item>,
    tests: Vec<Test>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct NativeModuleKey {
    path: Vec<String>,
    arity: usize,
}

pub struct NativeModule {
    items: HashSet<ItemKey>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ItemKey {
    name: String,
    arity: usize,
}

pub enum Item {
    Function(Box<Function>),
    Procedure(Box<Procedure>),
    Rule(Box<Rule>),
    Submodule(Box<ModuleKey>),
    Rename(Box<Rename>),
}

pub struct Function {
    span: Vec<Span>,
    is_exported: bool,
    code: Code,
}

pub struct Procedure {
    span: Vec<Span>,
    is_exported: bool,
    code: Code,
}

pub struct Rule {
    span: Vec<Span>,
    is_exported: bool,
    code: Code,
}

pub struct Test {
    span: Span,
    code: Code,
}

pub struct Rename {
    span: Span,
    item: Pattern,
    is_exported: bool,
}

pub enum Code {
    Unif(Unification),
    Eval(Evaluation),
    Flow(ControlFlow),
}

pub struct Unification {
    span: Span,
    pattern: Pattern,
}

pub struct Evaluation {
    span: Span,
    expression: Expression,
}

pub struct ConstantEvaluation {
    span: Span,
    expression: Expression,
}

pub enum ControlFlow {
    Block(Vec<Code>),
    // `for` and `while`
    Loop(Vec<Branch>),
    // `if` and `match
    Branch(Vec<Branch>),
}

pub struct Branch {
    condition: Vec<Code>,
    body: Vec<Code>,
}

// TODO: IR version of these big boys.
pub enum Expression {}
pub enum Pattern {}
