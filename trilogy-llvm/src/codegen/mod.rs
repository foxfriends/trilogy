//! The core code generation tool.
use crate::types;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::values::PointerValue;
use inkwell::{OptimizationLevel, values::FunctionValue};
use source_span::Span;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use trilogy_ir::{Id, ir};

mod cleanup;
mod continuation_point;
mod debug_info;
mod definitions;
mod snapshot;
mod variables;

pub(crate) use continuation_point::{Brancher, Merger};
use continuation_point::{ContinuationPoint, Exit, Parent};
use debug_info::DebugInfo;
pub(crate) use snapshot::Snapshot;
use variables::Closed;
pub(crate) use variables::{Head, Variable};

#[must_use = "confirm that the current basic block will end without further instructions"]
pub(crate) struct NeverValue;

pub(crate) struct Codegen<'ctx> {
    pub(crate) atoms: Rc<RefCell<HashMap<String, u64>>>,
    pub(crate) context: &'ctx Context,
    pub(crate) module: Rc<Module<'ctx>>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) di: DebugInfo<'ctx>,
    pub(crate) execution_engine: ExecutionEngine<'ctx>,
    pub(crate) modules: &'ctx HashMap<String, &'ctx ir::Module>,
    pub(crate) globals: HashMap<Id, Head>,
    pub(crate) location: String,

    /// The chain of continuations arriving at the current expression being compiled.
    ///
    /// The last one is the current point. There should always be at least one (while
    /// compiling a function), and they should be topologically sorted according to their
    /// contained `capture_from` lists.
    continuation_points: RefCell<Vec<Rc<ContinuationPoint<'ctx>>>>,
    current_definition: RefCell<(String, Span)>,
    closure_array: Cell<Option<PointerValue<'ctx>>>,
    pub(crate) function_params: RefCell<Vec<PointerValue<'ctx>>>,
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(
        context: &'ctx Context,
        modules: &'ctx HashMap<String, &'ctx ir::Module>,
    ) -> Self {
        let mut atoms = HashMap::new();
        atoms.insert("undefined".to_owned(), types::TAG_UNDEFINED);
        atoms.insert("unit".to_owned(), types::TAG_UNIT);
        atoms.insert("boolean".to_owned(), types::TAG_BOOL);
        atoms.insert("atom".to_owned(), types::TAG_ATOM);
        atoms.insert("char".to_owned(), types::TAG_CHAR);
        atoms.insert("string".to_owned(), types::TAG_STRING);
        atoms.insert("number".to_owned(), types::TAG_NUMBER);
        atoms.insert("bits".to_owned(), types::TAG_BITS);
        atoms.insert("struct".to_owned(), types::TAG_STRUCT);
        atoms.insert("tuple".to_owned(), types::TAG_TUPLE);
        atoms.insert("array".to_owned(), types::TAG_ARRAY);
        atoms.insert("set".to_owned(), types::TAG_SET);
        atoms.insert("record".to_owned(), types::TAG_RECORD);
        atoms.insert("callable".to_owned(), types::TAG_CALLABLE);
        atoms.insert("left".to_owned(), 14);
        atoms.insert("right".to_owned(), 15);
        atoms.insert("lt".to_owned(), 16);
        atoms.insert("eq".to_owned(), 17);
        atoms.insert("gt".to_owned(), 18);

        let module = context.create_module("trilogy:runtime");
        let ee = module
            .create_jit_execution_engine(OptimizationLevel::Default)
            .unwrap();
        let di = DebugInfo::new(&module, "trilogy:runtime", &ee);

        let codegen = Codegen {
            atoms: Rc::new(RefCell::new(atoms)),
            builder: context.create_builder(),
            di,
            context,
            execution_engine: ee,
            module: Rc::new(module),
            modules,
            globals: HashMap::default(),
            location: "trilogy:runtime".to_owned(),
            continuation_points: RefCell::default(),
            current_definition: RefCell::default(),
            closure_array: Cell::default(),
            function_params: RefCell::default(),
        };

        codegen
    }

    /// Creates a `Codegen` for another (distinct) Trilogy module.
    ///
    /// This function is called `sub` as in `submodule` incorrectly; it is not for creating
    /// a `Codegen` for a Trilogy module's submodule.
    pub(crate) fn for_submodule(&self, name: &str) -> Codegen<'ctx> {
        let module = self.context.create_module(name);
        let di = DebugInfo::new(&module, name, &self.execution_engine);
        Codegen {
            atoms: self.atoms.clone(),
            context: self.context,
            builder: self.context.create_builder(),
            di,
            execution_engine: self.execution_engine.clone(),
            module: Rc::new(module),
            modules: self.modules,
            globals: HashMap::new(),
            location: name.to_owned(),
            continuation_points: RefCell::default(),
            current_definition: RefCell::default(),
            closure_array: Cell::default(),
            function_params: RefCell::default(),
        }
    }

    /// Sets the current definition. This will push a fresh continuation point as the current
    /// implicit continuation point (the first continuation point in the case of a top level
    /// func/proc, or one down the line for a nested fn/do).
    ///
    /// If this is a nested scope, the previous continuation point should be branched from and
    /// later resumed, otherwise we will be lost at the end of the nested code.
    pub(crate) fn set_current_definition(&self, name: String, span: Span) {
        *self.current_definition.borrow_mut() = (name, span);
        self.continuation_points
            .borrow_mut()
            .push(Rc::new(ContinuationPoint::default()));
    }

    pub(crate) fn get_current_definition(&self) -> (String, Span) {
        self.current_definition.borrow().clone()
    }

    pub(crate) fn allocate_value(&self, name: &str) -> PointerValue<'ctx> {
        let value = self.builder.build_alloca(self.value_type(), name).unwrap();
        self.builder
            .build_store(value, self.value_type().const_zero())
            .unwrap();
        value
    }

    pub(crate) fn begin_function(&self, function: FunctionValue<'ctx>, span: Span) {
        self.di.push_subprogram(function.get_subprogram().unwrap());
        self.di.push_block_scope(span);
        self.set_span(span);
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.closure_array.set(None);
        *self.function_params.borrow_mut() = function
            .get_param_iter()
            .map(|param| {
                let container = self
                    .builder
                    .build_alloca(
                        self.value_type(),
                        &format!("{}.value", param.get_name().to_string_lossy()),
                    )
                    .unwrap();
                self.builder.build_store(container, param).unwrap();
                container
            })
            .collect();
    }

    pub(crate) fn begin_next_function(&self, function: FunctionValue<'ctx>) {
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.transfer_debug_info();
        self.closure_array.set(None);
        *self.function_params.borrow_mut() = function
            .get_param_iter()
            .map(|param| {
                let container = self
                    .builder
                    .build_alloca(
                        self.value_type(),
                        &format!("{}.value", param.get_name().to_string_lossy()),
                    )
                    .unwrap();
                self.builder.build_store(container, param).unwrap();
                container
            })
            .collect();
    }

    pub(crate) fn end_function(&self) {
        self.di.pop_scope();
        self.di.pop_scope();
    }
}
