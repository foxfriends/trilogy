use super::{Labeler, Scope};
use crate::prelude::*;
use trilogy_ir::ir::{self, Iterator};
use trilogy_ir::visitor::HasBindings;
use trilogy_ir::Id;
use trilogy_vm::{Atom, ChunkBuilder, ChunkWriter, Instruction, Offset, Value};

pub(crate) struct Context<'a> {
    labeler: &'a mut Labeler,
    pub scope: Scope<'a>,
    builder: &'a mut ChunkBuilder,
}

impl<'a> Context<'a> {
    pub fn new(builder: &'a mut ChunkBuilder, labeler: &'a mut Labeler, scope: Scope<'a>) -> Self {
        Self {
            labeler,
            scope,
            builder,
        }
    }
}

impl ChunkWriter for Context<'_> {
    fn reference<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.reference(label);
        self
    }

    fn cond_jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.cond_jump(label);
        self
    }

    fn jump<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.jump(label);
        self
    }

    fn shift<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.shift(label);
        self
    }

    fn close<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.close(label);
        self
    }

    fn instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.builder.instruction(instruction);
        self
    }

    fn label<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.builder.label(label);
        self
    }

    fn constant<V: Into<Value>>(&mut self, value: V) -> &mut Self {
        self.builder.constant(value);
        self
    }

    fn make_atom<S: AsRef<str>>(&self, value: S) -> Atom {
        self.builder.make_atom(value)
    }
}

impl LabelMaker for Context<'_> {
    fn make_label(&mut self, label: &str) -> String {
        self.labeler.unique_hint(label)
    }
}

impl StackTracker for Context<'_> {
    fn intermediate(&mut self) -> Offset {
        self.scope.intermediate()
    }

    fn end_intermediate(&mut self) -> &mut Self {
        self.scope.end_intermediate();
        self
    }

    fn push_continue(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_continue(offset);
        self
    }

    fn pop_continue(&mut self) -> &mut Self {
        self.scope.pop_continue();
        self
    }

    fn push_break(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_break(offset);
        self
    }

    fn pop_break(&mut self) -> &mut Self {
        self.scope.pop_break();
        self
    }

    fn push_cancel(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_cancel(offset);
        self
    }

    fn pop_cancel(&mut self) -> &mut Self {
        self.scope.pop_cancel();
        self
    }

    fn push_resume(&mut self, offset: Offset) -> &mut Self {
        self.scope.push_resume(offset);
        self
    }

    fn pop_resume(&mut self) -> &mut Self {
        self.scope.pop_resume();
        self
    }
}

impl Context<'_> {
    pub fn declare_variables(&mut self, variables: impl IntoIterator<Item = Id>) -> usize {
        let mut n = 0;
        for id in variables {
            if self.scope.declare_variable(id.clone()) {
                let label = self.labeler.var(&id);
                self.label(label);
                self.instruction(Instruction::Variable);
                n += 1;
            }
        }
        n
    }

    pub fn undeclare_variables(
        &mut self,
        variables: impl IntoIterator<Item = Id>,
        pop: bool,
    ) -> &mut Self {
        for id in variables {
            if self.scope.undeclare_variable(&id) && pop {
                let label = self.labeler.unvar(&id);
                self.label(label);
                self.instruction(Instruction::Pop);
            }
        }
        self
    }

    pub fn iterator(
        &mut self,
        iterator: &Iterator,
        r#continue: Option<Offset>,
        r#break: Option<Offset>,
    ) -> &mut Self {
        self.declare_variables(iterator.query.bindings());
        write_query_state(self, &iterator.query);
        self.repeat(|context, exit| {
            write_query(context, &iterator.query, exit);

            context.intermediate(); // state
            if let Some(r#break) = r#break {
                context.push_break(r#break);
            }
            if let Some(r#continue) = r#continue {
                context.push_continue(r#continue);
            }

            match &iterator.value.value {
                ir::Value::Mapping(mapping) => {
                    write_expression(context, &mapping.0);
                    context.intermediate();
                    write_expression(context, &mapping.1);
                    context.end_intermediate().instruction(Instruction::Cons);
                }
                other => write_evaluation(context, other),
            }

            if r#continue.is_some() {
                context.pop_continue();
            }
            if r#break.is_some() {
                context.pop_break();
            }
            context
                .atom("next")
                .instruction(Instruction::Construct)
                .r#yield()
                .instruction(Instruction::Pop) // resume value discarded
                .end_intermediate(); // state no longer intermediate
        })
        .instruction(Instruction::Pop)
        .undeclare_variables(iterator.query.bindings(), true);
        self
    }

    pub fn evaluate(&mut self, value: &ir::Value) -> &mut Self {
        write_evaluation(self, value);
        self
    }
}

impl Context<'_> {
    pub fn sequence(&mut self, seq: &[ir::Expression]) -> &mut Self {
        let mut seq = seq.iter();
        let Some(mut expr) = seq.next() else {
            // An empty sequence must still have a value
            return self.constant(());
        };
        loop {
            self.evaluate(&expr.value);
            let Some(next_expr) = seq.next() else {
                break self;
            };
            expr = next_expr;
            self.instruction(Instruction::Pop);
        }
    }

    pub fn comprehension<F: FnOnce(&mut Context), G: FnOnce(&mut Context)>(
        &mut self,
        append: F,
        init: G,
    ) -> &mut Self {
        self.iterate(
            |context, params| {
                context
                    .instruction(Instruction::LoadLocal(params.cancel))
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::LoadLocal(params.resume))
                    .constant(())
                    .call_function()
                    .instruction(Instruction::Clone)
                    .instruction(Instruction::Swap)
                    .pipe(append)
                    .become_function();
            },
            init,
        )
    }

    pub fn r#loop<
        F: FnOnce(&mut Context),
        G: FnOnce(&mut Context, &str),
        H: FnOnce(&mut Context),
        I: FnOnce(&mut Context),
    >(
        &mut self,
        setup: F,
        head: G,
        body: H,
        cleanup: I,
    ) -> &mut Self {
        let begin = self.make_label("loop");
        let done = self.make_label("loop_done");
        let end = self.make_label("loop_end");

        // Break is just a continuation that points to the end of the loop. The value
        // passed to break is discarded
        let r#break = self
            .continuation_fn(|c| {
                c.instruction(Instruction::Pop).jump(&end);
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
            .continuation_fn(|c| {
                c.instruction(Instruction::Pop).jump(&begin);
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

    pub fn r#while(&mut self, condition: &ir::Value, body: &ir::Value) -> &mut Self {
        self.r#loop(
            |_| {},
            |context, done| {
                context
                    .evaluate(condition)
                    .typecheck("boolean")
                    .cond_jump(done);
            },
            |context| {
                context.evaluate(body);
            },
            |_| {},
        )
        // Evaluation requires that an extra value ends up on the stack.
        // While "evaluates" to unit
        .constant(())
    }

    pub fn r#for(&mut self, query: &ir::Query, body: &ir::Value) -> &mut Self {
        let did_match = self.constant(false).intermediate();
        self.r#loop(
            |context| {
                context.declare_variables(query.bindings());
                write_query_state(context, query);
            },
            |context, done| {
                write_query(context, query, done);
                context
                    // Mark down that this loop did get a match
                    .constant(true)
                    .instruction(Instruction::SetLocal(did_match))
                    .intermediate(); // query state
            },
            |context| {
                context.evaluate(body).end_intermediate(); // query state
            },
            |context| {
                context
                    .instruction(Instruction::Pop)
                    .undeclare_variables(query.bindings(), true);
            },
        )
        .end_intermediate() // did match (no longer intermediate)
    }
}
