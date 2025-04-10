use crate::codegen::{Brancher, Codegen};
use crate::types::{CALLABLE_CONTINUATION, CALLABLE_CONTINUE, CALLABLE_RESUME};
use inkwell::llvm_sys::LLVMCallConv;
use inkwell::module::Linkage;
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, FunctionValue, InstructionValue, IntValue,
    LLVMTailCallKind, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate};

impl<'ctx> Codegen<'ctx> {
    /// Checks whether a value that is supposedly a closure is actually a closure, or if it is
    /// just the value `NO_CLOSURE = 0`.
    fn is_closure(&self, closure: PointerValue<'ctx>) -> IntValue<'ctx> {
        let has_closure = self
            .builder
            .build_ptr_to_int(
                closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None),
                "",
            )
            .unwrap();
        self.builder
            .build_int_compare(
                IntPredicate::NE,
                has_closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
                    .const_zero(),
                "",
            )
            .unwrap()
    }

    /// Makes a function or procedure call, following standard calling convention.
    ///
    /// The current basic block is terminated, as Trilogy functions return by calling
    /// a continuation.
    fn make_call(
        &self,
        function_ptr: PointerValue<'ctx>,
        args: &[PointerValue<'ctx>],
        arity: usize,
        has_closure: bool,
    ) {
        let args_loaded: Vec<_> = args
            .iter()
            .map(|arg| self.load_value(*arg, "").into())
            .collect();
        let call = self
            .builder
            .build_indirect_call(
                self.procedure_type(arity, has_closure),
                function_ptr,
                &args_loaded,
                "",
            )
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();
    }

    /// Calls a standard callable (function or procedure).
    ///
    /// As per standard calling convention, it's basically `call/cc`:
    /// 1. The `return_to` is a newly constructed continuation, representing the current continuation ("returning" is done by `call_continuation`)
    /// 2. The rest of the pointers are maintained from the calling context
    /// 3. The arguments are provided
    /// 4. If the callable was a closure, the captures are pulled from the callable object.
    ///
    /// As per the general calling convention, the callable object itself is destroyed, and all
    /// arguments are moved into the call.
    fn call_callable(
        &self,
        continuation_function: FunctionValue<'ctx>,
        value: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
        function_ptr: PointerValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        name: &str,
    ) -> PointerValue<'ctx> {
        let arity = arguments.len();

        // Values must be extracted before they are invalidated
        let end_to = self.get_end("");
        let yield_to = self.get_yield("");
        let cancel_to = self.get_cancel("");
        let resume_to = self.get_resume("");
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");
        let bound_closure = self.allocate_value("");
        self.trilogy_callable_closure_into(bound_closure, callable, "");

        // All variables and values are invalid after this point.
        let return_to = self.close_current_continuation_as_return(continuation_function, "cc");
        self.trilogy_value_destroy(value);

        let mut args = Vec::with_capacity(arity + 8);
        args.extend([
            return_to,
            yield_to,
            end_to,
            cancel_to,
            resume_to,
            break_to,
            continue_to,
        ]);
        args.extend_from_slice(arguments);

        let has_closure = self.is_closure(bound_closure);
        let direct_block = self
            .context
            .append_basic_block(self.get_function(), "call.definition");
        let closure_block = self
            .context
            .append_basic_block(self.get_function(), "call.closure");
        self.builder
            .build_conditional_branch(has_closure, closure_block, direct_block)
            .unwrap();

        self.builder.position_at_end(direct_block);
        self.make_call(function_ptr, &args, arity, false);

        self.builder.position_at_end(closure_block);
        args.push(bound_closure);
        self.make_call(function_ptr, &args, arity, true);

        self.begin_next_function(continuation_function);
        self.get_continuation(name)
    }

    /// Calls a procedure value with the provided arguments.
    ///
    /// See `call_callable` for more information on the calling convention.
    pub(crate) fn call_procedure(
        &self,
        procedure: PointerValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        name: &str,
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(procedure, "");
        let arity = arguments.len();
        let function = self.trilogy_procedure_untag(callable, arity, "");
        let continuation_function = self.add_continuation("cc");
        self.call_callable(
            continuation_function,
            procedure,
            callable,
            function,
            arguments,
            name,
        )
    }

    /// Applies a function value to the provided argument. This may also be used to call
    /// a continuation value, as continuations and functions appear identically in Trilogy
    /// source code.
    ///
    /// See `call_callable` for more information on the calling convention.
    pub(crate) fn apply_function(
        &self,
        callable_value: PointerValue<'ctx>,
        argument: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(callable_value, "");
        let tag = self.get_callable_tag(callable, "");
        let call_function = self
            .context
            .append_basic_block(self.get_function(), "ap.func");
        let call_continuation = self
            .context
            .append_basic_block(self.get_function(), "ap.cont");
        let call_continue = self
            .context
            .append_basic_block(self.get_function(), "ap.continue");
        let call_resume = self
            .context
            .append_basic_block(self.get_function(), "ap.resume");

        self.builder
            .build_switch(
                tag,
                call_function,
                &[
                    (
                        self.tag_type().const_int(CALLABLE_CONTINUATION, false),
                        call_continuation,
                    ),
                    (
                        self.tag_type().const_int(CALLABLE_CONTINUE, false),
                        call_continue,
                    ),
                    (
                        self.tag_type().const_int(CALLABLE_RESUME, false),
                        call_resume,
                    ),
                ],
            )
            .unwrap();

        let brancher = self.end_continuation_point_as_branch();

        self.builder.position_at_end(call_continuation);
        self.call_regular_continuation(
            callable_value,
            callable,
            self.load_value(argument, "").into(),
        );

        self.builder.position_at_end(call_continue);
        self.resume_continuation_point(&brancher);
        self.call_continue_inner(callable_value, argument, "");

        let continuation_function = self.add_continuation("cc");
        self.builder.position_at_end(call_resume);
        self.resume_continuation_point(&brancher);
        self.call_resume_inner(
            continuation_function,
            callable_value,
            argument,
            Some(&brancher),
        );

        self.builder.position_at_end(call_function);
        let function = self.trilogy_function_untag(callable, "");
        self.resume_continuation_point(&brancher);
        self.call_callable(
            continuation_function,
            callable_value,
            callable,
            function,
            &[argument],
            name,
        )
    }

    /// Applies a continuation value to the provided argument.
    ///
    /// As per continuation calling convention:
    /// 1. The `return_to` and `yield_to` pointers are stored in the continuation object
    /// 2. The `end_to` pointer is always pulled from the calling context
    /// 3. The argument is provided
    /// 4. The closure is created from the current scope
    ///
    /// The continuation's code itself knows where it is to continue to, so as usual we do not need
    /// to provide that information. The current basic block is terminated, as there is no returning
    /// from a continuation call. The current implicit continuation point is ended as well, leaving
    /// us without a valid implicit continuation point.
    ///
    /// As per the general calling convention, the continuation object itself is destroyed, and all
    /// arguments are moved into the call.
    pub(crate) fn call_known_continuation(
        &self,
        continuation_value: PointerValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) {
        let callable = self.trilogy_callable_untag(continuation_value, "");
        self.call_regular_continuation(
            continuation_value,
            callable,
            self.load_value(argument, "").into(),
        );
    }

    /// Applies a continuation value, passing void as its argument. The called continuation must
    /// not refer to the value. See `call_continuation` for more info.
    pub(crate) fn void_call_continuation(&self, continuation_value: PointerValue<'ctx>) {
        let callable = self.trilogy_callable_untag(continuation_value, "");
        self.call_regular_continuation(
            continuation_value,
            callable,
            self.value_type().const_zero().into(),
        );
    }

    fn do_if<F: Fn()>(&self, bool_value: IntValue<'ctx>, body: F) {
        let true_block = self.context.append_basic_block(self.get_function(), "");
        let false_block = self.context.append_basic_block(self.get_function(), "");
        self.builder
            .build_conditional_branch(bool_value, true_block, false_block)
            .unwrap();
        self.builder.position_at_end(true_block);
        body();
        self.builder
            .build_unconditional_branch(false_block)
            .unwrap();
        self.builder.position_at_end(false_block);
    }

    fn call_regular_continuation(
        &self,
        continuation_value: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) {
        let continuation_pointer = self.trilogy_continuation_untag(callable, "");
        let return_to = self.allocate_value("");
        let yield_to = self.allocate_value("");
        let end_to = self.get_end("");
        let cancel_to = self.allocate_value("");
        let resume_to = self.allocate_value("");
        let break_to = self.allocate_value("");
        let continue_to = self.allocate_value("");
        let closure = self.allocate_value("");
        self.trilogy_callable_return_to_into(return_to, callable);
        self.do_if(self.is_undefined(return_to), || {
            self.clone_return(return_to);
        });
        self.trilogy_callable_yield_to_into(yield_to, callable);
        self.do_if(self.is_undefined(yield_to), || {
            self.clone_yield(yield_to);
        });
        self.trilogy_callable_cancel_to_into(cancel_to, callable);
        self.do_if(self.is_undefined(cancel_to), || {
            self.clone_cancel(cancel_to);
        });
        self.trilogy_callable_resume_to_into(resume_to, callable);
        self.do_if(self.is_undefined(resume_to), || {
            self.clone_resume(resume_to);
        });
        self.trilogy_callable_break_to_into(break_to, callable);
        self.do_if(self.is_undefined(break_to), || {
            self.clone_break(break_to);
        });
        self.trilogy_callable_continue_to_into(continue_to, callable);
        self.do_if(self.is_undefined(continue_to), || {
            self.clone_continue(continue_to);
        });
        self.trilogy_callable_closure_into(closure, callable, "");

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.load_value(cancel_to, "").into(),
            self.load_value(resume_to, "").into(),
            self.load_value(break_to, "").into(),
            self.load_value(continue_to, "").into(),
            argument,
            self.load_value(closure, "").into(),
        ];

        self.trilogy_value_destroy(continuation_value);

        // NOTE: cleanup will be inserted here
        let call = self
            .builder
            .build_indirect_call(self.continuation_type(), continuation_pointer, args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        let call = call
            .try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap();
        self.end_continuation_point_as_clean(call);
        self.builder.build_return(None).unwrap();
    }

    /// An internal function in this case is one that follows the convention of
    /// the first parameter being the output parameter, and all values being
    /// TrilogyValue pointers.
    ///
    /// Since an internal function cannot diverge in the way that Trilogy source
    /// functions can, the work of calling is reduced significantly.
    pub(crate) fn call_internal(
        &self,
        target: PointerValue<'ctx>,
        procedure: FunctionValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) -> InstructionValue<'ctx> {
        let mut args = vec![target.into()];
        args.extend_from_slice(arguments);
        let call = self.builder.build_call(procedure, &args, "").unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap()
    }

    /// An core function is much like an internal function, but is defined in C
    /// and called directly, so should not use FastCC.
    pub(crate) fn call_core(
        &self,
        target: PointerValue<'ctx>,
        procedure: FunctionValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) -> InstructionValue<'ctx> {
        let mut args = vec![target.into()];
        args.extend_from_slice(arguments);
        let call = self.builder.build_call(procedure, &args, "").unwrap();
        call.try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap()
    }

    /// Calls the contextual `yield` continuation.
    ///
    /// Calling `yield` is almost like calling a regular continuation, but we allocate a new
    /// `resume_to` value that corresponds to the continuation after the `yield`.
    ///
    /// Since `yield` cannot be captured in Trilogy code at this time, we do not need to have
    /// any special handling for calling yield elsewhere; we just special case it in the compiler.
    pub(crate) fn call_yield(&self, effect: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let handler_value = self.get_yield("");
        let handler = self.trilogy_callable_untag(handler_value, "");

        let end_to = self.get_end("");

        let return_to = self.allocate_value("");
        let yield_to = self.allocate_value("");
        let cancel_to = self.allocate_value("");
        let break_to = self.allocate_value("");
        let continue_to = self.allocate_value("");
        let closure = self.allocate_value("");
        self.trilogy_callable_return_to_into(return_to, handler);
        self.trilogy_callable_yield_to_into(yield_to, handler);
        self.trilogy_callable_cancel_to_into(cancel_to, handler);
        self.trilogy_callable_break_to_into(break_to, handler);
        self.trilogy_callable_continue_to_into(continue_to, handler);
        self.trilogy_callable_closure_into(closure, handler, "");

        let continuation_function = self.add_continuation("yield.resume");
        let resume_to =
            self.close_current_continuation_as_resume(continuation_function, "yield.resume");

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.load_value(cancel_to, "").into(),
            self.load_value(resume_to, "").into(),
            self.load_value(break_to, "").into(),
            self.load_value(continue_to, "").into(),
            self.load_value(effect, "").into(),
            self.load_value(closure, "").into(),
        ];
        let handler_continuation = self.trilogy_continuation_untag(handler, "");
        self.trilogy_value_destroy(handler_value);
        self.builder
            .build_indirect_call(self.continuation_type(), handler_continuation, args, name)
            .unwrap();
        self.builder.build_return(None).unwrap();

        self.begin_next_function(continuation_function);
        self.get_continuation(name)
    }

    /// Calls the contextual `resume` continuation.
    ///
    /// Calling `resume` causes a shift, as if a new handler was installed. This replaces the
    /// contextual `cancel_to` value so that when the current delimited continuation (`when`)
    /// is completed, we can go back into this handler.
    pub(crate) fn call_resume(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let resume_value = self.get_resume("");
        let continuation_function = self.add_continuation("resume.back");
        self.call_resume_inner(continuation_function, resume_value, value, None);
        self.begin_next_function(continuation_function);
        self.get_continuation(name)
    }

    fn call_resume_inner(
        &self,
        continuation_function: FunctionValue<'ctx>,
        resume_value: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
        branch: Option<&Brancher<'ctx>>,
    ) {
        let resume = self.trilogy_callable_untag(resume_value, "");
        let resume_continuation = self.trilogy_continuation_untag(resume, "");

        let end_to = self.get_end("");
        let resume_to = self.get_resume("");

        let return_to = self.allocate_value("");
        let yield_to = self.allocate_value("");
        let break_to = self.allocate_value("");
        let continue_to = self.allocate_value("");
        let closure = self.allocate_value("");
        let cancel_to =
            self.close_current_continuation_as_cancel(continuation_function, branch, "when.cancel");

        self.trilogy_callable_return_to_into(return_to, resume);

        self.trilogy_callable_yield_to_into(yield_to, resume);
        let cancel_clone = self.allocate_value("");
        self.trilogy_value_clone_into(cancel_clone, cancel_to);
        let resume_clone = self.allocate_value("");
        self.trilogy_value_clone_into(resume_clone, resume_to);
        self.trilogy_callable_promote(
            yield_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            cancel_clone,
            resume_clone,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
        );

        self.trilogy_callable_break_to_into(break_to, resume);
        self.trilogy_callable_continue_to_into(continue_to, resume);
        self.trilogy_callable_resume_to_into(resume_to, resume);
        self.trilogy_callable_closure_into(closure, resume, "");

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.load_value(cancel_to, "").into(),
            self.load_value(resume_to, "").into(),
            self.load_value(break_to, "").into(),
            self.load_value(continue_to, "").into(),
            self.load_value(value, "").into(),
            self.load_value(closure, "").into(),
        ];
        self.trilogy_value_destroy(resume_value);
        self.builder
            .build_indirect_call(self.continuation_type(), resume_continuation, args, "")
            .unwrap();
        self.builder.build_return(None).unwrap();
    }

    /// Calls the contextual `continue` continuation.
    ///
    /// Calling continue requires passing the same `continue_to` value to its own continuation,
    /// so that it may also continue to the same place.
    pub(crate) fn call_continue(&self, value: PointerValue<'ctx>, name: &str) {
        let continue_to = self.get_continue("");
        self.call_continue_inner(continue_to, value, name);
    }

    fn call_continue_inner(
        &self,
        continue_value: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
        name: &str,
    ) {
        let continue_callable = self.trilogy_callable_untag(continue_value, "");
        let continue_continuation = self.trilogy_continuation_untag(continue_callable, "");

        let end_to = self.get_end("");
        let return_to = self.allocate_value("");
        let yield_to = self.allocate_value("");
        let cancel_to = self.allocate_value("");
        let resume_to = self.allocate_value("");
        let break_to = self.allocate_value("");
        let closure = self.allocate_value("");
        self.trilogy_callable_return_to_into(return_to, continue_callable);
        self.do_if(self.is_undefined(return_to), || {
            self.clone_return(return_to);
        });
        self.trilogy_callable_yield_to_into(yield_to, continue_callable);
        self.do_if(self.is_undefined(yield_to), || {
            self.clone_yield(yield_to);
        });
        self.trilogy_callable_cancel_to_into(cancel_to, continue_callable);
        self.do_if(self.is_undefined(cancel_to), || {
            self.clone_cancel(cancel_to);
        });
        self.trilogy_callable_cancel_to_into(resume_to, continue_callable);
        self.do_if(self.is_undefined(resume_to), || {
            self.clone_resume(resume_to);
        });
        self.trilogy_callable_break_to_into(break_to, continue_callable);
        self.do_if(self.is_undefined(break_to), || {
            self.clone_break(break_to);
        });
        self.trilogy_callable_closure_into(closure, continue_callable, "");

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.load_value(cancel_to, "").into(),
            self.load_value(resume_to, "").into(),
            self.load_value(break_to, "").into(),
            self.load_value(continue_value, "").into(),
            self.load_value(value, "").into(),
            self.load_value(closure, "").into(),
        ];
        let call = self
            .builder
            .build_indirect_call(self.continuation_type(), continue_continuation, args, name)
            .unwrap()
            .try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap();
        self.builder.build_return(None).unwrap();
        self.end_continuation_point_as_clean(call);
    }

    /// Calls the `main` function as the Trilogy program entrypoint.
    ///
    /// This is similar to a standard procedure call, but because this is the first call
    /// in a program, we have to create the initial `return_to`, `yield_to`, and `end_to`
    /// continuations from scratch.
    pub(crate) fn call_main(&self, value: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let chain_function = self.module.add_function(
            "main.return",
            self.continuation_type(),
            Some(Linkage::Private),
        );

        let yield_function = self.module.add_function(
            "main.unhandled_effect",
            self.continuation_type(),
            Some(Linkage::Private),
        );

        let end_function =
            self.module
                .add_function("main.end", self.continuation_type(), Some(Linkage::Private));

        let callable = self.trilogy_callable_untag(value, "");
        let function = self.trilogy_procedure_untag(callable, 0, "");
        let return_continuation = self.allocate_value("return");
        let yield_continuation = self.allocate_value("yield");
        let end_continuation = self.allocate_value("end");
        let cancel_continuation = self.allocate_value("cancel");
        let resume_continuation = self.allocate_value("resume");
        let break_continuation = self.allocate_value("break");
        let continue_continuation = self.allocate_value("continue");

        let return_closure = self.allocate_value("main.ret");
        let yield_closure = self.allocate_value("main.yield");
        let end_closure = self.allocate_value("main.end");
        let cancel_closure = self.allocate_value("main.cancel");
        let resume_closure = self.allocate_value("main.resume");
        let break_closure = self.allocate_value("main.break");
        let continue_closure = self.allocate_value("main.continue");
        self.trilogy_array_init_cap(return_closure, 0, "");
        self.trilogy_array_init_cap(yield_closure, 0, "");
        self.trilogy_array_init_cap(end_closure, 0, "");
        self.trilogy_array_init_cap(cancel_closure, 0, "");
        self.trilogy_array_init_cap(resume_closure, 0, "");
        self.trilogy_array_init_cap(break_closure, 0, "");
        self.trilogy_array_init_cap(continue_closure, 0, "");
        self.trilogy_callable_init_cont(
            return_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            return_closure,
            chain_function,
        );
        self.trilogy_callable_init_cont(
            yield_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            yield_closure,
            yield_function,
        );
        self.trilogy_callable_init_cont(
            end_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            end_closure,
            end_function,
        );
        self.trilogy_callable_init_cont(
            cancel_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            cancel_closure,
            end_function,
        );
        self.trilogy_callable_init_cont(
            resume_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            resume_closure,
            end_function,
        );
        self.trilogy_callable_init_cont(
            break_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            break_closure,
            end_function,
        );
        self.trilogy_callable_init_cont(
            continue_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            continue_closure,
            end_function,
        );

        let args = &[
            self.load_value(return_continuation, "").into(),
            self.load_value(yield_continuation, "").into(),
            self.load_value(end_continuation, "").into(),
            self.load_value(cancel_continuation, "").into(),
            self.load_value(resume_continuation, "").into(),
            self.load_value(break_continuation, "").into(),
            self.load_value(continue_continuation, "").into(),
        ];
        self.trilogy_value_destroy(value);
        let call = self
            .builder
            .build_indirect_call(self.procedure_type(0, false), function, args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindNone);
        self.builder.build_return(None).unwrap();

        let entry = self.context.append_basic_block(yield_function, "entry");
        self.builder.position_at_end(entry);
        let effect = self.allocate_value("effect");
        self.builder
            .build_store(effect, self.get_function().get_nth_param(7).unwrap())
            .unwrap();
        _ = self.trilogy_unhandled_effect(effect);

        let entry = self.context.append_basic_block(end_function, "entry");
        self.builder.position_at_end(entry);
        _ = self.trilogy_execution_ended();

        let entry = self.context.append_basic_block(chain_function, "entry");
        self.builder.position_at_end(entry);
        let result = self.allocate_value("result");
        self.builder
            .build_store(result, self.get_function().get_nth_param(7).unwrap())
            .unwrap();
        result
    }
}
