use crate::context::Context;
use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_ir::visitor::{HasBindings, HasCanEvaluate};
use trilogy_vm::Instruction;

pub(crate) fn write_rule(context: &mut Context, rule: &ir::Rule, on_fail: &str) {
    let next = context.atom("next");
    let done = context.atom("done");

    let setup = context.labeler.unique_hint("setup");
    let end = context.labeler.unique_hint("end");
    let call = context.labeler.unique_hint("call");

    context
        .write_instruction(Instruction::Copy)
        .write_instruction(Instruction::Const(().into()))
        .write_instruction(Instruction::ValNeq)
        .cond_jump(&setup)
        .write_label(call.clone())
        .write_instruction(Instruction::Copy)
        .write_instruction(Instruction::Call(0))
        // After calling this overload's iterator, flatten failures into the parent.
        // The parent expects this to work more like a query than an iterator for
        // failures.
        .write_instruction(Instruction::Copy)
        .write_instruction(Instruction::Const(done.clone().into()))
        .write_instruction(Instruction::ValEq)
        .cond_jump(&end)
        .write_instruction(Instruction::Pop)
        .write_instruction(Instruction::Pop)
        .jump(on_fail);

    context
        .write_label(setup)
        .write_instruction(Instruction::Pop);
    // First check all the parameters, make sure they work. If they don't
    // match, we can fail without even constructing the state.
    let mut cleanup = vec![];
    for (i, parameter) in rule.parameters.iter().enumerate() {
        let skip = context.labeler.unique_hint("skip");
        cleanup.push(context.labeler.unique_hint("cleanup"));
        context.declare_variables(parameter.bindings());
        context
            .write_instruction(Instruction::LoadLocal(1))
            .write_instruction(Instruction::Const(i.into()))
            .write_instruction(Instruction::Contains)
            .cond_jump(&skip)
            .write_instruction(Instruction::LoadLocal(2 + i));
        write_pattern_match(context, parameter, &cleanup[i]);
        context.write_label(skip);
    }

    // Happy path: we continue by writing the query state down, and then
    // encapsulating all that into a closure which will be the iterator for
    // this overload of the rule.
    write_query_state(context, &rule.body);
    context.close(&call);

    // The actual body of the rule involves running the query, then
    // returning the return value in 'next. We convert failure to
    // returning 'done, as in a regular iterator.
    let on_done = context.labeler.unique_hint("on_done");
    let actual_state = context.scope.intermediate();
    context.write_instruction(Instruction::LoadLocal(actual_state));
    write_query(context, &rule.body, &on_done, Some(1));
    context.write_instruction(Instruction::SetLocal(actual_state));
    context.scope.end_intermediate();
    // The query is normal, then the value is computed by evaluating
    // the parameter patterns now as expressions.
    //
    // TODO[Optimization]: Parameters that were already fully bound
    // can just be loaded directly instead of re-evaluated.
    context.scope.intermediate(); // At this point, the query state is an intermediate
    for (i, param) in rule.parameters.iter().enumerate() {
        let eval = context.labeler.unique_hint("eval");
        let next = context.labeler.unique_hint("next");
        context
            .write_instruction(Instruction::LoadLocal(1))
            .write_instruction(Instruction::Const(i.into()))
            .write_instruction(Instruction::Contains)
            .cond_jump(&eval)
            .write_instruction(Instruction::LoadLocal(2 + i))
            .jump(&next);
        context.write_label(eval);
        if param.can_evaluate() {
            write_expression(context, param);
        } else {
            context.write_instruction(Instruction::Fizzle);
        }
        context.scope.intermediate(); // As is each subsequent parameter value
        context.write_label(next);
    }
    // The return value is a (backwards) list
    context.write_instruction(Instruction::Const(().into()));
    for _ in &rule.parameters {
        context
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Cons);
        context.scope.end_intermediate();
    }
    // Finally, put the return value into 'next()
    context
        .write_instruction(Instruction::Const(next.into()))
        .write_instruction(Instruction::Swap)
        .write_instruction(Instruction::Construct)
        .write_instruction(Instruction::Return);
    // This ends with 2 expected values on the stack ([state, retval])
    // so they are no longer intemediate
    context.scope.end_intermediate();

    // On failure, just return 'done
    context
        .write_label(on_done)
        .write_instruction(Instruction::Const(done.into()))
        .write_instruction(Instruction::Return);

    // Sad path: we have to undeclare all the variables that have been
    // declared so far, then fail as regular.
    for parameter in rule.parameters.iter().rev() {
        context.write_label(cleanup.pop().unwrap());
        context.undeclare_variables(parameter.bindings(), true);
    }
    context.jump(on_fail);

    context.write_label(end);
}
