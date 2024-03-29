use crate::needs::BreakContinue;
use crate::prelude::*;
pub(crate) use trilogy_vm::ChunkWriter;
pub(crate) use trilogy_vm::Instruction;

#[derive(Copy, Clone)]
pub(crate) struct HandlerParameters {
    pub effect: Offset,
    pub cancel: Offset,
    pub resume: Offset,
}

pub(crate) trait StatefulChunkWriterExt:
    StackTracker + ChunkWriter + LabelMaker + Sized
{
    fn continuation_fn<F: FnOnce(&mut Self)>(&mut self, body: F) -> &mut Self {
        self.continuation(|c| {
            c.unlock_function()
                .instruction(Instruction::Slide(2))
                .instruction(Instruction::Pop)
                .instruction(Instruction::Pop)
                .pipe(body);
        })
    }

    fn closure<F: FnOnce(&mut Self, &dyn Fn(&mut Self))>(&mut self, params: F) -> &mut Self {
        let end = self.make_label("closure_end");
        let module = self
            .instruction(Instruction::LoadRegister(MODULE))
            .intermediate();
        self.close(&end)
            .pipe(|c| {
                params(c, &|c: &mut Self| {
                    c.instruction(Instruction::LoadLocal(module))
                        .instruction(Instruction::SetRegister(MODULE));
                })
            })
            .label(end)
            .instruction(Instruction::Swap)
            .instruction(Instruction::Pop)
            .end_intermediate()
    }

    fn fn_closure<F: FnOnce(&mut Self)>(&mut self, arity: usize, contents: F) -> &mut Self {
        self.closure(|context, restore| {
            context.unlock_function();
            for _ in 0..arity - 1 {
                context.close(RETURN).unlock_function();
            }
            context.pipe(restore).pipe(contents);
        })
    }

    fn func_closure<S: Into<String>>(&mut self, arity: usize, func: S) -> &mut Self {
        self.closure(|context, restore| {
            context.unlock_function();
            for _ in 0..arity - 1 {
                context.close(RETURN).unlock_function();
            }
            context.pipe(restore);
            let parameters = context.intermediate();
            context.reference(func);
            for i in 0..arity - 1 {
                context
                    .instruction(Instruction::LoadLocal(parameters + i as Offset))
                    .call_function();
            }
            context
                .instruction(Instruction::LoadLocal(parameters + arity as Offset - 1))
                .become_function()
                .instruction(Instruction::Return)
                .end_intermediate();
        })
    }

    fn do_closure<F: FnOnce(&mut Self)>(&mut self, arity: usize, contents: F) -> &mut Self {
        self.closure(|context, restore| {
            context.unlock_procedure(arity).pipe(restore).pipe(contents);
        })
    }

    fn proc_closure<S: Into<String>>(&mut self, arity: usize, proc: S) -> &mut Self {
        self.closure(|context, restore| {
            context
                .unlock_procedure(arity)
                .pipe(restore)
                .reference(proc)
                .instruction(Instruction::Slide(arity as Offset))
                .become_procedure(arity);
        })
    }

    fn qy_closure<F: FnOnce(&mut Self), G: FnOnce(&mut Self, Offset)>(
        &mut self,
        arity: usize,
        setup: F,
        body: G,
    ) -> &mut Self {
        self.closure(|context, restore| {
            let state = context
                .close(RETURN)
                .unlock_rule(arity)
                .pipe(restore)
                .pipe(setup)
                .intermediate();
            context
                .close(RETURN)
                .pipe(restore)
                .pipe(|c| body(c, state))
                .end_intermediate();
        })
    }

    fn rule_closure<S: Into<String>>(&mut self, arity: usize, rule: S) -> &mut Self {
        self.closure(|context, restore| {
            let closure = context
                .pipe(restore)
                .reference(rule)
                .instruction(Instruction::Call(0))
                .intermediate();
            let iterator = context
                .close(RETURN)
                .unlock_rule(arity)
                .pipe(restore)
                .instruction(Instruction::LoadLocal(closure))
                .instruction(Instruction::Slide(arity as Offset))
                .call_rule(arity)
                .intermediate();
            context
                .close(RETURN)
                .pipe(restore)
                .instruction(Instruction::LoadLocal(iterator))
                .instruction(Instruction::Become(0))
                .end_intermediate()
                .end_intermediate();
        })
    }

    fn continuation<F: FnOnce(&mut Self)>(&mut self, body: F) -> &mut Self {
        let module = self
            .instruction(Instruction::LoadRegister(MODULE))
            .intermediate();
        let handler = self
            .instruction(Instruction::LoadRegister(HANDLER))
            .intermediate();

        let end = self.make_label("continuation_end");
        self.shift(&end)
            .instruction(Instruction::LoadLocal(module))
            .instruction(Instruction::SetRegister(MODULE))
            .instruction(Instruction::LoadLocal(handler))
            .instruction(Instruction::SetRegister(HANDLER))
            .pipe(body)
            .label(end)
            .instruction(Instruction::Slide(2))
            .instruction(Instruction::Pop)
            .end_intermediate()
            .instruction(Instruction::Pop)
            .end_intermediate()
    }

    fn with<F: FnOnce(&mut Self, HandlerParameters), G: FnOnce(&mut Self)>(
        &mut self,
        handlers: F,
        handled: G,
    ) -> &mut Self {
        let end = self.make_label("with_end");

        // First step of entering an effect handler is to create the "cancel" continuation
        // (effectively defining the "reset" operator). From the top level, to reset is to
        // simply exit the effect handling. This operator will get replaced each time
        // a handler calls `resume` such that the `cancel` points to the last resume.
        let cancel = self
            .continuation_fn(|c| {
                c.jump(&end);
            })
            .intermediate();

        // The new yield is created next.
        self.continuation(|context| {
            // While every other continuation is treated like a function (with unlock_apply)
            // the yield is special since it can't actually be accessed by the programmer
            // directly, so can never be incorrectly called, so does not have to be unlocked.
            // It's also called with 2 arguments instead of 1 like any other continuation.

            // That new yield will be called with the effect and the resume continuation.
            let effect = context.intermediate();
            let resume = context.intermediate();

            // While the caller gave us their half of the resume operator, we have to wrap
            // it so that it preserves all the context correctly.
            let actual_resume = context
                .closure(|context, restore| {
                    context.pipe(restore);
                    context.unlock_function();
                    let resume_value = context.intermediate();
                    // To actually resume is to:
                    // 1. Save the current cancellation
                    let prev_cancel = context
                        .instruction(Instruction::LoadLocal(cancel))
                        .intermediate();
                    // 2. Put a new cancellation in its place:
                    context.continuation(|c| {
                        c.unlock_function()
                            // This cancellation restores the previous one
                            .instruction(Instruction::LoadLocal(prev_cancel))
                            .instruction(Instruction::SetLocal(cancel))
                            // Then returns to whoever called resume
                            .instruction(Instruction::Return);
                    });
                    context.instruction(Instruction::SetLocal(cancel));
                    // 3. Actually do the resuming
                    context
                        .instruction(Instruction::LoadLocal(resume))
                        .instruction(Instruction::LoadLocal(resume_value))
                        .become_function();
                    context.end_intermediate(); // prev cancel
                    context.end_intermediate(); // resume value
                })
                .intermediate();

            context.pipe(|c| {
                handlers(
                    c,
                    HandlerParameters {
                        effect,
                        cancel,
                        resume: actual_resume,
                    },
                )
            });
            // NOTE: this should be unreachable, seeing as effect handlers are required
            // to include the `else` clause... so if it happens lets fail in a weird way.
            context
                .atom("UnexpectedUnhandledEffect")
                .instruction(Instruction::Construct)
                .instruction(Instruction::Panic);
            context.end_intermediate(); // actual resume
            context.end_intermediate(); // resume
            context.end_intermediate(); // effect
        });

        // The body of the `when` statement involves saving the `yield` that was just created,
        // running the expression, and then cleaning up.
        self.instruction(Instruction::SetRegister(HANDLER))
            .pipe(handled)
            // When the expression finishes evaluation, we reset from any shifted continuations
            // by calling the cancel continuation.
            .instruction(Instruction::LoadLocal(cancel))
            .instruction(Instruction::Swap)
            .become_function()
            .end_intermediate() // cancel
            .label(end)
    }

    fn r#loop<
        F: FnOnce(&mut Self),
        G: FnOnce(&mut Self, &str),
        H: FnOnce(&mut Self),
        I: FnOnce(&mut Self),
    >(
        &mut self,
        setup: F,
        head: G,
        body: H,
        cleanup: I,
        needs: BreakContinue,
    ) -> &mut Self {
        let begin = self.make_label("loop");
        let done = self.make_label("loop_done");
        let end = self.make_label("loop_end");

        // Break is just a continuation that points to the end of the loop. The value
        // passed to break is discarded
        let r#break = self
            .pipe(|c| {
                if needs.fc_break || needs.st_break {
                    c.continuation_fn(|c| {
                        c.instruction(Instruction::Pop).jump(&end);
                    });
                } else {
                    c.instruction(Instruction::Unit);
                }
            })
            .intermediate();
        // The actual loop we can implement in the standard way after the continuations are
        // created.
        self.pipe(setup)
            .label(&begin)
            // Check the condition
            .pipe(|c| head(c, &done));
        // If it's true, run the body. The body has access to continue and break.
        // Continue is a continuation much like break, but it points to the start of the loop
        let r#continue = self
            .pipe(|c| {
                if needs.fc_continue || needs.st_continue {
                    c.continuation_fn(|c| {
                        c.instruction(Instruction::Pop).jump(&begin);
                    });
                } else {
                    c.instruction(Instruction::Unit);
                }
            })
            .intermediate();
        self.push_break(r#break)
            .push_continue(r#continue)
            .pipe(body)
            .pop_break()
            .pop_continue()
            .instruction(Instruction::Pop) // Body value
            .instruction(Instruction::Pop) // Continue
            .end_intermediate()
            .jump(&begin)
            .label(&done)
            .pipe(cleanup)
            .instruction(Instruction::Pop) // break
            .end_intermediate()
            .label(&end)
    }

    fn iterate<F: FnOnce(&mut Self, HandlerParameters), G: FnOnce(&mut Self)>(
        &mut self,
        handler: F,
        iterator: G,
    ) -> &mut Self {
        self.with(
            |context, params| {
                context
                    .case(|context, next| {
                        context
                            .instruction(Instruction::LoadLocal(params.effect))
                            .unwrap_next(next)
                            .pipe(|c| handler(c, params));
                    })
                    .case(|context, _next| {
                        context
                            .instruction(Instruction::LoadLocal(params.cancel))
                            .instruction(Instruction::LoadLocal(params.resume))
                            .instruction(Instruction::LoadLocal(params.effect))
                            .r#yield()
                            .call_function()
                            .call_function();
                    });
            },
            iterator,
        )
    }
}

impl<T> StatefulChunkWriterExt for T where T: StackTracker + ChunkWriter + LabelMaker {}
