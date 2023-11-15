use crate::prelude::*;
pub(crate) use trilogy_vm::ChunkWriter;
pub(crate) use trilogy_vm::Instruction;

pub(crate) trait StatefulChunkWriterExt:
    StackTracker + ChunkWriter + LabelMaker + Sized
{
    fn r#continue<S: Into<String>>(&mut self, label: S) -> &mut Self {
        let cont = self.intermediate();
        self.instruction(Instruction::Variable)
            .continuation(|context| {
                // Continue is called with a value that is ignored. This is definitely an oversight
                // that I should get around to fixing... or maybe there's a way to use that value?
                context
                    .unlock_function()
                    .instruction(Instruction::Pop)
                    .jump(label);
            })
            .instruction(Instruction::SetLocal(cont));
        self.end_intermediate();
        self
    }

    fn r#break<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.continuation(|context| {
            // Break is called with a value that is ignored. This is definitely an oversight
            // that I should get around to fixing... or maybe there's a way to use that value?
            context
                .unlock_function()
                .instruction(Instruction::Pop)
                .jump(label);
        })
    }

    fn with<F: FnOnce(&mut Self, Offset), G: FnOnce(&mut Self)>(
        &mut self,
        handlers: F,
        handled: G,
    ) -> &mut Self {
        let end = self.make_label("with_end");

        // Effect handlers are implemented using continuations and a single global cell (Register(HANDLER)).
        // There's a few extra things that are held in context too though, which must be preserved in order
        // to effectively save and restore the program state.
        // The module context must be preserved, as the yield of the effect may be in a different module
        // than the handler is defined.
        let stored_context = self
            .instruction(Instruction::LoadRegister(MODULE))
            .intermediate();
        // The parent handler is preserved so that a yield in response to a yield correctly moves up
        // the chain.
        let stored_yield = self
            .instruction(Instruction::LoadRegister(HANDLER))
            .intermediate();

        // First step of entering an effect handler is to create the "cancel" continuation
        // (effectively defining the "reset" operator). From the top level, to reset is to
        // simply exit the effect handling. This operator will get replaced each time
        // a handler calls `resume` such that the `cancel` points to the last resume.
        let cancel = self
            .continuation(|c| {
                c.unlock_function().jump(&end);
            })
            .intermediate();
        self.push_cancel(cancel);

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
                .closure(|context| {
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
            context.push_resume(actual_resume);

            // Immediately restore the parent `yield` and module context, as a handler may use it.
            // The yielder's values aren't needed though, as the `yield` expression itself takes
            // care of saving that to restore it when resumed.
            context
                .instruction(Instruction::LoadLocal(stored_yield))
                .instruction(Instruction::SetRegister(HANDLER))
                .instruction(Instruction::LoadLocal(stored_context))
                .instruction(Instruction::SetRegister(MODULE));

            context.pipe(|c| handlers(c, effect));

            // NOTE: this should be unreachable, seeing as effect handlers are required
            // to include the `else` clause... so if it happens lets fail in a weird way.
            context
                .constant("unexpected unhandled effect")
                .instruction(Instruction::Panic);
            context.pop_resume();
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
            .pop_cancel()
            .label(end)
            // Once we're out of the handler reset the state of the `yield` register and finally done!
            .instruction(Instruction::Swap)
            .instruction(Instruction::SetRegister(HANDLER))
            .instruction(Instruction::Swap)
            .instruction(Instruction::SetRegister(MODULE))
            .end_intermediate() // stored yield
            .end_intermediate() // stored module
    }
}

impl<T> StatefulChunkWriterExt for T where T: StackTracker + ChunkWriter + LabelMaker {}
