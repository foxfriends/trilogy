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

pub(crate) use continuation_point::{ContinuationPoint, Merger};
use continuation_point::{Exit, Parent};
use debug_info::DebugInfo;
pub(crate) use snapshot::Snapshot;
use variables::Closed;
pub(crate) use variables::{Global, Head, Variable};

#[must_use = "confirm that the current basic block will end without further instructions"]
pub(crate) struct NeverValue;

pub(crate) const ATOM_ASSERTION_FAILED: u64 = 21;

pub(crate) struct Codegen<'ctx> {
    pub(crate) atoms: Rc<RefCell<HashMap<String, u64>>>,
    pub(crate) context: &'ctx Context,
    pub(crate) module: Rc<Module<'ctx>>,
    pub(crate) builder: Builder<'ctx>,
    pub(crate) di: DebugInfo<'ctx>,
    pub(crate) execution_engine: ExecutionEngine<'ctx>,
    pub(crate) modules: &'ctx HashMap<String, &'ctx ir::Module>,
    pub(crate) globals: HashMap<Id, Global>,
    pub(crate) location: String,
    pub(crate) path: Vec<String>,

    pub(crate) tests: Vec<String>,

    /// The chain of continuations arriving at the current expression being compiled.
    ///
    /// The last one is the current point. There should always be at least one (while
    /// compiling a function), and they should be topologically sorted according to their
    /// contained `capture_from` lists.
    continuation_points: RefCell<Vec<Rc<ContinuationPoint<'ctx>>>>,
    current_definition: RefCell<(String, String, Span)>,
    closure_array: Cell<Option<PointerValue<'ctx>>>,
    pub(crate) function_params: RefCell<Vec<PointerValue<'ctx>>>,
    pub(crate) current_break: RefCell<Vec<PointerValue<'ctx>>>,
    pub(crate) current_continue: RefCell<Vec<PointerValue<'ctx>>>,
    pub(crate) current_cancel: RefCell<Vec<PointerValue<'ctx>>>,
    pub(crate) current_resume: RefCell<Vec<PointerValue<'ctx>>>,
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
        atoms.insert("module".to_owned(), types::TAG_MODULE);
        atoms.insert("left".to_owned(), 15);
        atoms.insert("right".to_owned(), 16);
        atoms.insert("lt".to_owned(), 17);
        atoms.insert("eq".to_owned(), 18);
        atoms.insert("gt".to_owned(), 19);
        atoms.insert("eof".to_owned(), 20);
        atoms.insert("assertion_failed".to_owned(), ATOM_ASSERTION_FAILED);

        let module = context.create_module("trilogy:runtime");
        let ee = module
            .create_jit_execution_engine(OptimizationLevel::Default)
            .unwrap();
        let di = DebugInfo::new(&module, "trilogy:runtime", &ee);

        Codegen {
            path: vec![],
            tests: vec![],
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
            current_break: RefCell::default(),
            current_continue: RefCell::default(),
            current_cancel: RefCell::default(),
            current_resume: RefCell::default(),
        }
    }

    /// Creates a `Codegen` for another Trilogy file (module).
    pub(crate) fn for_file(&self, name: &str) -> Codegen<'ctx> {
        let module = self.context.create_module(name);
        let di = DebugInfo::new(&module, name, &self.execution_engine);
        Codegen {
            path: vec![],
            tests: vec![],
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
            current_break: RefCell::default(),
            current_continue: RefCell::default(),
            current_cancel: RefCell::default(),
            current_resume: RefCell::default(),
        }
    }

    pub(crate) fn module_path(&self) -> String {
        self.path.iter().fold(
            self.module.get_name().to_str().unwrap().to_owned(),
            |p, s| format!("{p}::{s}"),
        )
    }

    /// Sets the current definition. This will push a fresh continuation point as the current
    /// implicit continuation point (the first continuation point in the case of a top level
    /// func/proc, or one down the line for a nested fn/do).
    ///
    /// If this is a nested scope, the previous continuation point should be branched from and
    /// later resumed, otherwise we will be lost at the end of the nested code.
    pub(crate) fn set_current_definition(
        &self,
        name: String,
        linkage_name: String,
        span: Span,
        module_context: Option<Vec<Id>>,
    ) {
        *self.current_definition.borrow_mut() = (name, linkage_name, span);
        self.continuation_points
            .borrow_mut()
            .push(Rc::new(ContinuationPoint::new(
                module_context.unwrap_or_default(),
            )));
    }

    pub(crate) fn get_current_definition(&self) -> (String, String, Span) {
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
        self.current_break.borrow_mut().clear();
        self.current_continue.borrow_mut().clear();
        self.current_cancel.borrow_mut().clear();
        self.current_resume.borrow_mut().clear();
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
                self.bind_temporary(container);
                container
            })
            .collect();
    }

    pub(crate) fn begin_constant(&self, function: FunctionValue<'ctx>, span: Span) {
        self.di.push_subprogram(function.get_subprogram().unwrap());
        self.di.push_block_scope(span);
        self.set_span(span);
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.closure_array.set(None);
        *self.function_params.borrow_mut() = function
            .get_param_iter()
            .map(|param| param.into_pointer_value())
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
                self.bind_temporary(container);
                container
            })
            .collect();
    }

    pub(crate) fn end_function(&self) {
        self.di.pop_scope();
        self.di.pop_scope();
    }

    pub(crate) fn push_loop_scope(
        &self,
        break_to: PointerValue<'ctx>,
        continue_to: PointerValue<'ctx>,
    ) {
        self.current_break.borrow_mut().push(break_to);
        self.current_continue.borrow_mut().push(continue_to);
    }

    pub(crate) fn pop_loop_scope(&self) {
        self.current_break.borrow_mut().pop();
        self.current_continue.borrow_mut().pop();
    }

    pub(crate) fn push_with_scope(&self, cancel_to: PointerValue<'ctx>) {
        self.current_cancel.borrow_mut().push(cancel_to);
    }

    pub(crate) fn pop_with_scope(&self) {
        self.current_cancel.borrow_mut().pop();
    }

    pub(crate) fn push_handler_scope(&self, resume_to: PointerValue<'ctx>) {
        self.current_resume.borrow_mut().push(resume_to);
    }

    pub(crate) fn pop_handler_scope(&self) {
        self.current_resume.borrow_mut().pop();
    }

    pub(crate) fn consume(&mut self, submodule: Self) {
        submodule.di.builder.finalize();
        self.module
            .link_in_module(Rc::into_inner(submodule.module).unwrap())
            .unwrap();
        self.tests.extend(submodule.tests);
    }
}
